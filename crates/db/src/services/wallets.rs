use crate::entities;
use crate::entities::enums::WalletType;
use chrono::Utc;
use rust_decimal::Decimal;
use sea_orm::*;

pub async fn list_wallets(
    db: &DatabaseConnection,
    user_id: &str,
) -> Result<Vec<entities::wallets::Model>, DbErr> {
    entities::wallets::Entity::find()
        .filter(entities::wallets::Column::UserId.eq(user_id))
        .all(db)
        .await
}

pub async fn create_wallet(
    db: &DatabaseConnection,
    user_id: &str,
    name: &str,
    wallet_type: WalletType,
    initial_balance: Decimal,
) -> Result<entities::wallets::Model, DbErr> {
    let wallet = entities::wallets::ActiveModel {
        id: Set(uuid::Uuid::now_v7().to_string()),
        user_id: Set(user_id.to_string()),
        name: Set(name.to_string()),
        r#type: Set(wallet_type.to_string()),
        balance: Set(initial_balance),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
    };

    wallet.insert(db).await
}

pub async fn update_wallet(
    db: &DatabaseConnection,
    user_id: &str,
    wallet_id: &str,
    name: Option<String>,
    balance: Option<Decimal>,
) -> Result<entities::wallets::Model, DbErr> {
    let mut wallet: entities::wallets::ActiveModel = entities::wallets::Entity::find()
        .filter(entities::wallets::Column::UserId.eq(user_id))
        .filter(entities::wallets::Column::Id.eq(wallet_id))
        .one(db)
        .await?
        .ok_or(DbErr::Custom("Wallet not found".to_string()))?
        .into();

    if let Some(n) = name {
        wallet.name = Set(n);
    }
    if let Some(b) = balance {
        wallet.balance = Set(b);
    }
    wallet.updated_at = Set(Utc::now().into());

    wallet.update(db).await
}

pub async fn adjust_balance<C>(
    db: &C,
    wallet_id: &str,
    amount: Decimal,
) -> Result<(), DbErr>
where
    C: ConnectionTrait,
{
    let wallet = entities::wallets::Entity::find_by_id(wallet_id.to_string())
        .one(db)
        .await?
        .ok_or(DbErr::Custom("Wallet not found".to_string()))?;

    let mut wallet: entities::wallets::ActiveModel = wallet.into();
    wallet.balance = Set(wallet.balance.as_ref() + amount);
    wallet.updated_at = Set(Utc::now().into());

    wallet.update(db).await?;
    Ok(())
}

pub async fn delete_wallet(
    db: &DatabaseConnection,
    user_id: &str,
    wallet_id: &str,
) -> Result<u64, DbErr> {
    let result = entities::wallets::Entity::delete_many()
        .filter(entities::wallets::Column::UserId.eq(user_id))
        .filter(entities::wallets::Column::Id.eq(wallet_id))
        .exec(db)
        .await?;

    Ok(result.rows_affected)
}
