use super::*;
use crate::receipts_filter::filter_receipts;
use crate::receipts_filter::tests::dividends;
use crate::receipts_filter::ReceiptSearch;
use crate::session::SessionStore;
use leptos::prelude::*;
use std::collections::{HashMap, HashSet};

#[test]
fn tab_switch_resets_search_but_same_tab_and_cache_keep_it() {
    let owner = Owner::new();
    owner.with(|| {
        let store = ReceiptsStore {
            session: SessionStore::new(),
            active_tab: RwSignal::new(ReceiptsTab::Dividend),
            search: RwSignal::new(ReceiptSearch {
                query: "9432".into(),
                ..Default::default()
            }),
            expanded: RwSignal::new(HashSet::new()),
            mobile_summary_expanded: RwSignal::new(false),
            expanded_epoch: RwSignal::new(None),
            visited: RwSignal::new(HashSet::new()),
            cache: RwSignal::new(HashMap::from([(
                (0, ReceiptsTab::Dividend),
                TabState::Ready(ReceiptTabData {
                    rows: dividends(),
                    summary: None,
                    truncated: false,
                }),
            )])),
            fetch: Action::new_unsync(|_: &(u64, ReceiptsTab)| async {}),
            csv: RwSignal::new(HashMap::new()),
            csv_files: RwSignal::new(HashMap::new()),
        };
        assert_eq!(
            leptos::prelude::untrack(|| {
                filter_receipts(
                    ReceiptsTab::Dividend,
                    &store.rows(ReceiptsTab::Dividend),
                    &store.search.get().query,
                )
            })
            .len(),
            2
        );
        assert_eq!(
            leptos::prelude::untrack(|| store.count(ReceiptsTab::Dividend)),
            3
        );
        store.expanded.update(|set| {
            set.insert("g0".to_string());
        });
        store.mobile_summary_expanded.set(true);
        store.select_tab(ReceiptsTab::Dividend);
        assert_eq!(store.search.get_untracked().query, "9432");
        assert!(store.expanded.with_untracked(|set| set.contains("g0")));
        assert!(store.mobile_summary_expanded.get_untracked());
        store.select_tab(ReceiptsTab::DomesticStock);
        assert_eq!(store.search.get_untracked(), ReceiptSearch::default());
        assert!(!store.expanded.with_untracked(|set| set.contains("g0")));
        assert!(!store.mobile_summary_expanded.get_untracked());
        store.select_tab(ReceiptsTab::Dividend);
        assert_eq!(
            leptos::prelude::untrack(|| {
                filter_receipts(
                    ReceiptsTab::Dividend,
                    &store.rows(ReceiptsTab::Dividend),
                    &store.search.get().query,
                )
            })
            .len(),
            3
        );
    });
}
