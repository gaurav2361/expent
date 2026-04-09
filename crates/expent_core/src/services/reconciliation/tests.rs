use super::matching::*;
use chrono::Duration;
use chrono::{TimeZone, Utc};
use db::entities::enums::{TransactionDirection, TransactionSource, TransactionStatus};
use db::entities::{bank_statement_rows, transactions};
use rstest::{fixture, rstest};
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ConnectionTrait, Database, DatabaseConnection, DbBackend, Schema, Set,
    Statement,
};

async fn setup_db() -> DatabaseConnection {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    db.execute(Statement::from_string(
        DbBackend::Sqlite,
        "PRAGMA foreign_keys = OFF;",
    ))
    .await
    .unwrap();

    let builder = db.get_database_backend();
    let schema = Schema::new(builder);

    let stmt_users = builder.build(&schema.create_table_from_entity(db::entities::users::Entity));
    let stmt_wallets =
        builder.build(&schema.create_table_from_entity(db::entities::wallets::Entity));
    let stmt_categories =
        builder.build(&schema.create_table_from_entity(db::entities::categories::Entity));
    let stmt_groups = builder.build(&schema.create_table_from_entity(db::entities::groups::Entity));
    let stmt_ledger_tabs =
        builder.build(&schema.create_table_from_entity(db::entities::ledger_tabs::Entity));

    db.execute(stmt_users).await.unwrap();
    db.execute(stmt_groups).await.unwrap();
    db.execute(stmt_wallets).await.unwrap();
    db.execute(stmt_categories).await.unwrap();
    db.execute(stmt_ledger_tabs).await.unwrap();

    let stmt1 = builder.build(&schema.create_table_from_entity(bank_statement_rows::Entity));
    let stmt2 = builder.build(&schema.create_table_from_entity(transactions::Entity));

    db.execute(stmt1).await.unwrap();
    db.execute(stmt2).await.unwrap();

    db
}

fn base_row() -> bank_statement_rows::ActiveModel {
    bank_statement_rows::ActiveModel {
        id: Set("row_1".to_string()),
        date: Set(Utc.with_ymd_and_hms(2024, 1, 15, 0, 0, 0).unwrap().into()),
        description: Set("AMAZON PURCHASE".to_string()),
        debit: Set(Some(Decimal::new(10000, 2))),
        credit: Set(None),
        balance: Set(Decimal::new(100000, 2)),
        user_id: Set("user_1".to_string()),
        is_matched: Set(false),
    }
}

fn base_txn() -> transactions::ActiveModel {
    transactions::ActiveModel {
        id: Set("txn_1".to_string()),
        user_id: Set("user_1".to_string()),
        amount: Set(Decimal::new(10000, 2)),
        direction: Set(TransactionDirection::Out),
        date: Set(Utc.with_ymd_and_hms(2024, 1, 15, 0, 0, 0).unwrap().into()),
        source: Set(TransactionSource::Manual),
        status: Set(TransactionStatus::Completed),
        purpose_tag: Set(Some("AMAZON".to_string())),
        group_id: Set(None),
        source_wallet_id: Set(None),
        destination_wallet_id: Set(None),
        ledger_tab_id: Set(None),
        category_id: Set(None),
        deleted_at: Set(None),
        notes: Set(None),
    }
}

#[fixture]
async fn test_db() -> DatabaseConnection {
    setup_db().await
}

#[rstest]
#[case::happy_path(true, 0, true, Ok(100))]
#[case::boundary_date_limit(true, 3, false, Ok(80))]
#[case::negative_not_found(false, 0, true, Err("Resource not found: Statement row not found".to_string()))]
#[tokio::test]
async fn test_get_row_matches(
    #[future] test_db: DatabaseConnection,
    #[case] row_exists: bool,
    #[case] days_offset: i64,
    #[case] keyword_matches: bool,
    #[case] expected: Result<i32, String>,
) {
    let db = test_db.await;

    if row_exists {
        let row = base_row();
        let mut txn = base_txn();

        let base_date = Utc.with_ymd_and_hms(2024, 1, 15, 0, 0, 0).unwrap();
        txn.date = Set((base_date + Duration::days(days_offset)).into());

        if !keyword_matches {
            txn.purpose_tag = Set(Some("WALMART".to_string()));
        }

        row.insert(&db).await.unwrap();
        txn.insert(&db).await.unwrap();
    }

    let result = get_row_matches(&db, "user_1", "row_1").await;

    match expected {
        Ok(expected_score) => {
            let matches = result.expect("Expected Ok, got Err");
            assert_eq!(matches.len(), 1);
            assert_eq!(matches[0].1, expected_score);
        }
        Err(e_msg) => {
            let err = result.expect_err("Expected Err, got Ok");
            assert_eq!(err.to_string(), e_msg);
        }
    }
}
