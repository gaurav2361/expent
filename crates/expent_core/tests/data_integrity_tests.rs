mod helpers;

use chrono::Utc;
use ::db::entities::enums::{TransactionDirection, TransactionSource, GroupRole};
use ::db::entities::{self, transactions, wallets};
use expent_core::services::transactions::{create_transaction, delete_transaction, update_transaction};
use helpers::db::setup_test_db;
use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, Set};
use uuid::Uuid;

async fn create_user(db: &DatabaseConnection) -> String {
    let id = Uuid::now_v7().to_string();
    entities::users::ActiveModel {
        id: Set(id.clone()),
        email: Set(format!("test{}@example.com", id)),
        name: Set("Test User".to_string()),
        is_active: Set(true),
        email_verified: Set(true),
        banned: Set(Some(false)),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
        image: Set(None),
        role: Set(Some(GroupRole::Admin)),
        two_factor_enabled: Set(Some(false)),
        ..Default::default()
    }
    .insert(db)
    .await
    .unwrap();
    id
}

async fn create_wallet(db: &DatabaseConnection, user_id: &str, balance: Decimal) -> String {
    let id = Uuid::now_v7().to_string();
    entities::wallets::ActiveModel {
        id: Set(id.clone()),
        user_id: Set(user_id.to_string()),
        name: Set("Test Wallet".to_string()),
        balance: Set(balance),
        r#type: Set("default".to_string()),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
        ..Default::default()
    }
    .insert(db)
    .await
    .unwrap();
    id
}

#[tokio::test]
async fn test_wallet_ledger_parity() {
    let db = setup_test_db().await;
    let user_id = create_user(&db).await;

    // Initial balance is 1000
    let initial_balance = Decimal::new(1000, 0);
    let wallet_id = create_wallet(&db, &user_id, initial_balance).await;

    // Run a flurry of transactions, updates, and deletes
    // 1. Create OUT 100
    let t1 = create_transaction(
        &db,
        &user_id,
        Decimal::new(100, 0),
        TransactionDirection::Out,
        Utc::now().into(),
        TransactionSource::Manual,
        None,
        None,
        Some(wallet_id.clone()),
        None,
        None,
        None,
    )
    .await
    .unwrap();

    // 2. Create IN 500
    let t2 = create_transaction(
        &db,
        &user_id,
        Decimal::new(500, 0),
        TransactionDirection::In,
        Utc::now().into(),
        TransactionSource::Manual,
        None,
        None,
        None,
        Some(wallet_id.clone()),
        None,
        None,
    )
    .await
    .unwrap();

    // 3. Update t1 to OUT 150
    update_transaction(
        &db,
        &user_id,
        &t1.id,
        Some(Decimal::new(150, 0)),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap();

    // 4. Delete t2
    delete_transaction(&db, &user_id, &t2.id).await.unwrap();

    // 5. Create OUT 50
    create_transaction(
        &db,
        &user_id,
        Decimal::new(50, 0),
        TransactionDirection::Out,
        Utc::now().into(),
        TransactionSource::Manual,
        None,
        None,
        Some(wallet_id.clone()),
        None,
        None,
        None,
    )
    .await
    .unwrap();

    // Parity Check Function
    let wallet = wallets::Entity::find_by_id(wallet_id.clone()).one(&db).await.unwrap().unwrap();

    // Sum all NON-DELETED transactions for this wallet
    use sea_orm::QuerySelect;

    // Income sum (Destination wallet)
    #[derive(sea_orm::FromQueryResult)]
    struct SumResult { total: Option<Decimal> }

    let in_sum: Option<Decimal> = transactions::Entity::find()
        .filter(transactions::Column::DestinationWalletId.eq(wallet_id.clone()))
        .filter(transactions::Column::DeletedAt.is_null())
        .select_only()
        .column_as(transactions::Column::Amount.sum(), "total")
        .into_model::<SumResult>()
        .one(&db)
        .await
        .unwrap()
        .and_then(|r| r.total);

    // Expense sum (Source wallet)
    let out_sum: Option<Decimal> = transactions::Entity::find()
        .filter(transactions::Column::SourceWalletId.eq(wallet_id.clone()))
        .filter(transactions::Column::DeletedAt.is_null())
        .select_only()
        .column_as(transactions::Column::Amount.sum(), "total")
        .into_model::<SumResult>()
        .one(&db)
        .await
        .unwrap()
        .and_then(|r| r.total);

    // The calculated ledger balance should be: Initial Balance + Incomes - Expenses
    let calculated_balance = initial_balance
        + in_sum.unwrap_or(Decimal::ZERO)
        - out_sum.unwrap_or(Decimal::ZERO);

    // Assert Wallet Parity
    assert_eq!(
        wallet.balance,
        calculated_balance,
        "Wallet balance ({}) does not match Ledger sum ({})",
        wallet.balance,
        calculated_balance
    );
}
