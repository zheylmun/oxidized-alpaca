//! Helpers shared by the multi-symbol historical endpoints
//! (`stock_bars_multi`, `stock_quotes_multi`, `stock_trades_multi`).
//!
//! All three endpoints page through the response merging per-symbol series
//! into a `HashMap<String, Vec<T>>`, applying a client-side per-symbol cap
//! and stopping once every requested symbol has reached the cap. The logic
//! is identical regardless of the item type, so it lives here as a generic
//! helper to avoid drift across the three sites.

use std::collections::HashMap;

/// Alpaca's documented per-page maximum for the multi-symbol bars,
/// trades, and quotes endpoints. Used to clamp the internal page-size
/// hint so a large per-symbol cap times a large symbol list never asks
/// for more than the API will serve in a single page.
pub(super) const MAX_PAGE_SIZE: usize = 10_000;

/// Compute the server-side `limit` query parameter to send when the
/// caller has set a client-side per-symbol cap. We aim to fit the cap
/// for every requested symbol in a single page (`cap * symbols`), but
/// clamp to the API's per-page maximum.
pub(super) fn page_size_hint(cap: Option<usize>, symbol_count: usize) -> Option<usize> {
    let cap = cap?;
    if cap == 0 || symbol_count == 0 {
        return None;
    }
    Some(cap.saturating_mul(symbol_count).min(MAX_PAGE_SIZE))
}

/// Append the items from one paginated response page to the running
/// per-symbol map, truncating each symbol's series to `cap` when one is
/// set.
pub(super) fn extend_capped<T>(
    combined: &mut HashMap<String, Vec<T>>,
    page: HashMap<String, Vec<T>>,
    cap: Option<usize>,
) {
    for (symbol, items) in page {
        let entry = combined.entry(symbol).or_default();
        entry.extend(items);
        if let Some(cap) = cap {
            entry.truncate(cap);
        }
    }
}

/// Return `true` once every requested symbol has at least `cap` items in
/// the running map. Symbols absent from the map count as zero, so a
/// genuinely empty/illiquid symbol will keep pagination going until the
/// API itself stops returning a `next_page_token`.
pub(super) fn all_symbols_filled<T>(
    combined: &HashMap<String, Vec<T>>,
    requested: &[String],
    cap: usize,
) -> bool {
    requested
        .iter()
        .all(|s| combined.get(s).map_or(0, Vec::len) >= cap)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(entries: &[(&str, &[i32])]) -> HashMap<String, Vec<i32>> {
        entries
            .iter()
            .map(|(s, v)| ((*s).to_string(), v.to_vec()))
            .collect()
    }

    #[test]
    fn extend_capped_without_cap_accumulates() {
        let mut combined = HashMap::new();
        extend_capped(&mut combined, page(&[("AAPL", &[1, 2])]), None);
        extend_capped(&mut combined, page(&[("AAPL", &[3, 4, 5])]), None);
        assert_eq!(combined["AAPL"], vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn extend_capped_truncates_to_cap_across_pages() {
        let mut combined = HashMap::new();
        extend_capped(&mut combined, page(&[("AAPL", &[1, 2])]), Some(3));
        extend_capped(&mut combined, page(&[("AAPL", &[3, 4, 5])]), Some(3));
        assert_eq!(combined["AAPL"], vec![1, 2, 3]);
    }

    #[test]
    fn extend_capped_creates_entries_for_new_symbols() {
        let mut combined = HashMap::new();
        extend_capped(
            &mut combined,
            page(&[("AAPL", &[1]), ("MSFT", &[2, 3])]),
            Some(5),
        );
        assert_eq!(combined.len(), 2);
        assert_eq!(combined["AAPL"], vec![1]);
        assert_eq!(combined["MSFT"], vec![2, 3]);
    }

    #[test]
    fn extend_capped_with_zero_cap_clears_entries() {
        let mut combined = HashMap::new();
        extend_capped(&mut combined, page(&[("AAPL", &[1, 2])]), Some(0));
        assert!(combined["AAPL"].is_empty());
    }

    #[test]
    fn all_symbols_filled_true_when_every_symbol_meets_cap() {
        let mut combined: HashMap<String, Vec<i32>> = HashMap::new();
        combined.insert("AAPL".into(), vec![1, 2, 3]);
        combined.insert("MSFT".into(), vec![4, 5, 6]);
        let requested = vec!["AAPL".to_string(), "MSFT".to_string()];
        assert!(all_symbols_filled(&combined, &requested, 3));
    }

    #[test]
    fn all_symbols_filled_false_when_a_symbol_short() {
        let mut combined: HashMap<String, Vec<i32>> = HashMap::new();
        combined.insert("AAPL".into(), vec![1, 2, 3]);
        combined.insert("MSFT".into(), vec![4]);
        let requested = vec!["AAPL".to_string(), "MSFT".to_string()];
        assert!(!all_symbols_filled(&combined, &requested, 3));
    }

    #[test]
    fn all_symbols_filled_false_when_symbol_missing() {
        let mut combined: HashMap<String, Vec<i32>> = HashMap::new();
        combined.insert("AAPL".into(), vec![1, 2, 3]);
        let requested = vec!["AAPL".to_string(), "MSFT".to_string()];
        assert!(!all_symbols_filled(&combined, &requested, 1));
    }

    #[test]
    fn page_size_hint_scales_with_symbol_count() {
        assert_eq!(page_size_hint(Some(100), 3), Some(300));
    }

    #[test]
    fn page_size_hint_clamps_to_api_max() {
        assert_eq!(page_size_hint(Some(5_000), 4), Some(MAX_PAGE_SIZE));
    }

    #[test]
    fn page_size_hint_none_without_cap_or_symbols() {
        assert_eq!(page_size_hint(None, 3), None);
        assert_eq!(page_size_hint(Some(0), 3), None);
        assert_eq!(page_size_hint(Some(50), 0), None);
    }
}
