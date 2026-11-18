use super::ReceiptRow;
use crate::api::ApiClient;
use crate::api::dto::AssetBalance;
use crate::features::asset_balance;
use crate::features::dividend_per_share;
use crate::features::receipts::model::DividendTotals;
use crate::session::{Generation, SessionStore};
use crate::settings;
use crate::support::list_search;
use crate::support::list_search::group_key;
use crate::ui::amount::Amount;
use crate::ui::badge::{Badge, BadgeVariant};
use crate::ui::card::{Card, CardVariant};
use crate::ui::disclosure::{DisclosureStyle, DisclosureToggle};
use crate::ui::format;
use crate::ui::labels;
use crate::ui::receipts::{
    DividendStatCard, DividendStatGrid, DividendSummaryBody, DividendSummaryHeading,
    DividendSummaryTitle, DividendTotalCard, DividendTotalGrid,
};
use leptos::prelude::*;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use shared::format::{format_currency, format_number};
use shared::normalize::normalize_security_code;

pub(crate) fn search_security_code(rows: &[ReceiptRow], query: &str) -> String {
    let code =
        group_key::derive_security_code_from_query(query, rows, |row| row.code(), |row| row.name());
    if list_search::is_searchable_code(&code) {
        code
    } else {
        String::new()
    }
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
        labels::CSV_MANAGE_FETCH_RUNNING.to_string()
    } else {
        per_share.map_or_else(
            || labels::MISSING.to_string(),
            |value| {
                // 1株配当は列ごとに桁をそろえる決まり(整数のみなら0桁、最大2桁)
                let decimals = format::unit_price_decimals(std::iter::once(value));
                format::format_unit_price(value, decimals)
            },
        )
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
        let next =
            (authenticated && list_search::is_searchable_code(&code)).then_some((generation, code));
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
            if let Ok(rows) = asset_balance::fetch_single_asset_balance(&client, &code).await
                && store.is_balance_active(generation, revision, &code)
            {
                store
                    .asset_balance
                    .set(asset_balance::find_by_code(&rows, &code).cloned());
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
        let max_pending = dividend_per_share::dividend_pending_max_retries(1);
        let mut pending_used = 0u32;
        let mut network_used = 0u32;
        loop {
            if !self.is_poll_active(generation, revision, &code) {
                return;
            }
            self.per_share_loading.set(true);
            if let Ok(batch) = dividend_per_share::fetch_dividend_batch(
                &codes,
                dividend_per_share::post_dividend_batch,
            )
            .await
            {
                let (maps, has_pending) = dividend_per_share::dividend_maps_from_batch(&batch);
                if !self.is_poll_active(generation, revision, &code) {
                    return;
                }
                self.per_share.set(maps.per_share.get(&code).copied());
                self.per_share_loading.set(false);
                if !has_pending || pending_used >= max_pending {
                    return;
                }
                pending_used += 1;
            } else {
                if !self.is_poll_active(generation, revision, &code) {
                    return;
                }
                self.per_share_loading.set(false);
                if network_used >= dividend_per_share::DIVIDEND_NETWORK_MAX_RETRIES {
                    return;
                }
                network_used += 1;
            }
            gloo_timers::future::TimeoutFuture::new(dividend_per_share::DIVIDEND_RETRY_DELAY_MS)
                .await;
        }
    }
}

#[component]
fn AssetBadge() -> impl IntoView {
    view! { <Badge variant=BadgeVariant::Positive>{labels::DIVIDEND_HELD_BADGE}</Badge> }
}

