use db::entities;
use db::{AppError, TransactionWithDetail};
use sea_orm::{
    ActiveEnum, ActiveModelBehavior, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder, QuerySelect,
};

pub async fn list_transactions(
    db: &DatabaseConnection,
    user_id: &str,
    limit: Option<u64>,
    offset: Option<u64>,
) -> Result<Vec<TransactionWithDetail>, AppError> {
    let mut query = entities::transactions::Entity::find()
        .filter(entities::transactions::Column::UserId.eq(user_id))
        .filter(entities::transactions::Column::DeletedAt.is_null())
        .order_by_desc(entities::transactions::Column::Date);

    if let Some(l) = limit {
        query = query.limit(l);
    }
    if let Some(o) = offset {
        query = query.offset(o);
    }

    let results = query.all(db).await?;

    let mut final_results = Vec::new();
    for txn in results {
        let mut source_name = None;
        let mut dest_name = None;

        if let Some(sw_id) = &txn.source_wallet_id
            && let Some(w) = entities::wallets::Entity::find_by_id(sw_id.clone())
                .one(db)
                .await?
        {
            source_name = Some(w.name);
        }

        if let Some(dw_id) = &txn.destination_wallet_id
            && let Some(w) = entities::wallets::Entity::find_by_id(dw_id.clone())
                .one(db)
                .await?
        {
            dest_name = Some(w.name);
        }

        let mut contact_name = None;
        let mut contact_id = None;

        if let Some(party) = entities::txn_parties::Entity::find()
            .filter(entities::txn_parties::Column::TransactionId.eq(txn.id.clone()))
            .filter(entities::txn_parties::Column::Role.eq("COUNTERPARTY"))
            .one(db)
            .await?
            && let Some(c_id) = party.contact_id
            && let Some(c) = entities::contacts::Entity::find_by_id(c_id.clone())
                .one(db)
                .await?
        {
            contact_name = Some(c.name);
            contact_id = Some(c_id);
        }

        final_results.push(TransactionWithDetail {
            transaction: txn,
            source_wallet_name: source_name,
            destination_wallet_name: dest_name,
            contact_name,
            contact_id,
        });
    }

    Ok(final_results)
}
