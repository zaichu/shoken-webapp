use super::ReceiptRow;
use crate::api::dto::AssetBalance;
use crate::api::ApiClient;
use crate::features::asset_balance::{fetch_single_asset_balance, find_by_code, to_fixed};
use crate::features::dividend_per_share::{
    dividend_maps_from_batch, dividend_pending_max_retries, fetch_dividend_batch,
    post_dividend_batch, DIVIDEND_NETWORK_MAX_RETRIES, DIVIDEND_RETRY_DELAY_MS,
};
use crate::features::receipts::model::{format_currency, format_number, DividendTotals};
use crate::session::{Generation, SessionStore};
use crate::support::list_search::group_key::derive_security_code_from_query;
use crate::ui::badge::{Badge, BadgeVariant};
use crate::ui::card::{Card, CardVariant};
use crate::ui::disclosure::{DisclosureStyle, DisclosureToggle};
use crate::ui::security_link::is_searchable_code;
use leptos::prelude::*;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use shared::normalize::normalize_security_code;

const ASSET_BALANCE_HINT: &str = "資産管理にCSVを取り込むと表示されます";
const JQUANTS_HINT: &str = "自動で取得されます";

pub(crate) fn search_security_code(rows: &[ReceiptRow], query: &str) -> String {
    let code = derive_security_code_from_query(query, rows, |row| row.code(), |row| row.name());
    if is_searchable_code(&code) {
        code
    } else {
        String::new()
    }
}

fn format_percentage_value(value: f64) -> String {
    if value.is_nan() {
        return "—".to_string();
    }
    format!("{:.2}%", to_fixed(value, 2))
}

fn investment_amount(asset: &AssetBalance) -> f64 {
    asset.average_purchase_price.to_f64().unwrap_or(0.0) * asset.shares.to_f64().unwrap_or(0.0)
}

fn dividend_rate(amount: f64, investment: f64) -> f64 {
    if investment > 0.0 && amount > 0.0 {
        amount / investment * 100.0
    } else {
        0.0
    }
}

fn per_share_display(per_share: Option<f64>, loading: bool) -> String {
    if loading {
        "取得中...".to_string()
    } else {
        per_share
            .and_then(|value| value.to_string().parse::<Decimal>().ok())
            .map_or_else(|| "—".to_string(), format_currency)
    }
}

/// 非同期応答の後着で表示が巻き戻らないよう、銘柄切替ごとのリビジョンを持つ
#[derive(Clone, Copy)]
pub(crate) struct DividendInfoStore {
    session: SessionStore,
    current: RwSignal<Option<(Generation, String)>>,
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

