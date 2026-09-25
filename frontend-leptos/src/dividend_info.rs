use crate::api::ApiClient;
use crate::asset_balance_domain::{normalize_security_code, to_fixed};
use crate::asset_balance_lookup::{fetch_single_asset_balance, find_by_code};
use crate::dividend_per_share::{
    dividend_maps_from_batch, dividend_pending_max_retries, fetch_dividend_batch,
    DIVIDEND_NETWORK_MAX_RETRIES, DIVIDEND_RETRY_DELAY_MS,
};
use crate::dto::{AssetBalance, Dividend};
use crate::receipts_domain::{format_currency, format_number, DividendTotals};
use crate::receipts_search_group_key::derive_security_code_from_query;
use crate::security_link::is_searchable_code;
use crate::session::SessionStore;
use leptos::prelude::*;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

const ASSET_BALANCE_HINT: &str = "資産管理にCSVを取り込むと表示されます";
const JQUANTS_HINT: &str = "自動で取得されます";

pub(crate) fn search_security_code(rows: &[Dividend], query: &str) -> String {
    let code = derive_security_code_from_query(
        query,
        rows,
        |row| row.security_code.as_str(),
        |row| row.security_name.as_str(),
    );
    if is_searchable_code(&code) {
        code
    } else {
        String::new()
    }
}

fn format_percentage_value(value: f64) -> String {
    if value.is_nan() {
        return "-".to_string();
    }
    format!("{:.2}%", to_fixed(value, 2))
}

fn per_share_display(per_share: Option<f64>, loading: bool) -> String {
    if loading {
        "取得中...".to_string()
    } else {
        per_share
            .and_then(|value| value.to_string().parse::<Decimal>().ok())
            .map(format_currency)
            .unwrap_or_else(|| "---".to_string())
    }
}

/// 非同期応答の後着で表示が巻き戻らないよう、銘柄切替ごとのリビジョンを持つ
#[derive(Clone, Copy)]
pub(crate) struct DividendInfoStore {
    session: SessionStore,
    current: RwSignal<Option<(u64, String)>>,
    code_revision: RwSignal<u64>,
    balance_revision: RwSignal<u64>,
    asset_balance: RwSignal<Option<AssetBalance>>,
    per_share: RwSignal<Option<f64>>,
    per_share_loading: RwSignal<bool>,
}

impl DividendInfoStore {
    pub fn new(session: SessionStore) -> Self {
        Self {
            session,
            current: RwSignal::new(None),
            code_revision: RwSignal::new(0),
            balance_revision: RwSignal::new(0),
            asset_balance: RwSignal::new(None),
            per_share: RwSignal::new(None),
            per_share_loading: RwSignal::new(false),
        }
    }

    pub fn set_code(&self, generation: u64, authenticated: bool, raw_code: &str) {
        let code = normalize_security_code(raw_code);
        let next = (authenticated && is_searchable_code(&code)).then_some((generation, code));
        if self.current.get_untracked() == next {
            return;
        }
        self.code_revision.update(|revision| *revision += 1);
        self.current.set(next.clone());
        self.asset_balance.set(None);
        self.per_share.set(None);
        self.per_share_loading.set(false);
        let Some((generation, code)) = next else {
            return;
        };

        self.refresh_balance();
        let revision = self.code_revision.get_untracked();
        let store = *self;
        leptos::task::spawn_local(async move {
            store.poll_dividend(generation, revision, code).await;
        });
    }

    /// 保有状況だけを再取得する。配当ポーリングは継続させるため code_revision は据え置く
    pub fn refresh_balance(&self) {
        let Some((generation, code)) = self.current.get_untracked() else {
            return;
        };
        if !self.session.is_current(generation) {
            return;
        }
        self.balance_revision.update(|revision| *revision += 1);
        let revision = self.balance_revision.get_untracked();
        let store = *self;
        leptos::task::spawn_local(async move {
            let client = ApiClient::read_client();
            if let Ok(rows) = fetch_single_asset_balance(&client, &code).await {
                if store.is_balance_active(generation, revision, &code) {
                    store.asset_balance.set(find_by_code(&rows, &code).cloned());
                }
            }
        });
    }

