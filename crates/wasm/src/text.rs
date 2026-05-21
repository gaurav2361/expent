use ts_rs::TS;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn normalize_text(text: &str) -> String {
    use any_ascii::any_ascii;
    any_ascii(text)
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[wasm_bindgen]
pub fn phonetic_encode(text: &str) -> String {
    use rphonetic::{Encoder, Metaphone};
    let normalized = normalize_text(text);
    let metaphone = Metaphone::default();
    normalized
        .split_whitespace()
        .map(|word| metaphone.encode(word))
        .collect::<Vec<_>>()
        .join(" ")
}

#[wasm_bindgen]
pub fn fuzzy_score(a: &str, b: &str) -> f64 {
    strsim::jaro_winkler(a, b)
}

#[derive(serde::Serialize, TS)]
#[ts(export)]
pub struct FuzzySearchResult {
    pub index: usize,
    pub score: f64,
}

#[wasm_bindgen]
pub fn batch_fuzzy_search(query: &str, items: Vec<String>, threshold: f64) -> JsValue {
    let query_norm = normalize_text(query);
    let mut results: Vec<FuzzySearchResult> = items
        .into_iter()
        .enumerate()
        .filter_map(|(index, item)| {
            let item_norm = normalize_text(&item);
            let score = fuzzy_score(&query_norm, &item_norm);
            if score >= threshold {
                Some(FuzzySearchResult { index, score })
            } else {
                None
            }
        })
        .collect();

    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    serde_wasm_bindgen::to_value(&results).unwrap_or(JsValue::NULL)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalization() {
        assert_eq!(normalize_text("Gaurav S."), "gaurav s");
        assert_eq!(normalize_text("UPI: someone@okaxis"), "upi someone okaxis");
    }

    #[test]
    fn test_phonetic() {
        let a = phonetic_encode("Gaurav");
        let b = phonetic_encode("Gourav");
        assert_eq!(a, b);
    }
}
