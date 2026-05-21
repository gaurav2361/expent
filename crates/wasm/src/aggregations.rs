use crate::text::normalize_text;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::str::FromStr;
use ts_rs::TS;
use wasm_bindgen::prelude::*;

#[derive(serde::Deserialize, serde::Serialize, TS)]
#[ts(export)]
pub struct Txn {
    pub amount: String,
    pub direction: String,
    pub status: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, TS)]
#[ts(export)]
pub struct AggregatedMetrics {
    pub total_income: String,
    pub total_expense: String,
    pub net_balance: String,
    pub count: usize,
}

#[wasm_bindgen]
pub fn aggregate_transactions(transactions: JsValue) -> Result<JsValue, JsError> {
    let txns: Vec<Txn> = serde_wasm_bindgen::from_value(transactions)?;
    let result = aggregate_transactions_internal(txns);
    Ok(serde_wasm_bindgen::to_value(&result)?)
}

pub fn aggregate_transactions_internal(txns: Vec<Txn>) -> AggregatedMetrics {
    let mut total_income = Decimal::ZERO;
    let mut total_expense = Decimal::ZERO;
    let mut count = 0;

    for tx in txns {
        if tx.status.as_deref() == Some("CANCELLED") {
            continue;
        }
        let amt = Decimal::from_str(&tx.amount).unwrap_or(Decimal::ZERO);
        if tx.direction == "IN" {
            total_income += amt;
        } else {
            total_expense += amt;
        }
        count += 1;
    }

    AggregatedMetrics {
        total_income: total_income.to_string(),
        total_expense: total_expense.to_string(),
        net_balance: (total_income - total_expense).to_string(),
        count,
    }
}

#[derive(serde::Deserialize, serde::Serialize, TS)]
#[ts(export)]
pub struct TxnPattern {
    pub amount: String,
    pub date: String,
    pub purpose_tag: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, TS)]
#[ts(export)]
pub struct DetectedSubscription {
    pub name: String,
    pub amount: String,
    pub cycle: String,
    pub last_date: String,
    pub count: usize,
}

#[wasm_bindgen]
pub fn detect_subscription_patterns(transactions: JsValue) -> Result<JsValue, JsError> {
    let txns: Vec<TxnPattern> = serde_wasm_bindgen::from_value(transactions)?;
    let suspected = detect_subscription_patterns_internal(txns);
    Ok(serde_wasm_bindgen::to_value(&suspected)?)
}

pub fn detect_subscription_patterns_internal(mut txns: Vec<TxnPattern>) -> Vec<DetectedSubscription> {
    // Sort by date
    txns.sort_by_key(|t| t.date.clone());

    let mut patterns = HashMap::new();

    // Group by normalized purpose + amount
    for tx in &txns {
        let key = format!(
            "{}:{}",
            normalize_text(tx.purpose_tag.as_deref().unwrap_or("unknown")),
            tx.amount
        );
        patterns.entry(key).or_insert_with(Vec::new).push(tx);
    }

    let mut suspected = Vec::new();

    for (key, group) in patterns {
        if group.len() < 2 {
            continue;
        }

        let mut intervals = Vec::new();
        for i in 0..group.len() - 1 {
            let d1 = chrono::DateTime::parse_from_rfc3339(&group[i].date).ok();
            let d2 = chrono::DateTime::parse_from_rfc3339(&group[i + 1].date).ok();
            if let (Some(d1), Some(d2)) = (d1, d2) {
                let diff = (d2 - d1).num_days();
                intervals.push(diff);
            }
        }

        if intervals.is_empty() {
            continue;
        }

        // Heuristic: if intervals are mostly around 30 days (+- 3) or 7 days (+- 1)
        let is_monthly = intervals.iter().all(|&d| (27..=33).contains(&d));
        let is_weekly = intervals.iter().all(|&d| (6..=8).contains(&d));

        if is_monthly || is_weekly {
            suspected.push(DetectedSubscription {
                name: group[0]
                    .purpose_tag
                    .clone()
                    .unwrap_or_else(|| key.clone()),
                amount: group[0].amount.clone(),
                cycle: (if is_monthly { "MONTHLY" } else { "WEEKLY" }).to_string(),
                last_date: group.last().unwrap().date.clone(),
                count: group.len(),
            });
        }
    }
    suspected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregation() {
        let txns = vec![
            Txn {
                amount: "100".to_string(),
                direction: "IN".to_string(),
                status: None,
            },
            Txn {
                amount: "50".to_string(),
                direction: "OUT".to_string(),
                status: None,
            },
            Txn {
                amount: "1000".to_string(),
                direction: "OUT".to_string(),
                status: Some("CANCELLED".to_string()),
            },
        ];
        let res = aggregate_transactions_internal(txns);

        assert_eq!(res.total_income, "100");
        assert_eq!(res.total_expense, "50");
        assert_eq!(res.net_balance, "50");
        assert_eq!(res.count, 2);
    }

    #[test]
    fn test_subscription_detection() {
        let txns = vec![
            TxnPattern {
                amount: "500".to_string(),
                date: "2023-01-01T10:00:00Z".to_string(),
                purpose_tag: Some("Netflix".to_string()),
            },
            TxnPattern {
                amount: "500".to_string(),
                date: "2023-02-01T10:00:00Z".to_string(),
                purpose_tag: Some("Netflix".to_string()),
            },
            TxnPattern {
                amount: "500".to_string(),
                date: "2023-03-01T10:00:00Z".to_string(),
                purpose_tag: Some("Netflix".to_string()),
            },
        ];
        let res = detect_subscription_patterns_internal(txns);

        assert_eq!(res.len(), 1);
        assert_eq!(res[0].name, "Netflix");
        assert_eq!(res[0].cycle, "MONTHLY");
    }
}