    fn is_current_code(&self, generation: u64, code: &str) -> bool {
        self.session.is_current(generation)
            && self
                .current
                .get_untracked()
                .is_some_and(|(g, c)| g == generation && c == code)
    }

    fn is_balance_active(&self, generation: u64, revision: u64, code: &str) -> bool {
        self.balance_revision.get_untracked() == revision && self.is_current_code(generation, code)
    }

    fn is_poll_active(&self, generation: u64, revision: u64, code: &str) -> bool {
        self.code_revision.get_untracked() == revision && self.is_current_code(generation, code)
    }

    /// pending（バックエンド処理待ち）と通信失敗は別カウンタで打ち切る
    async fn poll_dividend(&self, generation: u64, revision: u64, code: String) {
        let codes = vec![code.clone()];
        let max_pending = dividend_pending_max_retries(1);
        let mut pending_used = 0u32;
        let mut network_used = 0u32;
        loop {
            if !self.is_poll_active(generation, revision, &code) {
                return;
            }
            self.per_share_loading.set(true);
            match fetch_dividend_batch(&codes).await {
                Ok(batch) => {
                    let (maps, has_pending) = dividend_maps_from_batch(&batch);
                    if !self.is_poll_active(generation, revision, &code) {
                        return;
                    }
                    self.per_share.set(maps.per_share.get(&code).copied());
                    self.per_share_loading.set(false);
                    if !has_pending || pending_used >= max_pending {
                        return;
                    }
                    pending_used += 1;
                }
                Err(_) => {
                    if !self.is_poll_active(generation, revision, &code) {
                        return;
                    }
                    self.per_share_loading.set(false);
                    if network_used >= DIVIDEND_NETWORK_MAX_RETRIES {
                        return;
                    }
                    network_used += 1;
                }
            }
            gloo_timers::future::TimeoutFuture::new(DIVIDEND_RETRY_DELAY_MS).await;
        }
    }
}

#[component]
fn AssetBadge() -> impl IntoView {
    view! {
        <span class="ml-1 inline-flex items-center rounded bg-emerald-100 px-1.5 py-0.5 text-xs font-medium text-emerald-700">
            "保有銘柄"
        </span>
    }
}