    pub fn set_code(&self, generation: Generation, authenticated: bool, raw_code: &str) {
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

    fn is_current_code(&self, generation: Generation, code: &str) -> bool {
        self.session.is_current(generation)
            && self
                .current
                .get_untracked()
                .is_some_and(|(g, c)| g == generation && c == code)
    }

    fn is_balance_active(&self, generation: Generation, revision: u64, code: &str) -> bool {
        self.balance_revision.get_untracked() == revision && self.is_current_code(generation, code)
    }

    fn is_poll_active(&self, generation: Generation, revision: u64, code: &str) -> bool {
        self.code_revision.get_untracked() == revision && self.is_current_code(generation, code)
    }

    /// pending（バックエンド処理待ち）と通信失敗は別カウンタで打ち切る
    async fn poll_dividend(&self, generation: Generation, revision: u64, code: String) {
        let codes = vec![code.clone()];
        let max_pending = dividend_pending_max_retries(1);
        let mut pending_used = 0u32;
        let mut network_used = 0u32;
        loop {
            if !self.is_poll_active(generation, revision, &code) {
                return;
            }
            self.per_share_loading.set(true);
            match fetch_dividend_batch(&codes, post_dividend_batch).await {
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
    view! { <Badge variant=BadgeVariant::Positive>"保有銘柄"</Badge> }
}

#[component]
pub(crate) fn DividendInfo(store: DividendInfoStore, totals: DividendTotals) -> impl IntoView {
    let asset = move || store.asset_balance.get();
    let per_share = move || store.per_share.get();
    let loading = move || store.per_share_loading.get();

    let investment = move || asset().map_or(0.0, |a| investment_amount(&a));
    let gross = totals.total_dividends_before_tax.to_f64().unwrap_or(0.0);
    let net = totals.total_net_amount_received.to_f64().unwrap_or(0.0);
    let gross_rate = move || dividend_rate(gross, investment());
    let net_rate = move || dividend_rate(net, investment());

    let average_price_text = move || {
        asset().map_or_else(
            || "—".to_string(),
            |a| format_currency(a.average_purchase_price),
        )
    };
    let shares_text =
        move || asset().map_or_else(|| "—".to_string(), |a| format_number(a.shares, 2));
    let per_share_text = move || per_share_display(per_share(), loading());

    view! {
        <div>
            <div class="grid grid-cols-1 gap-4 md:grid-cols-3">
                <Card variant=CardVariant::Sunken>
                    <p class="text-xs font-medium text-text-muted mb-1">"平均取得価格"</p>
                    <p
                        class="text-2xl font-bold tabular-nums text-text-deep"
                        title=move || asset().is_none().then_some(ASSET_BALANCE_HINT)
                    >
                        {average_price_text}
                    </p>
                    {move || {
                        asset().is_none().then(|| {
                            view! { <p class="mt-0.5 text-xs text-text-subtle">{ASSET_BALANCE_HINT}</p> }
                        })
                    }}
                </Card>
                <Card variant=CardVariant::Sunken>
                    <p class="text-xs font-medium text-text-muted mb-1">"保有数量(株)"</p>
                    <p
                        class="text-2xl font-bold tabular-nums text-text-deep"
                        title=move || asset().is_none().then_some(ASSET_BALANCE_HINT)
                    >
                        {shares_text}
                    </p>
                    {move || {
                        asset().is_none().then(|| {
                            view! { <p class="mt-0.5 text-xs text-text-subtle">{ASSET_BALANCE_HINT}</p> }
                        })
                    }}
                </Card>
                <Card variant=CardVariant::Sunken>
                    <p class="text-xs font-medium text-text-muted mb-1">"一株配当"</p>
                    <p class="text-2xl font-bold tabular-nums text-text-deep">{per_share_text}</p>
                    {move || {
                        (!loading() && per_share().is_none()).then(|| {
                            view! { <p class="mt-0.5 text-xs text-text-subtle">{JQUANTS_HINT}</p> }
                        })
                    }}
                </Card>
            </div>
            <div class="grid grid-cols-1 gap-4 md:grid-cols-3 mt-4 pt-4 border-t border-border-subtle">
                <Card variant=CardVariant::Sunken>
                    <p class="text-xs font-medium text-text-muted mb-1">"配当金額 (配当利回り)"</p>
                    <p class="text-2xl font-bold tabular-nums text-ink">
                        {format_currency(totals.total_dividends_before_tax)}
                        {move || {
                            let rate = gross_rate();
                            (rate > 0.0).then(|| {
                                view! {
                                    <span class="text-sm font-normal text-text-subtle ml-1">
                                        {format!("({})", format_percentage_value(rate))}
                                    </span>
                                }
                            })
                        }}
                    </p>
                </Card>
                <Card variant=CardVariant::Sunken>
                    <p class="text-xs font-medium text-text-muted mb-1">"税額"</p>
                    <p class="text-2xl font-bold tabular-nums text-ink">
                        {format_currency(totals.total_taxes)}
                    </p>
                </Card>
                <Card variant=CardVariant::Sunken>
                    <p class="text-xs font-medium text-text-muted mb-1">"税引後 (累積利回り)"</p>
                    <p class="text-2xl font-bold tabular-nums text-ink">
                        {format_currency(totals.total_net_amount_received)}
                        {move || {
                            let rate = net_rate();
                            (rate > 0.0).then(|| {
                                view! {
                                    <span class="text-sm font-normal text-text-subtle ml-1">
                                        {format!("({})", format_percentage_value(rate))}
                                    </span>
                                }
                            })
                        }}
                    </p>
                </Card>
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
    #[prop(optional)] preview: bool,
) -> impl IntoView {
    let title = if preview {
        "集計情報(プレビュー)"
    } else {
        "集計情報"
    };
    let totals_mobile = totals.clone();
    view! {
        <Card variant=CardVariant::Collapsible testid="receipt-summary-strip" class=Signal::derive(move || {
            if mobile_expanded.get() { "" } else { "max-sm:hidden" }.to_string()
        })>
            <div
                id="receipt-summary-mobile-body"
                role="region"
                aria-label=title
                class="sm:hidden"
                hidden=move || !mobile_expanded.get()
            >
                {move || mobile_expanded.get().then(|| view! {
                    <DividendInfo store=store totals=totals_mobile.clone() />
                })}
            </div>
            <div class="hidden sm:block" data-testid="receipt-summary-desktop">
                <DisclosureToggle
                    style=DisclosureStyle::HeaderFlat
                    expanded=Signal::derive(move || expanded.get())
                    controls="receipt-summary-body".to_string()
                    testid="receipt-header"
                    hint=true
                    on_toggle=move || expanded.update(|open| *open = !*open)
                >
                    <div class="flex items-center gap-2">
                        <h2 class="text-sm font-black text-ink">{title}</h2>
                        {move || store.asset_balance.get().is_some().then(AssetBadge)}
                    </div>
                </DisclosureToggle>
                <div id="receipt-summary-body" hidden=move || !expanded.get() class="pt-3">
                    <DividendInfo store=store totals=totals.clone() />
                </div>
            </div>
        </Card>
    }
}

#[cfg(test)]
mod tests;
