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

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{DatabaseBackend, MockDatabase};

    #[tokio::test]
    async fn test_list_wallets() {
        let user_id = "user_123";
        let mock_wallets = vec![
            entities::wallets::Model {
                id: "wallet_1".to_string(),
                user_id: user_id.to_string(),
                name: "Cash".to_string(),
                r#type: WalletType::Cash,
                balance: Decimal::from(100),
                created_at: Utc::now().into(),
                updated_at: Utc::now().into(),
            },
            entities::wallets::Model {
                id: "wallet_2".to_string(),
                user_id: user_id.to_string(),
                name: "Bank".to_string(),
                r#type: WalletType::Bank,
                balance: Decimal::from(5000),
                created_at: Utc::now().into(),
                updated_at: Utc::now().into(),
            },
        ];

        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(vec![mock_wallets.clone()])
            .into_connection();

        let result = list_wallets(&db, user_id).await.unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id, "wallet_1");
        assert_eq!(result[1].id, "wallet_2");
        assert_eq!(result[0].user_id, user_id);
        assert_eq!(result[1].user_id, user_id);

        // Verify the query was filtered by user_id
        let log = db.into_transaction_log();
        assert_eq!(log.len(), 1);
        let query = &log[0];
        if let sea_orm::Transaction::Query(q) = query {
            assert!(q.sql.contains("\"user_id\" = $1"));
            assert_eq!(q.values, vec![user_id.into()]);
        } else {
            panic!("Expected a query");
        }
    }
}