#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Leptos component props are owned by the view"
)]
pub(crate) fn DividendInfo(store: DividendInfoStore, totals: DividendTotals) -> impl IntoView {
    let asset = move || store.asset_balance.get();
    let per_share = move || store.per_share.get();
    let loading = move || store.per_share_loading.get();

    let investment = move || asset().map_or_default(|a| investment_amount(&a));
    let gross = totals.total_dividends_before_tax.to_f64().unwrap_or(0.0);
    let net = totals.total_net_amount_received.to_f64().unwrap_or(0.0);
    let gross_negative = totals.total_dividends_before_tax < Decimal::ZERO;
    let gross_positive = totals.total_dividends_before_tax > Decimal::ZERO;
    let tax_negative = totals.total_taxes < Decimal::ZERO;
    let tax_positive = totals.total_taxes > Decimal::ZERO;
    let net_negative = totals.total_net_amount_received < Decimal::ZERO;
    let net_positive = totals.total_net_amount_received > Decimal::ZERO;
    let gross_rate = move || dividend_rate(gross, investment());
    let net_rate = move || dividend_rate(net, investment());

    let average_price_text = move || {
        asset().map_or_else(
            || labels::MISSING.to_string(),
            |a| format_currency(a.average_purchase_price),
        )
    };
    let shares_text = move || {
        asset().map_or_else(
            || labels::MISSING.to_string(),
            |a| format_number(a.shares, 2),
        )
    };
    let per_share_text = move || per_share_display(per_share(), loading());

    view! {
        <div>
            <DividendStatGrid>
                <DividendStatCard
                    label=labels::AVERAGE_PRICE
                    title_hint=true
                    hint=Some(Signal::derive(move || asset().is_none().then_some(labels::DIVIDEND_BALANCE_HINT)))
                >
                    <Amount text=Signal::derive(average_price_text) />
                </DividendStatCard>
                <DividendStatCard
                    label=labels::QUANTITY
                    title_hint=true
                    hint=Some(Signal::derive(move || asset().is_none().then_some(labels::DIVIDEND_BALANCE_HINT)))
                >
                    {shares_text}
                </DividendStatCard>
                <DividendStatCard
                    label=labels::PER_SHARE_DIVIDEND
                    hint=Some(Signal::derive(move || {
                        (!loading() && per_share().is_none()).then_some(labels::DIVIDEND_AUTO_HINT)
                    }))
                >
                    <Amount text=Signal::derive(per_share_text) />
                </DividendStatCard>
            </DividendStatGrid>
            <DividendTotalGrid>
                <DividendTotalCard
                    label=labels::DIVIDEND_AMOUNT_AND_YIELD
                    text=settings::format_amount(totals.total_dividends_before_tax)
                    negative=gross_negative
                    positive=gross_positive
                    rate=Signal::derive(move || {
                        let rate = gross_rate();
                        (rate != 0.0)
                            .then(|| (
                                format::format_percentage_value(rate),
                                rate.is_sign_negative(),
                                rate.is_sign_positive(),
                            ))
                    })
                />
                <DividendTotalCard
                    label=labels::TAX
                    text=settings::format_amount(totals.total_taxes)
                    negative=tax_negative
                    positive=tax_positive
                    rate=Signal::derive(|| None)
                />
                <DividendTotalCard
                    label=labels::AFTER_TAX_AND_YIELD
                    text=settings::format_amount(totals.total_net_amount_received)
                    negative=net_negative
                    positive=net_positive
                    rate=Signal::derive(move || {
                        let rate = net_rate();
                        (rate != 0.0)
                            .then(|| (
                                format::format_percentage_value(rate),
                                rate.is_sign_negative(),
                                rate.is_sign_positive(),
                            ))
                    })
                />
            </DividendTotalGrid>
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
    let title = labels::DIVIDEND_SUMMARY_TITLE;
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
                    on_toggle=move || expanded.update(bool::toggle)
                >
                    <DividendSummaryTitle>
                        <DividendSummaryHeading title=title />
                        {move || store.asset_balance.get().is_some().then(AssetBadge)}
                    </DividendSummaryTitle>
                </DisclosureToggle>
                <DividendSummaryBody expanded=expanded>
                    <DividendInfo store=store totals=totals.clone() />
                </DividendSummaryBody>
            </div>
        </Card>
    }
}

#[cfg(test)]
mod tests;
