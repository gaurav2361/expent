use db::entities::*;
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, Schema, Statement};

pub async fn setup_test_db() -> DatabaseConnection {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    db.execute(Statement::from_string(
        DbBackend::Sqlite,
        "PRAGMA foreign_keys = OFF;",
    ))
    .await
    .unwrap();

    let builder = db.get_database_backend();
    let schema = Schema::new(builder);

    let tables = vec![
        builder.build(&schema.create_table_from_entity(users::Entity)),
        builder.build(&schema.create_table_from_entity(groups::Entity)),
        builder.build(&schema.create_table_from_entity(wallets::Entity)),
        builder.build(&schema.create_table_from_entity(categories::Entity)),
        builder.build(&schema.create_table_from_entity(contacts::Entity)),
        builder.build(&schema.create_table_from_entity(ledger_tabs::Entity)),
        builder.build(&schema.create_table_from_entity(transactions::Entity)),
        builder.build(&schema.create_table_from_entity(transaction_edits::Entity)),
        builder.build(&schema.create_table_from_entity(txn_parties::Entity)),
        builder.build(&schema.create_table_from_entity(subscriptions::Entity)),
        builder.build(&schema.create_table_from_entity(bank_statement_rows::Entity)),
        builder.build(&schema.create_table_from_entity(p2p_requests::Entity)),
    ];

    for stmt in tables {
        db.execute(stmt).await.unwrap();
    }

    db
}
