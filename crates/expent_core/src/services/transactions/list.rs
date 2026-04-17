use db::entities;
use db::{AppError, PaginatedTransactions, TransactionWithDetail};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, JoinType, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, RelationTrait,
};

pub async fn list_transactions(
    db: &DatabaseConnection,
    user_id: &str,
    limit: Option<u64>,
    offset: Option<u64>,
) -> Result<PaginatedTransactions, AppError> {
    let base_query = entities::transactions::Entity::find()
        .filter(entities::transactions::Column::UserId.eq(user_id))
        .filter(entities::transactions::Column::DeletedAt.is_null());

    // Get total count
    let total_count = base_query.clone().count(db).await?;

    // Refined query with joins
    let mut query = base_query
        .order_by_desc(entities::transactions::Column::Date)
        .column_as(entities::categories::Column::Name, "category_name")
        .join_rev(
            JoinType::LeftJoin,
            entities::categories::Entity::belongs_to(entities::transactions::Entity)
                .from(entities::categories::Column::Id)
                .to(entities::transactions::Column::CategoryId)
                .into(),
        );

    if let Some(l) = limit {
        query = query.limit(l);
    }
    if let Some(o) = offset {
        query = query.offset(o);
    }

    // Since we need to join wallets twice and contacts via a party table,
    // and SeaORM's Join is a bit verbose for double joins on same table,
    // we will fetch the raw models and then do a second pass for the names
    // using a more efficient "batch" approach if possible, or just optimized joins if simple.

    // Actually, for simplicity and correctness with SeaORM, let's use the joins for everything.
    // However, SeaORM's Model doesn't have fields for joined columns unless we use SelectTwoMany etc.

    // Let's stick to a slightly more manual but still efficient approach:
    // fetch main models, then batch fetch related names.

    let results = query.all(db).await?;

    if results.is_empty() {
        return Ok(PaginatedTransactions {
            items: Vec::new(),
            total_count,
        });
    }

    let mut final_results = Vec::new();

    // Extract IDs for batch fetching
    let wallet_ids: std::collections::HashSet<String> = results
        .iter()
        .flat_map(|t| vec![t.source_wallet_id.clone(), t.destination_wallet_id.clone()])
        .flatten()
        .collect();

    let txn_ids: Vec<String> = results.iter().map(|t| t.id.clone()).collect();

    // Batch fetch wallets
    let wallets_map: std::collections::HashMap<String, String> = if !wallet_ids.is_empty() {
        entities::wallets::Entity::find()
            .filter(entities::wallets::Column::Id.is_in(wallet_ids))
            .all(db)
            .await?
            .into_iter()
            .map(|w| (w.id, w.name))
            .collect()
    } else {
        std::collections::HashMap::new()
    };

    // Batch fetch categories (already handled in query? No, SeaORM models don't auto-fill extra fields)
    let category_ids: std::collections::HashSet<String> = results
        .iter()
        .filter_map(|t| t.category_id.clone())
        .collect();

    let categories_map: std::collections::HashMap<String, String> = if !category_ids.is_empty() {
        entities::categories::Entity::find()
            .filter(entities::categories::Column::Id.is_in(category_ids))
            .all(db)
            .await?
            .into_iter()
            .map(|c| (c.id, c.name))
            .collect()
    } else {
        std::collections::HashMap::new()
    };

    // Batch fetch contacts via txn_parties
    let parties = entities::txn_parties::Entity::find()
        .filter(entities::txn_parties::Column::TransactionId.is_in(txn_ids))
        .filter(entities::txn_parties::Column::Role.eq("COUNTERPARTY"))
        .all(db)
        .await?;

    let contact_ids: std::collections::HashSet<String> = parties
        .iter()
        .filter_map(|p| p.contact_id.clone())
        .collect();

    let contacts_map: std::collections::HashMap<String, String> = if !contact_ids.is_empty() {
        entities::contacts::Entity::find()
            .filter(entities::contacts::Column::Id.is_in(contact_ids))
            .all(db)
            .await?
            .into_iter()
            .map(|c| (c.id, c.name))
            .collect()
    } else {
        std::collections::HashMap::new()
    };

    let txn_to_contact: std::collections::HashMap<String, (String, String)> = parties
        .into_iter()
        .filter_map(|p| {
            let c_id = p.contact_id?;
            let c_name = contacts_map.get(&c_id)?;
            Some((p.transaction_id, (c_id, c_name.clone())))
        })
        .collect();

    for txn in results {
        let source_wallet_name = txn
            .source_wallet_id
            .as_ref()
            .and_then(|id| wallets_map.get(id))
            .cloned();
        let destination_wallet_name = txn
            .destination_wallet_id
            .as_ref()
            .and_then(|id| wallets_map.get(id))
            .cloned();
        let category_name = txn
            .category_id
            .as_ref()
            .and_then(|id| categories_map.get(id))
            .cloned();

        let (contact_id, contact_name) = txn_to_contact
            .get(&txn.id)
            .map(|(id, name)| (Some(id.clone()), Some(name.clone())))
            .unwrap_or((None, None));

        final_results.push(TransactionWithDetail {
            transaction: txn,
            source_wallet_name,
            destination_wallet_name,
            contact_name,
            contact_id,
            category_name,
        });
    }

    Ok(PaginatedTransactions {
        items: final_results,
        total_count,
    })
}
