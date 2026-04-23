use chrono::Utc;
use db::AppError;
use db::entities;
use db::entities::enums::WalletType;
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter,
    Set,
};

pub async fn create_wallet(
    db: &DatabaseConnection,
    user_id: &str,
    name: &str,
    wallet_type: WalletType,
    initial_balance: Decimal,
) -> Result<entities::wallets::Model, AppError> {
    let wallet = entities::wallets::ActiveModel {
        id: Set(uuid::Uuid::now_v7().to_string()),
        user_id: Set(user_id.to_string()),
        name: Set(name.to_string()),
        r#type: Set(wallet_type),
        balance: Set(initial_balance),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
    };

    wallet.insert(db).await.map_err(AppError::from)
}

pub async fn list_wallets(
    db: &DatabaseConnection,
    user_id: &str,
) -> Result<Vec<entities::wallets::Model>, AppError> {
    entities::wallets::Entity::find()
        .filter(entities::wallets::Column::UserId.eq(user_id))
        .all(db)
        .await
        .map_err(AppError::from)
}

pub async fn delete_wallet(
    db: &DatabaseConnection,
    user_id: &str,
    wallet_id: &str,
) -> Result<u64, AppError> {
    let result = entities::wallets::Entity::delete_many()
        .filter(entities::wallets::Column::UserId.eq(user_id))
        .filter(entities::wallets::Column::Id.eq(wallet_id))
        .exec(db)
        .await?;

    Ok(result.rows_affected)
}

pub async fn update_wallet(
    db: &DatabaseConnection,
    user_id: &str,
    wallet_id: &str,
    name: Option<String>,
    balance: Option<Decimal>,
) -> Result<entities::wallets::Model, AppError> {
    let mut wallet: entities::wallets::ActiveModel = entities::wallets::Entity::find()
        .filter(entities::wallets::Column::UserId.eq(user_id))
        .filter(entities::wallets::Column::Id.eq(wallet_id))
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Wallet not found"))?
        .into();

    if let Some(n) = name {
        wallet.name = Set(n);
    }
    if let Some(b) = balance {
        wallet.balance = Set(b);
    }
    wallet.updated_at = Set(Utc::now().into());

    wallet.update(db).await.map_err(AppError::from)
}

pub async fn get_balance(db: &DatabaseConnection, wallet_id: &str) -> Result<Decimal, AppError> {
    let wallet = entities::wallets::Entity::find_by_id(wallet_id.to_string())
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Wallet not found"))?;

    Ok(wallet.balance)
}

/// Adjusts the balance of a wallet atomically using database-level expressions.
/// This prevents race conditions where multiple updates happen simultaneously.
pub async fn adjust_balance<C>(
    db: &C,
    wallet_id: &str,
    amount: Decimal,
    allow_negative: bool,
) -> Result<(), AppError>
where
    C: ConnectionTrait,
{
    // If we don't allow negative balance and the amount is negative, we need to check current balance first.
    // Note: While this check isn't perfectly atomic with the update below unless we use a complex SQL constraint,
    // it's a good first-level defense. For true atomicity, a database constraint CHECK (balance >= 0) is preferred.
    if !allow_negative && amount.is_sign_negative() {
        let current_balance = entities::wallets::Entity::find_by_id(wallet_id.to_string())
            .one(db)
            .await?
            .ok_or_else(|| AppError::not_found("Wallet not found"))?
            .balance;

        if current_balance + amount < Decimal::ZERO {
            return Err(AppError::validation("Insufficient funds in wallet"));
        }
    }

    // Use Sea-ORM's update_many with an expression for atomic update
    let result = entities::wallets::Entity::update_many()
        .col_expr(
            entities::wallets::Column::Balance,
            sea_orm::sea_query::Expr::col(entities::wallets::Column::Balance).add(amount),
        )
        .col_expr(
            entities::wallets::Column::UpdatedAt,
            sea_orm::sea_query::Expr::value(Utc::now()),
        )
        .filter(entities::wallets::Column::Id.eq(wallet_id))
        .exec(db)
        .await?;

    if result.rows_affected == 0 {
        return Err(AppError::not_found("Wallet not found"));
    }

    Ok(())
}

pub async fn get_wallet(
    db: &DatabaseConnection,
    user_id: &str,
    wallet_id: &str,
) -> Result<entities::wallets::Model, AppError> {
    entities::wallets::Entity::find()
        .filter(entities::wallets::Column::UserId.eq(user_id))
        .filter(entities::wallets::Column::Id.eq(wallet_id))
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Wallet not found"))
}
