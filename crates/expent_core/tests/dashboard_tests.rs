mod helpers;

use chrono::{Duration, Utc, TimeZone};
use ::db::entities::enums::{TransactionDirection, TransactionSource, GroupRole};
use ::db::entities::{self};
use expent_core::services::transactions::{create_transaction, get_dashboard_summary};
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
async fn test_dashboard_total_balance() {
    let db = setup_test_db().await;
    let user_id = create_user(&db).await;

    // Create 3 wallets with 100, 200, and -50 balances
    create_wallet(&db, &user_id, Decimal::new(100, 0)).await;
    create_wallet(&db, &user_id, Decimal::new(200, 0)).await;
    create_wallet(&db, &user_id, Decimal::new(-50, 0)).await;

    let summary = get_dashboard_summary(&db, &user_id).await.unwrap();

    assert_eq!(summary.total_balance, Decimal::new(250, 0));
}

#[tokio::test]
async fn test_dashboard_monthly_spend_boundary() {
    let db = setup_test_db().await;
    let user_id = create_user(&db).await;
    let wallet_id = create_wallet(&db, &user_id, Decimal::new(1000, 0)).await;

    let now = Utc::now();

    // We need to calculate last month's last day precisely
    // A simpler way: take the 1st of current month, and subtract 1 minute.
    use chrono::Datelike;
    let first_day_this_month = Utc.with_ymd_and_hms(now.year(), now.month(), 1, 0, 0, 0).unwrap();
    let last_day_last_month = first_day_this_month - Duration::minutes(1);

    // Transaction on last day of last month (Out, 100)
    create_transaction(
        &db,
        &user_id,
        Decimal::new(100, 0),
        TransactionDirection::Out,
        last_day_last_month.into(),
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

    // Transaction on first day of this month (Out, 200)
    create_transaction(
        &db,
        &user_id,
        Decimal::new(200, 0),
        TransactionDirection::Out,
        (first_day_this_month + Duration::minutes(1)).into(),
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

    let summary = get_dashboard_summary(&db, &user_id).await.unwrap();

    // Only this month's transaction should be included in monthly_spend
    assert_eq!(summary.monthly_spend, Decimal::new(200, 0));
}

#[tokio::test]
async fn test_dashboard_weekly_trend_bucketing() {
    let db = setup_test_db().await;
    let user_id = create_user(&db).await;
    let wallet_id = create_wallet(&db, &user_id, Decimal::new(1000, 0)).await;

    // Create a transaction 1 day ago
    let yesterday = Utc::now() - Duration::days(1);
    create_transaction(
        &db,
        &user_id,
        Decimal::new(50, 0),
        TransactionDirection::Out,
        yesterday.into(),
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

    let summary = get_dashboard_summary(&db, &user_id).await.unwrap();

    // Ensure the 7 day trends are populated
    assert_eq!(summary.weekly_trends.len(), 7);

    // Find the trend for 'yesterday'
    let yesterday_name = yesterday.format("%a").to_string();
    let trend = summary.weekly_trends.iter().find(|t| t.month == yesterday_name);
    assert!(trend.is_some());
    // Since it could be the same day name but a week ago (0), we check > 0
    assert!(trend.unwrap().expense >= Decimal::new(50, 0));
}

#[tokio::test]
async fn test_dashboard_top_expenses() {
    let db = setup_test_db().await;
    let user_id = create_user(&db).await;
    let wallet_id = create_wallet(&db, &user_id, Decimal::new(1000, 0)).await;

    let contact1 = Uuid::now_v7().to_string();
    let contact2 = Uuid::now_v7().to_string();

    entities::contacts::ActiveModel {
        id: Set(contact1.clone()),
        name: Set("Contact A".to_string()),
        is_pinned: Set(false),
        ..Default::default()
    }
    .insert(&db)
    .await
    .unwrap();

    entities::contacts::ActiveModel {
        id: Set(contact2.clone()),
        name: Set("Contact B".to_string()),
        is_pinned: Set(false),
        ..Default::default()
    }
    .insert(&db)
    .await
    .unwrap();

    // Out 100 to Contact A
    create_transaction(
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
        Some(contact1.clone()),
        None,
    )
    .await
    .unwrap();

    // Out 300 to Contact B
    create_transaction(
        &db,
        &user_id,
        Decimal::new(300, 0),
        TransactionDirection::Out,
        Utc::now().into(),
        TransactionSource::Manual,
        None,
        None,
        Some(wallet_id.clone()),
        None,
        Some(contact2.clone()),
        None,
    )
    .await
    .unwrap();

    let summary = get_dashboard_summary(&db, &user_id).await.unwrap();

    // Top expenses should be sorted descending
    assert_eq!(summary.top_expenses.len(), 2);
    assert_eq!(summary.top_expenses[0].name, "Contact B");
    assert_eq!(summary.top_expenses[0].amount, Decimal::new(300, 0));
    assert_eq!(summary.top_expenses[1].name, "Contact A");
    assert_eq!(summary.top_expenses[1].amount, Decimal::new(100, 0));
}
