use db::AppError;
use db::entities;
use db::entities::enums::TransactionStatus;
use sea_orm::*;

/// Adjusts wallet balances based on a transaction's change.
/// This reverses the effect of the old transaction (if any) and applies the effect of the new transaction (if any).
pub async fn adjust_transaction_wallets<C>(
    db: &C,
    old_txn: Option<&entities::transactions::Model>,
    new_txn: Option<&entities::transactions::Model>,
) -> Result<(), AppError>
where
    C: ConnectionTrait,
{
    // 1. Reverse old effect IF it was active (not cancelled AND not deleted)
    if let Some(old) = old_txn {
        let old_is_active = old.status != TransactionStatus::Cancelled && old.deleted_at.is_none();
        if old_is_active {
            if let Some(sw_id) = &old.source_wallet_id {
                crate::services::wallets::adjust_balance(db, sw_id, old.amount).await?;
            }
            if let Some(dw_id) = &old.destination_wallet_id {
                crate::services::wallets::adjust_balance(db, dw_id, -old.amount).await?;
            }
        }
    }

    // 2. Apply new effect IF it is active (not cancelled AND not deleted)
    if let Some(new) = new_txn {
        let new_is_active = new.status != TransactionStatus::Cancelled && new.deleted_at.is_none();
        if new_is_active {
            if let Some(sw_id) = &new.source_wallet_id {
                crate::services::wallets::adjust_balance(db, sw_id, -new.amount).await?;
            }
            if let Some(dw_id) = &new.destination_wallet_id {
                crate::services::wallets::adjust_balance(db, dw_id, new.amount).await?;
            }
        }
    }

    Ok(())
}
