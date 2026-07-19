//! Helpers shared by the multi-symbol historical endpoints
//! (`stock_bars_multi`, `stock_quotes_multi`, `stock_trades_multi`).
//!
//! All three endpoints page through the response merging per-symbol series
//! into a `HashMap<String, Vec<T>>`, applying a client-side per-symbol cap
//! and stopping once every requested symbol has reached the cap. The logic
//! is identical regardless of the item type, so it lives here as a generic
//! helper to avoid drift across the three sites.

use std::collections::HashMap;

/// Append the items from one paginated response page to the running
/// per-symbol map, truncating each symbol's series to `cap` when one is
/// set.
pub(crate) fn extend_capped<T>(
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

/// Return the subset of `requested` that has not yet reached `cap` items
/// in `combined`. Symbols absent from the map count as zero, so a
/// genuinely empty/illiquid symbol stays pending until the API itself
/// stops returning a `next_page_token`. The order of `requested` is
/// preserved so the resulting `?symbols=` query is stable.
pub(crate) fn pending_symbols<T>(
    combined: &HashMap<String, Vec<T>>,
    requested: &[String],
    cap: usize,
) -> Vec<String> {
    requested
        .iter()
        .filter(|s| combined.get(s.as_str()).map_or(0, Vec::len) < cap)
        .cloned()
        .collect()
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
    fn pending_symbols_empty_when_every_symbol_meets_cap() {
        let mut combined: HashMap<String, Vec<i32>> = HashMap::new();
        combined.insert("AAPL".into(), vec![1, 2, 3]);
        combined.insert("MSFT".into(), vec![4, 5, 6]);
        let requested = vec!["AAPL".to_string(), "MSFT".to_string()];
        assert!(pending_symbols(&combined, &requested, 3).is_empty());
    }

    #[test]
    fn pending_symbols_returns_short_symbols() {
        let mut combined: HashMap<String, Vec<i32>> = HashMap::new();
        combined.insert("AAPL".into(), vec![1, 2, 3]);
        combined.insert("MSFT".into(), vec![4]);
        let requested = vec!["AAPL".to_string(), "MSFT".to_string()];
        assert_eq!(
            pending_symbols(&combined, &requested, 3),
            vec!["MSFT".to_string()]
        );
    }

    #[test]
    fn pending_symbols_returns_missing_symbols() {
        let mut combined: HashMap<String, Vec<i32>> = HashMap::new();
        combined.insert("AAPL".into(), vec![1, 2, 3]);
        let requested = vec!["AAPL".to_string(), "MSFT".to_string()];
        assert_eq!(
            pending_symbols(&combined, &requested, 1),
            vec!["MSFT".to_string()]
        );
    }

    #[test]
    fn pending_symbols_preserves_request_order() {
        let combined: HashMap<String, Vec<i32>> = HashMap::new();
        let requested = vec!["AAPL".to_string(), "MSFT".to_string(), "GOOG".to_string()];
        assert_eq!(pending_symbols(&combined, &requested, 1), requested);
    }
}
