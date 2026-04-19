mod helpers;

use chrono::Utc;
use ::db::entities::enums::{TransactionDirection, TransactionSource, GroupRole};
use ::db::entities::{self, transactions, wallets};
use expent_core::services::transactions::{create_transaction, delete_transaction, update_transaction};
use helpers::db::setup_test_db;
use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
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
async fn test_create_transaction_success_expense() {
    let db = setup_test_db().await;
    let user_id = create_user(&db).await;
    let wallet_id = create_wallet(&db, &user_id, Decimal::new(1000, 0)).await;

    let result = create_transaction(
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
    .await;

    assert!(result.is_ok());
    let txn = result.unwrap();
    assert_eq!(txn.amount, Decimal::new(100, 0));

    let wallet = wallets::Entity::find_by_id(wallet_id).one(&db).await.unwrap().unwrap();
    assert_eq!(wallet.balance, Decimal::new(900, 0));
}

#[tokio::test]
async fn test_create_transaction_success_income() {
    let db = setup_test_db().await;
    let user_id = create_user(&db).await;
    let wallet_id = create_wallet(&db, &user_id, Decimal::new(1000, 0)).await;

    let result = create_transaction(
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
    .await;

    assert!(result.is_ok());

    let wallet = wallets::Entity::find_by_id(wallet_id).one(&db).await.unwrap().unwrap();
    assert_eq!(wallet.balance, Decimal::new(1500, 0));
}

#[tokio::test]
async fn test_create_transaction_validation_error() {
    let db = setup_test_db().await;
    let user_id = create_user(&db).await;
    let wallet_id = create_wallet(&db, &user_id, Decimal::new(1000, 0)).await;

    let res1 = create_transaction(
        &db,
        &user_id,
        Decimal::new(0, 0),
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
    .await;

    assert!(res1.is_err());
    let err = res1.as_ref().err().unwrap();
    assert_eq!(err.to_string(), "Validation error: Amount must be positive");

    let res2 = create_transaction(
        &db,
        &user_id,
        Decimal::new(-10, 0),
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
    .await;

    assert!(res2.is_err());
    let err2 = res2.as_ref().err().unwrap();
    assert_eq!(err2.to_string(), "Validation error: Amount must be positive");
}

#[tokio::test]
async fn test_create_transaction_contact_linking() {
    let db = setup_test_db().await;
    let user_id = create_user(&db).await;
    let wallet_id = create_wallet(&db, &user_id, Decimal::new(1000, 0)).await;

    let contact_id = Uuid::now_v7().to_string();
    entities::contacts::ActiveModel {
        id: Set(contact_id.clone()),
        name: Set("Test Contact".to_string()),
        is_pinned: Set(false),
        ..Default::default()
    }
    .insert(&db)
    .await
    .unwrap();

    let result = create_transaction(
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
        Some(contact_id.clone()),
        None,
    )
    .await;

    assert!(result.is_ok());
    let txn = result.unwrap();

    use sea_orm::ColumnTrait;
    use sea_orm::QueryFilter;
    let parties = entities::txn_parties::Entity::find()
        .filter(entities::txn_parties::Column::TransactionId.eq(txn.id))
        .all(&db)
        .await
        .unwrap();

    assert_eq!(parties.len(), 1);
    assert_eq!(parties[0].contact_id, Some(contact_id));
    assert_eq!(parties[0].role, "COUNTERPARTY");
}

#[tokio::test]
async fn test_update_transaction_amount_delta() {
    let db = setup_test_db().await;
    let user_id = create_user(&db).await;
    let wallet_id = create_wallet(&db, &user_id, Decimal::new(1000, 0)).await;

    let txn = create_transaction(
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

    let wallet = wallets::Entity::find_by_id(wallet_id.clone()).one(&db).await.unwrap().unwrap();
    assert_eq!(wallet.balance, Decimal::new(900, 0));

    let _ = update_transaction(
        &db,
        &user_id,
        &txn.id,
        Some(Decimal::new(120, 0)),
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

    let wallet = wallets::Entity::find_by_id(wallet_id).one(&db).await.unwrap().unwrap();
    assert_eq!(wallet.balance, Decimal::new(880, 0));

    use sea_orm::{QueryFilter, ColumnTrait};
    let edits = entities::transaction_edits::Entity::find()
        .filter(entities::transaction_edits::Column::TransactionId.eq(txn.id))
        .all(&db)
        .await
        .unwrap();
    assert_eq!(edits.len(), 1);
    assert_eq!(edits[0].old_amount, Decimal::new(100, 0));
    assert_eq!(edits[0].new_amount, Decimal::new(120, 0));
}

#[tokio::test]
async fn test_update_transaction_wallet_swap() {
    let db = setup_test_db().await;
    let user_id = create_user(&db).await;
    let wallet_a = create_wallet(&db, &user_id, Decimal::new(1000, 0)).await;
    let wallet_b = create_wallet(&db, &user_id, Decimal::new(500, 0)).await;

    let txn = create_transaction(
        &db,
        &user_id,
        Decimal::new(100, 0),
        TransactionDirection::Out,
        Utc::now().into(),
        TransactionSource::Manual,
        None,
        None,
        Some(wallet_a.clone()),
        None,
        None,
        None,
    )
    .await
    .unwrap();

    let wa = wallets::Entity::find_by_id(wallet_a.clone()).one(&db).await.unwrap().unwrap();
    assert_eq!(wa.balance, Decimal::new(900, 0));

    let _ = update_transaction(
        &db,
        &user_id,
        &txn.id,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(wallet_b.clone()),
        None,
        None,
    )
    .await
    .unwrap();

    let wa = wallets::Entity::find_by_id(wallet_a).one(&db).await.unwrap().unwrap();
    assert_eq!(wa.balance, Decimal::new(1000, 0)); // Reverted

    let wb = wallets::Entity::find_by_id(wallet_b).one(&db).await.unwrap().unwrap();
    assert_eq!(wb.balance, Decimal::new(400, 0)); // Applied
}

#[tokio::test]
async fn test_delete_transaction_reversal() {
    let db = setup_test_db().await;
    let user_id = create_user(&db).await;
    let wallet_id = create_wallet(&db, &user_id, Decimal::new(1000, 0)).await;

    let txn = create_transaction(
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

    let _ = delete_transaction(&db, &user_id, &txn.id).await.unwrap();

    let wallet = wallets::Entity::find_by_id(wallet_id.clone()).one(&db).await.unwrap().unwrap();
    assert_eq!(wallet.balance, Decimal::new(1000, 0));

    let deleted_txn = transactions::Entity::find_by_id(txn.id.clone()).one(&db).await.unwrap().unwrap();
    assert!(deleted_txn.deleted_at.is_some());

    let _ = delete_transaction(&db, &user_id, &txn.id).await.unwrap();
    let wallet = wallets::Entity::find_by_id(wallet_id).one(&db).await.unwrap().unwrap();
    assert_eq!(wallet.balance, Decimal::new(1000, 0));
}

#[tokio::test]
async fn test_update_transaction_direction_flip() {
    let db = setup_test_db().await;
    let user_id = create_user(&db).await;
    let wallet_id = create_wallet(&db, &user_id, Decimal::new(1000, 0)).await;

    let txn = create_transaction(
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

    let wallet = wallets::Entity::find_by_id(wallet_id.clone()).one(&db).await.unwrap().unwrap();
    assert_eq!(wallet.balance, Decimal::new(900, 0));
}
