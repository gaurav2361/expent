use db::AppError;
use db::entities;
use sea_orm::{DatabaseConnection, QueryFilter, QuerySelect, EntityTrait, JoinType, ColumnTrait, RelationTrait, ColumnTypeTrait, ActiveModelBehavior, Iterable, Iden, ActiveEnum, ActiveModelTrait};
use serde::{Deserialize, Serialize};
use strsim::jaro_winkler;

#[derive(Serialize, Deserialize, Debug)]
pub struct MergeSuggestion {
    pub contacts: Vec<entities::contacts::Model>,
    pub reason: String,
}

pub async fn get_merge_suggestions(
    db: &DatabaseConnection,
    user_id: &str,
) -> Result<Vec<MergeSuggestion>, AppError> {
    // Fetch all user contacts
    let contacts = entities::contacts::Entity::find()
        .join(
            JoinType::InnerJoin,
            entities::contacts::Relation::ContactLinks.def(),
        )
        .filter(entities::contact_links::Column::UserId.eq(user_id))
        .all(db)
        .await?;

    if contacts.len() < 2 {
        return Ok(vec![]);
    }

    // Fetch identifiers for all these contacts
    let contact_ids: Vec<String> = contacts.iter().map(|c| c.id.clone()).collect();
    let identifiers = entities::contact_identifiers::Entity::find()
        .filter(entities::contact_identifiers::Column::ContactId.is_in(contact_ids))
        .all(db)
        .await?;

    let mut suggestions: Vec<MergeSuggestion> = Vec::new();
    let mut processed_pairs = std::collections::HashSet::new();

    for (i, c1) in contacts.iter().enumerate() {
        for c2 in contacts.iter().skip(i + 1) {
            let pair_id = format!("{}-{}", c1.id.clone().min(c2.id.clone()), c1.id.clone().max(c2.id.clone()));
            if processed_pairs.contains(&pair_id) {
                continue;
            }

            let mut match_reason = None;

            // 1. Check exact phone match
            if let (Some(p1), Some(p2)) = (&c1.phone, &c2.phone)
                && !p1.trim().is_empty() && p1 == p2 {
                    match_reason = Some("Same phone number".to_string());
                }

            // 2. Check identifier overlap (UPI, Bank Acc)
            if match_reason.is_none() {
                let id1s: Vec<_> = identifiers.iter().filter(|id| id.contact_id == c1.id).collect();
                let id2s: Vec<_> = identifiers.iter().filter(|id| id.contact_id == c2.id).collect();

                'outer: for id1 in &id1s {
                    for id2 in &id2s {
                        if id1.r#type == id2.r#type && id1.value == id2.value {
                            match_reason = Some(format!("Shared {} identifier", id1.r#type));
                            break 'outer;
                        }
                    }
                }
            }

            // 3. Check fuzzy name match (jaro winkler > 0.85 is usually a good match)
            if match_reason.is_none() {
                let name1 = c1.name.to_lowercase();
                let name2 = c2.name.to_lowercase();
                if jaro_winkler(&name1, &name2) > 0.88 {
                    match_reason = Some("Similar name".to_string());
                }
            }

            if let Some(reason) = match_reason {
                suggestions.push(MergeSuggestion {
                    contacts: vec![c1.clone(), c2.clone()],
                    reason,
                });
                processed_pairs.insert(pair_id);
            }
        }
    }

    Ok(suggestions)
}
