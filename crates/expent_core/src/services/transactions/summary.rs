use chrono::{Datelike, Duration, TimeZone, Timelike, Utc};
use db::entities;
use db::{AppError, DashboardSummary, MonthlyTrend, NamedAmount};
use rust_decimal::Decimal;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, FromQueryResult, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, RelationTrait,
};

pub async fn get_dashboard_summary(
    db: &DatabaseConnection,
    user_id: &str,
) -> Result<DashboardSummary, AppError> {
    // 0. Get user email for P2P requests
    let user = entities::users::Entity::find_by_id(user_id.to_string())
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("User not found"))?;

    // 1. Total Balance (sum of all wallet balances)
    #[derive(FromQueryResult)]
    struct TotalResult {
        total: Option<Decimal>,
    }

    let balance_res: Option<TotalResult> = entities::wallets::Entity::find()
        .filter(entities::wallets::Column::UserId.eq(user_id))
        .select_only()
        .column_as(entities::wallets::Column::Balance.sum(), "total")
        .into_model::<TotalResult>()
        .one(db)
        .await?;

    let total_balance = balance_res.and_then(|r| r.total).unwrap_or(Decimal::ZERO);

    // 2. Monthly Spend & Income
    let now = Utc::now();
    let start_of_month = Utc
        .with_ymd_and_hms(now.year(), now.month(), 1, 0, 0, 0)
        .expect("Valid start of month");

    #[derive(FromQueryResult)]
    struct SumResult {
        total: Option<Decimal>,
    }

    let monthly_spend: Option<Decimal> = entities::transactions::Entity::find()
        .filter(entities::transactions::Column::UserId.eq(user_id))
        .filter(entities::transactions::Column::Direction.eq("OUT"))
        .filter(entities::transactions::Column::Date.gte(start_of_month))
        .filter(entities::transactions::Column::DeletedAt.is_null())
        .select_only()
        .column_as(entities::transactions::Column::Amount.sum(), "total")
        .into_model::<SumResult>()
        .one(db)
        .await?
        .and_then(|r| r.total);

    let monthly_income: Option<Decimal> = entities::transactions::Entity::find()
        .filter(entities::transactions::Column::UserId.eq(user_id))
        .filter(entities::transactions::Column::Direction.eq("IN"))
        .filter(entities::transactions::Column::Date.gte(start_of_month))
        .filter(entities::transactions::Column::DeletedAt.is_null())
        .select_only()
        .column_as(entities::transactions::Column::Amount.sum(), "total")
        .into_model::<SumResult>()
        .one(db)
        .await?
        .and_then(|r| r.total);

    // 3. Pending P2P count
    let pending_p2p_count = entities::p2p_requests::Entity::find()
        .filter(entities::p2p_requests::Column::ReceiverEmail.eq(user.email))
        .filter(entities::p2p_requests::Column::Status.is_in(["PENDING", "GROUP_INVITE"]))
        .count(db)
        .await?;

    // 3b. Total Transactions
    let total_transactions = entities::transactions::Entity::find()
        .filter(entities::transactions::Column::UserId.eq(user_id))
        .filter(entities::transactions::Column::DeletedAt.is_null())
        .count(db)
        .await?;

    // 4. Monthly Trends (last 6 months)
    let mut monthly_trends = Vec::new();
    for i in (0..6).rev() {
        let month_offset = if now.month() as i32 - i as i32 <= 0 {
            (now.month() as i32 - i as i32 + 12) as u32
        } else {
            now.month() - i as u32
        };

        let year_offset = if now.month() as i32 - i as i32 <= 0 {
            now.year() - 1
        } else {
            now.year()
        };

        let start = Utc
            .with_ymd_and_hms(year_offset, month_offset, 1, 0, 0, 0)
            .expect("Valid month range");
        let end = if month_offset == 12 {
            Utc.with_ymd_and_hms(year_offset + 1, 1, 1, 0, 0, 0)
                .expect("Valid month range")
        } else {
            Utc.with_ymd_and_hms(year_offset, month_offset + 1, 1, 0, 0, 0)
                .expect("Valid month range")
        };

        let inc: Option<Decimal> = entities::transactions::Entity::find()
            .filter(entities::transactions::Column::UserId.eq(user_id))
            .filter(entities::transactions::Column::Direction.eq("IN"))
            .filter(entities::transactions::Column::Date.gte(start))
            .filter(entities::transactions::Column::Date.lt(end))
            .filter(entities::transactions::Column::DeletedAt.is_null())
            .select_only()
            .column_as(entities::transactions::Column::Amount.sum(), "total")
            .into_model::<SumResult>()
            .one(db)
            .await?
            .and_then(|r| r.total);

        let exp: Option<Decimal> = entities::transactions::Entity::find()
            .filter(entities::transactions::Column::UserId.eq(user_id))
            .filter(entities::transactions::Column::Direction.eq("OUT"))
            .filter(entities::transactions::Column::Date.gte(start))
            .filter(entities::transactions::Column::Date.lt(end))
            .filter(entities::transactions::Column::DeletedAt.is_null())
            .select_only()
            .column_as(entities::transactions::Column::Amount.sum(), "total")
            .into_model::<SumResult>()
            .one(db)
            .await?
            .and_then(|r| r.total);

        monthly_trends.push(MonthlyTrend {
            month: start.format("%b").to_string(),
            income: inc.unwrap_or(Decimal::ZERO),
            expense: exp.unwrap_or(Decimal::ZERO),
        });
    }

    // 4b. Weekly Trends (last 7 days)
    let mut weekly_trends = Vec::new();
    for i in (0..7).rev() {
        let start = (Utc::now() - Duration::days(i as i64))
            .with_hour(0)
            .expect("Valid hour")
            .with_minute(0)
            .expect("Valid minute")
            .with_second(0)
            .expect("Valid second");
        let end = start + Duration::days(1);

        let inc: Option<Decimal> = entities::transactions::Entity::find()
            .filter(entities::transactions::Column::UserId.eq(user_id))
            .filter(entities::transactions::Column::Direction.eq("IN"))
            .filter(entities::transactions::Column::Date.gte(start))
            .filter(entities::transactions::Column::Date.lt(end))
            .filter(entities::transactions::Column::DeletedAt.is_null())
            .select_only()
            .column_as(entities::transactions::Column::Amount.sum(), "total")
            .into_model::<SumResult>()
            .one(db)
            .await?
            .and_then(|r| r.total);

        let exp: Option<Decimal> = entities::transactions::Entity::find()
            .filter(entities::transactions::Column::UserId.eq(user_id))
            .filter(entities::transactions::Column::Direction.eq("OUT"))
            .filter(entities::transactions::Column::Date.gte(start))
            .filter(entities::transactions::Column::Date.lt(end))
            .filter(entities::transactions::Column::DeletedAt.is_null())
            .select_only()
            .column_as(entities::transactions::Column::Amount.sum(), "total")
            .into_model::<SumResult>()
            .one(db)
            .await?
            .and_then(|r| r.total);

        weekly_trends.push(MonthlyTrend {
            month: start.format("%a").to_string(),
            income: inc.unwrap_or(Decimal::ZERO),
            expense: exp.unwrap_or(Decimal::ZERO),
        });
    }

    // 5. Category Distribution
    #[derive(FromQueryResult)]
    struct CatDist {
        category_name: String,
        amount: Decimal,
    }

    let category_distribution = entities::transactions::Entity::find()
        .filter(entities::transactions::Column::UserId.eq(user_id))
        .filter(entities::transactions::Column::Direction.eq("OUT"))
        .filter(entities::transactions::Column::DeletedAt.is_null())
        .filter(entities::transactions::Column::CategoryId.is_not_null())
        .join(
            sea_orm::JoinType::InnerJoin,
            entities::transactions::Relation::Categories.def(),
        )
        .select_only()
        .column_as(entities::categories::Column::Name, "category_name")
        .column_as(entities::transactions::Column::Amount.sum(), "amount")
        .group_by(entities::categories::Column::Name)
        .into_model::<CatDist>()
        .all(db)
        .await?
        .into_iter()
        .map(|c| NamedAmount {
            name: c.category_name,
            amount: c.amount,
        })
        .collect();

    // 6. Top Sources (by contact)
    #[derive(FromQueryResult)]
    struct TopSource {
        contact_id: String,
        amount: Decimal,
    }

    async fn get_top_sources(
        db: &DatabaseConnection,
        user_id: &str,
        direction: &str,
    ) -> Result<Vec<NamedAmount>, AppError> {
        let results: Vec<TopSource> = entities::transactions::Entity::find()
            .filter(entities::transactions::Column::UserId.eq(user_id))
            .filter(entities::transactions::Column::Direction.eq(direction))
            .filter(entities::transactions::Column::DeletedAt.is_null())
            .join(
                sea_orm::JoinType::InnerJoin,
                entities::transactions::Relation::TxnParties.def(),
            )
            .filter(entities::txn_parties::Column::Role.eq("COUNTERPARTY"))
            .filter(entities::txn_parties::Column::ContactId.is_not_null())
            .select_only()
            .column(entities::txn_parties::Column::ContactId)
            .column_as(entities::transactions::Column::Amount.sum(), "amount")
            .group_by(entities::txn_parties::Column::ContactId)
            .order_by_desc(entities::transactions::Column::Amount.sum())
            .limit(5)
            .into_model::<TopSource>()
            .all(db)
            .await?;

        if results.is_empty() {
            return Ok(Vec::new());
        }

        let contact_ids: Vec<String> = results.iter().map(|r| r.contact_id.clone()).collect();
        let contacts = entities::contacts::Entity::find()
            .filter(entities::contacts::Column::Id.is_in(contact_ids))
            .all(db)
            .await?;

        let contact_map: std::collections::HashMap<String, String> =
            contacts.into_iter().map(|c| (c.id, c.name)).collect();

        Ok(results
            .into_iter()
            .map(|r| NamedAmount {
                name: contact_map
                    .get(&r.contact_id)
                    .cloned()
                    .unwrap_or_else(|| "Unknown".to_string()),
                amount: r.amount,
            })
            .collect())
    }

    let top_expenses = get_top_sources(db, user_id, "OUT").await?;
    let top_income = get_top_sources(db, user_id, "IN").await?;

    Ok(DashboardSummary {
        total_balance,
        monthly_spend: monthly_spend.unwrap_or(Decimal::ZERO),
        monthly_income: monthly_income.unwrap_or(Decimal::ZERO),
        pending_p2p_count: pending_p2p_count as u64,
        total_transactions: total_transactions as u64,
        monthly_trends,
        weekly_trends,
        category_distribution,
        top_expenses,
        top_income,
    })
}
