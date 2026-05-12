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
        ..Default::default()
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
pub async fn adjust_balance<C>(
    db: &C,
    wallet_id: &str,
    amount: Decimal,
    allow_negative: bool,
) -> Result<(), AppError>
where
    C: ConnectionTrait,
{
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

pub struct ResolveWalletParams {
    pub bank_name: String,
    pub account_number: Option<String>,
}

pub async fn resolve_wallet<C>(
    db: &C,
    user_id: &str,
    params: ResolveWalletParams,
) -> Result<entities::wallets::Model, AppError>
where
    C: ConnectionTrait,
{
    use sea_orm::Condition;

    // 1. Try to find by account number if provided
    if let Some(ref acc_num) = params.account_number {
        let existing = entities::wallets::Entity::find()
            .filter(entities::wallets::Column::UserId.eq(user_id))
            .filter(entities::wallets::Column::AccountNumber.eq(acc_num))
            .one(db)
            .await?;

        if let Some(wallet) = existing {
            return Ok(wallet);
        }
    }

    // 2. Try to find by bank name (fuzzy/partial match)
    let existing = entities::wallets::Entity::find()
        .filter(entities::wallets::Column::UserId.eq(user_id))
        .filter(
            Condition::any()
                .add(entities::wallets::Column::Name.ilike(format!("%{}%", params.bank_name)))
                .add(entities::wallets::Column::BankName.ilike(format!("%{}%", params.bank_name))),
        )
        .one(db)
        .await?;

    if let Some(mut wallet) = existing {
        // Update account number if it was missing
        if wallet.account_number.is_none() && params.account_number.is_some() {
            let mut active: entities::wallets::ActiveModel = wallet.into();
            active.account_number = Set(params.account_number);
            active.updated_at = Set(Utc::now().into());
            wallet = active.update(db).await?;
        }
        return Ok(wallet);
    }

    // 3. Create new wallet if not found
    let wallet = entities::wallets::ActiveModel {
        id: Set(uuid::Uuid::now_v7().to_string()),
        user_id: Set(user_id.to_string()),
        name: Set(params.bank_name.clone()),
        r#type: Set(WalletType::Bank),
        balance: Set(Decimal::ZERO), // Default to zero, user can adjust
        bank_name: Set(Some(params.bank_name)),
        account_number: Set(params.account_number),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
    };

    wallet.insert(db).await.map_err(AppError::from)
}