#[component]
pub(crate) fn DividendInfo(store: DividendInfoStore, totals: DividendTotals) -> impl IntoView {
    let asset = move || store.asset_balance.get();
    let per_share = move || store.per_share.get();
    let loading = move || store.per_share_loading.get();

    let investment = move || {
        asset()
            .map(|a| {
                a.average_purchase_price.to_f64().unwrap_or(0.0) * a.shares.to_f64().unwrap_or(0.0)
            })
            .unwrap_or(0.0)
    };
    let gross = totals.total_dividends_before_tax.to_f64().unwrap_or(0.0);
    let net = totals.total_net_amount_received.to_f64().unwrap_or(0.0);
    let gross_rate = move || {
        let investment = investment();
        if investment > 0.0 && gross > 0.0 {
            gross / investment * 100.0
        } else {
            0.0
        }
    };
    let net_rate = move || {
        let investment = investment();
        if investment > 0.0 && net > 0.0 {
            net / investment * 100.0
        } else {
            0.0
        }
    };

    let average_price_text = move || {
        asset()
            .map(|a| format_currency(a.average_purchase_price))
            .unwrap_or_else(|| "---".to_string())
    };
    let shares_text = move || {
        asset()
            .map(|a| format_number(a.shares, 2))
            .unwrap_or_else(|| "---".to_string())
    };
    let per_share_text = move || per_share_display(per_share(), loading());

    view! {
        <div>
            <div class="grid grid-cols-1 gap-4 md:grid-cols-3">
                <div class="rounded-lg bg-slate-50 px-4 py-3">
                    <p class="text-xs font-medium text-slate-600 mb-1">
                        "平均取得価格"
                        {move || asset().is_some().then(AssetBadge)}
                    </p>
                    <p
                        class="text-2xl font-bold tabular-nums text-primary"
                        title=move || asset().is_none().then_some(ASSET_BALANCE_HINT)
                    >
                        {average_price_text}
                    </p>
                    {move || {
                        asset().is_none().then(|| {
                            view! { <p class="mt-0.5 text-xs text-slate-400">{ASSET_BALANCE_HINT}</p> }
                        })
                    }}
                </div>
                <div class="rounded-lg bg-slate-50 px-4 py-3">
                    <p class="text-xs font-medium text-slate-600 mb-1">
                        "保有数量(株)"
                        {move || asset().is_some().then(AssetBadge)}
                    </p>
                    <p
                        class="text-2xl font-bold tabular-nums text-primary"
                        title=move || asset().is_none().then_some(ASSET_BALANCE_HINT)
                    >
                        {shares_text}
                    </p>
                    {move || {
                        asset().is_none().then(|| {
                            view! { <p class="mt-0.5 text-xs text-slate-400">{ASSET_BALANCE_HINT}</p> }
                        })
                    }}
                </div>
                <div class="rounded-lg bg-slate-50 px-4 py-3">
                    <p class="text-xs font-medium text-slate-600 mb-1">"一株配当"</p>
                    <p class="text-2xl font-bold tabular-nums text-primary">{per_share_text}</p>
                    {move || {
                        (!loading() && per_share().is_none()).then(|| {
                            view! { <p class="mt-0.5 text-xs text-slate-400">{JQUANTS_HINT}</p> }
                        })
                    }}
                </div>
            </div>
            <div class="grid grid-cols-1 gap-4 md:grid-cols-3 mt-4 pt-4 border-t border-slate-200">
                <div class="rounded-lg bg-emerald-50 px-4 py-3">
                    <p class="text-xs font-medium text-slate-600 mb-1">"配当金額 (配当利回り)"</p>
                    <p class="text-2xl font-bold tabular-nums text-emerald-600">
                        {format_currency(totals.total_dividends_before_tax)}
                        {move || {
                            let rate = gross_rate();
                            (rate > 0.0).then(|| {
                                view! {
                                    <span class="text-sm font-normal text-slate-500 ml-1">
                                        {format!("({})", format_percentage_value(rate))}
                                    </span>
                                }
                            })
                        }}
                    </p>
                </div>
                <div class="rounded-lg bg-red-50 px-4 py-3">
                    <p class="text-xs font-medium text-slate-600 mb-1">"税額"</p>
                    <p class="text-2xl font-bold tabular-nums text-red-500">
                        {format_currency(totals.total_taxes)}
                    </p>
                </div>
                <div class="rounded-lg bg-emerald-50 px-4 py-3">
                    <p class="text-xs font-medium text-slate-600 mb-1">"受取金額 (累積利回り)"</p>
                    <p class="text-2xl font-bold tabular-nums text-emerald-600">
                        {format_currency(totals.total_net_amount_received)}
                        {move || {
                            let rate = net_rate();
                            (rate > 0.0).then(|| {
                                view! {
                                    <span class="text-sm font-normal text-slate-500 ml-1">
                                        {format!("({})", format_percentage_value(rate))}
                                    </span>
                                }
                            })
                        }}
                    </p>
                </div>
            </div>
        </div>
    }
}

