mod helpers;

use chrono::{Duration, Utc};
use ::db::entities::enums::{TransactionDirection, TransactionSource, GroupRole};
use ::db::entities::{self};
use expent_core::services::transactions::create_transaction;
use expent_core::services::subscriptions::detect_subscriptions;
use helpers::db::setup_test_db;
use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
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
async fn test_subscription_detection_exact_interval() {
    let db = setup_test_db().await;
    let user_id = create_user(&db).await;
    let wallet_id = create_wallet(&db, &user_id, Decimal::new(1000, 0)).await;

    let now = Utc::now();
    let t1 = now - Duration::days(60);
    let t2 = now - Duration::days(30);
    let t3 = now;

    for &t in &[t1, t2, t3] {
        create_transaction(
            &db,
            &user_id,
            Decimal::new(199, 0), // exact amount
            TransactionDirection::Out,
            t.into(),
            TransactionSource::Manual,
            Some("Netflix".to_string()), // same name
            None,
            Some(wallet_id.clone()),
            None,
            None,
            None,
        )
        .await
        .unwrap();
    }

    let subs = detect_subscriptions(&db, &user_id).await.unwrap();

    assert_eq!(subs.len(), 1);
    let sub = &subs[0];
    assert_eq!(sub.name, "Netflix");
    assert_eq!(sub.amount, Decimal::new(199, 0));
    assert_eq!(sub.cycle, "Monthly");
}

#[tokio::test]
async fn test_subscription_detection_fuzzy_drift() {
    let db = setup_test_db().await;
    let user_id = create_user(&db).await;
    let wallet_id = create_wallet(&db, &user_id, Decimal::new(1000, 0)).await;

    let now = Utc::now();
    // Drift ranges are: 27..=33 days for Monthly
    let t1 = now - Duration::days(61);
    let t2 = now - Duration::days(30);
    let t3 = now;

    // Diff 1: 31 days. Diff 2: 30 days. Both fall in 27..=33

    for &t in &[t1, t2, t3] {
        create_transaction(
            &db,
            &user_id,
            Decimal::new(99, 0), // exact amount
            TransactionDirection::Out,
            t.into(),
            TransactionSource::Manual,
            Some("Spotify".to_string()), // same name
            None,
            Some(wallet_id.clone()),
            None,
            None,
            None,
        )
        .await
        .unwrap();
    }

    let subs = detect_subscriptions(&db, &user_id).await.unwrap();

    assert_eq!(subs.len(), 1);
    let sub = &subs[0];
    assert_eq!(sub.name, "Spotify");
    assert_eq!(sub.cycle, "Monthly");
}
