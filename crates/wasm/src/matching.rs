use crate::text::{fuzzy_score, normalize_text};
use rust_decimal::Decimal;
use std::str::FromStr;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn calculate_match_score(
    row_date_ms: i64,
    row_desc: &str,
    row_amount: String,
    txn_date_ms: i64,
    txn_desc: &str,
    txn_amount: String,
) -> i32 {
    let row_amt = Decimal::from_str(&row_amount)
        .unwrap_or(Decimal::ZERO)
        .abs();
    let txn_amt = Decimal::from_str(&txn_amount)
        .unwrap_or(Decimal::ZERO)
        .abs();

    let mut score = 0;

    // Amount match (Absolute)
    if row_amt == txn_amt {
        score += 60;
    } else {
        // Proximity for amounts
        let diff = (row_amt - txn_amt).abs();
        if diff < Decimal::from(10) {
            score += 20;
        }
    }

    // Date proximity
    let date_diff_ms = (row_date_ms - txn_date_ms).abs();
    let three_days_ms = 3 * 24 * 60 * 60 * 1000;

    if date_diff_ms == 0 {
        score += 30;
    } else if date_diff_ms < three_days_ms {
        score += 15;
    }

    // Description fuzzy match
    let desc_score = fuzzy_score(&normalize_text(row_desc), &normalize_text(txn_desc));
    score += (desc_score * 10.0) as i32;

    score.min(100)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_scoring() {
        let score = calculate_match_score(
            1000,
            "Starbucks Coffee",
            "500".to_string(),
            1000,
            "Starbucks",
            "500".to_string(),
        );
        assert!(score >= 90);
    }
}