/// 展開状態は呼び出し側の signal を持たせ、フィルタ変更で view が作り直されても消えないようにする
#[component]
pub(crate) fn DividendSummarySection(
    store: DividendInfoStore,
    totals: DividendTotals,
    expanded: RwSignal<bool>,
    mobile_expanded: RwSignal<bool>,
) -> impl IntoView {
    let open_label = |open: bool| if open { "閉じる" } else { "開く" };
    let totals_mobile = totals.clone();
    view! {
        <section
            class="mb-3 rounded-xl border border-slate-950/10 bg-white/95 px-4 py-3 shadow-[0_12px_34px_-30px_rgba(15,23,42,0.85)] max-sm:px-3 max-sm:py-0"
            data-testid="receipt-summary-strip"
        >
            <div class="sm:hidden" data-testid="receipt-summary-compact">
                <button
                    type="button"
                    class="flex min-h-[40px] w-full items-center justify-between gap-3 text-left focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-slate-950"
                    aria-expanded=move || {
                        if mobile_expanded.get() {
                            "true"
                        } else {
                            "false"
                        }
                    }
                    aria-controls="receipt-summary-mobile-body"
                    aria-label="集計情報"
                    data-testid="receipt-summary-compact-toggle"
                    on:click=move |_| mobile_expanded.update(|open| *open = !*open)
                >
                    <span class="text-sm font-black text-slate-950">"集計情報"</span>
                    <span class="flex shrink-0 items-center gap-1 text-slate-700">
                        <span class="text-xs font-semibold">
                            {move || open_label(mobile_expanded.get())}
                        </span>
                        <svg
                            aria-hidden="true"
                            class=move || {
                                if mobile_expanded.get() {
                                    "h-4 w-4 text-slate-500 transition-transform duration-200 rotate-180"
                                } else {
                                    "h-4 w-4 text-slate-500 transition-transform duration-200"
                                }
                            }
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="M19 9l-7 7-7-7"
                            />
                        </svg>
                    </span>
                </button>
                {move || {
                    mobile_expanded.get().then(|| {
                        view! {
                            <div
                                id="receipt-summary-mobile-body"
                                role="region"
                                class="border-t border-slate-950/10 py-3"
                            >
                                <DividendInfo store=store totals=totals_mobile.clone() />
                            </div>
                        }
                    })
                }}
            </div>
            <div class="hidden sm:block" data-testid="receipt-summary-desktop">
                <button
                    type="button"
                    class="flex w-full items-start justify-between gap-3 border-b border-slate-950/10 pb-2.5 text-left"
                    aria-expanded=move || {
                        if expanded.get() {
                            "true"
                        } else {
                            "false"
                        }
                    }
                    aria-controls="receipt-summary-body"
                    data-testid="receipt-header"
                    on:click=move |_| expanded.update(|open| *open = !*open)
                >
                    <div>
                        <h2 class="text-sm font-black text-slate-950">"集計情報"</h2>
                    </div>
                    <span
                        class="flex items-center gap-1.5 rounded-md border border-slate-300 bg-white px-2.5 py-1 text-slate-700"
                        aria-hidden="true"
                    >
                        <span class="text-xs font-semibold">{move || open_label(expanded.get())}</span>
                        <svg
                            class=move || {
                                if expanded.get() {
                                    "w-4 h-4 text-slate-500 transition-transform duration-200 rotate-180"
                                } else {
                                    "w-4 h-4 text-slate-500 transition-transform duration-200"
                                }
                            }
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="M19 9l-7 7-7-7"
                            />
                        </svg>
                    </span>
                </button>
                <div id="receipt-summary-body" hidden=move || !expanded.get() class="pt-3">
                    <DividendInfo store=store totals=totals.clone() />
                </div>
            </div>
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn dividend(code: &str, name: &str) -> Dividend {
        Dividend {
            id: "id".to_string(),
            settlement_date: "2024-03-01".to_string(),
            product: "特定口座".to_string(),
            account: "SBI証券".to_string(),
            security_code: code.to_string(),
            security_name: name.to_string(),
            unit_price: dec!(30),
            shares: dec!(100),
            dividends_before_tax: dec!(3000),
            taxes: dec!(609),
            net_amount_received: dec!(2391),
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    fn authenticated_session() -> SessionStore {
        let session = SessionStore::new();
        session.user.set(Some(crate::dto::SessionUser {
            id: "a".to_string(),
            email: "a@example.com".to_string(),
            name: None,
            picture_url: None,
        }));
        session
    }

    fn init_test_executor() {
        let _ = any_spawner::Executor::init_futures_executor();
    }

    #[test]
    fn search_security_code_matches_react_gating() {
        let rows = vec![dividend("7203", "トヨタ自動車"), dividend("6758", "ソニー")];
        assert_eq!(search_security_code(&rows, "7203"), "7203");
        assert_eq!(search_security_code(&rows, "トヨタ自動車"), "7203");
        assert_eq!(search_security_code(&rows, "7203: トヨタ自動車"), "7203");
        assert_eq!(search_security_code(&rows, "トヨタ"), "");
        assert_eq!(search_security_code(&rows, ""), "");
        assert_eq!(search_security_code(&rows, "9999"), "");
        assert_eq!(search_security_code(&[], "7203"), "");
    }

    #[test]
    fn search_security_code_rejects_non_regex_codes() {
        let rows = vec![dividend("７２０３", "トヨタ自動車")];
        assert_eq!(search_security_code(&rows, "７２０３"), "");
    }

    #[test]
    fn set_code_dedupes_same_generation_code_and_clears_when_disabled() {
        let owner = Owner::new();
        owner.with(|| {
            let session = authenticated_session();
            let generation = session.generation.get_untracked();
            let store = DividendInfoStore::new(session);

            store.set_code(generation, false, "7203");
            assert!(store.current.get_untracked().is_none());

            store.set_code(generation, true, "７２０３");
            assert!(store.current.get_untracked().is_none());

            store.current.set(Some((generation, "7203".to_string())));
            store.set_code(generation, true, " 7203 ");
            assert_eq!(
                store.current.get_untracked(),
                Some((generation, "7203".to_string()))
            );

            store.set_code(generation, false, "7203");
            assert!(store.current.get_untracked().is_none());
            assert!(store.asset_balance.get_untracked().is_none());
            assert!(store.per_share.get_untracked().is_none());
        });
    }

    #[test]
    fn set_code_bumps_revisions_so_stale_requests_stay_inactive() {
        init_test_executor();
        let owner = Owner::new();
        owner.with(|| {
            let session = authenticated_session();
            let generation = session.generation.get_untracked();
            let store = DividendInfoStore::new(session);

            store.set_code(generation, true, "7203");
            let first_poll = store.code_revision.get_untracked();
            let first_balance = store.balance_revision.get_untracked();

            store.set_code(generation, true, "6758");
            store.set_code(generation, true, "7203");
            let latest_poll = store.code_revision.get_untracked();
            assert!(latest_poll > first_poll);

            // A→B→A と戻っても、先に投げたポーリング・保有数取得は再有効化しない
            assert!(!store.is_poll_active(generation, first_poll, "7203"));
            assert!(!store.is_balance_active(generation, first_balance, "7203"));
            assert!(store.is_poll_active(generation, latest_poll, "7203"));
        });
    }

    #[test]
    fn refresh_balance_supersedes_balance_fetch_and_keeps_poll_alive() {
        init_test_executor();
        let owner = Owner::new();
        owner.with(|| {
            let session = authenticated_session();
            let generation = session.generation.get_untracked();
            let store = DividendInfoStore::new(session);
            store.set_code(generation, true, "7203");
            let poll = store.code_revision.get_untracked();
            let balance = store.balance_revision.get_untracked();

            store.refresh_balance();
            assert_eq!(store.balance_revision.get_untracked(), balance + 1);
            assert!(!store.is_balance_active(generation, balance, "7203"));
            assert!(store.is_balance_active(generation, balance + 1, "7203"));
            assert_eq!(store.code_revision.get_untracked(), poll);
            assert!(store.is_poll_active(generation, poll, "7203"));

            store.set_code(generation, false, "7203");
            let bumped = store.balance_revision.get_untracked();
            store.refresh_balance();
            assert_eq!(store.balance_revision.get_untracked(), bumped);
        });
    }

    #[test]
    fn per_share_display_uses_short_decimal_digits() {
        assert_eq!(per_share_display(Some(50.1), false), "¥ 50.1");
        assert_eq!(per_share_display(Some(50.0), false), "¥ 50");
        assert_eq!(per_share_display(Some(50.12345), false), "¥ 50.12345");
        assert_eq!(per_share_display(None, false), "---");
        assert_eq!(per_share_display(Some(50.1), true), "取得中...");
        assert_eq!(per_share_display(Some(f64::NAN), false), "---");
    }

    #[test]
    fn percentage_value_matches_react_to_fixed() {
        assert_eq!(format_percentage_value(2.0), "2.00%");
        assert_eq!(format_percentage_value(0.9564), "0.96%");
        assert_eq!(format_percentage_value(f64::NAN), "-");
    }
}
