use chrono::Utc;
use db::AppError;
use db::entities;
use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, ConnectionTrait, EntityTrait, Iden, Set};

pub async fn adjust_balance<C>(db: &C, wallet_id: &str, amount: Decimal) -> Result<(), AppError>
where
    C: ConnectionTrait,
{
    let wallet = entities::wallets::Entity::find_by_id(wallet_id.to_string())
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Wallet not found"))?;

    let mut wallet: entities::wallets::ActiveModel = wallet.into();
    wallet.balance = Set(wallet.balance.as_ref() + amount);
    wallet.updated_at = Set(Utc::now().into());

    wallet.update(db).await?;
    Ok(())
}
