use super::palette::{holding_band_class, holding_bar_class};
use super::summary::ChartItem;
use crate::features::asset_balance::format::{
    format_currency, format_fixed_percent, format_profit_loss, format_signed_valuation_amount,
    format_signed_valuation_rate, format_valuation_amount, is_negative_valuation,
};
use crate::features::asset_balance::holdings::{
    format_dividend_annual, format_dividend_per_share, format_dividend_yield, holding_dividend,
};
use crate::features::asset_balance::model::{
    ValuationInput, format_number_value, holding_valuation,
};
use crate::features::dividend_per_share::DividendMaps;
use crate::ui::amount::Amount;
use crate::ui::badge::CodeBadge;
use crate::ui::card::{Card, CardVariant};
use crate::ui::disclosure::{ChevronIcon, DisclosureStyle, DisclosureToggle};
use crate::ui::security_link::{SecurityCodeLink, SecurityLinkVariant};
use crate::ui::stat_row::StatRow;
use leptos::prelude::*;

#[component]
pub(crate) fn HoldingCard(
    item: ChartItem,
    index: usize,
    dividends: RwSignal<DividendMaps>,
) -> impl IntoView {
    let band_class = holding_band_class(index);
    let bar_class = holding_bar_class(index);
    let code = item.view.code.clone();
    let shares = item.view.shares;
    let average_price = item.view.average_price;
    let dividend =
        Memo::new(move |_| holding_dividend(&code, shares, average_price, &dividends.get()));
    let percentage_text = item.percentage.map_or("—".to_string(), |percentage| {
        format_fixed_percent(percentage, 1)
    });
    let bar_width = item.percentage.map_or("0%".to_string(), |percentage| {
        format!("{}%", percentage.min(100.0))
    });
    let valuation = holding_valuation(&ValuationInput::from(&item.view));
    let market_text = format_valuation_amount(valuation.market_value);
    let pnl_text = format_signed_valuation_amount(valuation.profit_loss.amount);
    let pnl_rate_text = format_signed_valuation_rate(valuation.profit_loss.rate, 1);
    let pnl_negative = is_negative_valuation(valuation.profit_loss.amount);
    // 基準日があれば「YYYY/M/D時点」、無ければ「—」
    let valuation_note = item.view.price_as_of.as_ref().map_or_else(
        || "—".to_string(),
        |date| format!("{}時点", date.replace('-', "/")),
    );
    let dividend_class = move |present: bool| {
        if present {
            "mt-0.5 truncate text-xs font-semibold text-text"
        } else {
            "mt-0.5 truncate text-xs font-semibold text-text-subtle"
        }
    };
    view! {
        <Card variant=CardVariant::Holding class=format!("border-l-4 {band_class} px-3.5 py-3 max-sm:hidden")>
            <div class="flex items-start justify-between gap-3">
                <div class="min-w-0 flex-1">
                    <div class="flex min-w-0 items-center gap-2" data-testid="portfolio-card-identity">
                        <CodeBadge flush=true testid="portfolio-card-code">
                            <SecurityCodeLink
                                value=item.view.code.clone()
                                class="font-semibold no-underline hover:underline".to_string()
                            />
                        </CodeBadge>
                        <p class="line-clamp-2 text-base font-semibold text-text" title=item.view.name.clone()>
                            {item.view.name.clone()}
                        </p>
                    </div>
                </div>
            </div>

            <div class="mt-2.5 flex items-center gap-2">
                <div class="h-2 flex-1 rounded-full bg-surface-raised">
                    <div
                        class=format!("h-full rounded-full transition-all duration-300 {bar_class}")
                        style=format!("width: {bar_width}")
                        data-testid="portfolio-card-composition-bar"
                    />
                </div>
                <span class="shrink-0 text-xs font-medium tabular-nums text-text-subtle">
                    "構成比 "
                    {percentage_text}
                </span>
            </div>

            <Card
                variant=CardVariant::Strip
                class="mt-2 grid grid-cols-2"
                testid="portfolio-card-valuation"
            >
                <div class="min-w-0 px-2 py-2">
                    <p class="truncate text-xs font-medium text-text-subtle">"評価額"</p>
                    <p
                        class="mt-0.5 truncate text-xs font-semibold text-text"
                        title=market_text.clone()
                    >
                        {market_text.clone()}
                    </p>
                    <p class="mt-0.5 truncate text-xs text-text-subtle">{valuation_note}</p>
                </div>
                <div class="min-w-0 border-l border-border-subtle/80 px-2 py-2">
                    <p class="truncate text-xs font-medium text-text-subtle">"含み損益"</p>
                    <Amount
                        block=true
                        class="mt-0.5 truncate text-xs font-semibold text-text"
                        text=pnl_text
                        negative=pnl_negative
                    />
                    <Amount
                        block=true
                        class="mt-0.5 truncate text-xs text-text-subtle"
                        text=pnl_rate_text
                        negative=pnl_negative
                    />
                </div>
            </Card>

            <Card
                variant=CardVariant::Strip
                class="mt-2 grid grid-cols-2"
                testid="portfolio-card-acquisition-stats"
            >
                <div class="min-w-0 px-2 py-2">
                    <p class="truncate text-xs font-medium text-text-subtle">"取得総額"</p>
                    <p
                        class="mt-0.5 truncate text-xs font-semibold text-text"
                        title=format_currency(item.view.purchase)
                    >
                        {format_currency(item.view.purchase)}
                    </p>
                </div>
                <div class="min-w-0 border-l border-border-subtle/80 px-2 py-2">
                    <p class="truncate text-xs font-medium text-text-subtle">"取得単価"</p>
                    <p
                        class="mt-0.5 truncate text-xs font-semibold text-text"
                        title=format_currency(item.view.average_price)
                    >
                        {format_currency(item.view.average_price)}
                    </p>
                </div>
                <div class="min-w-0 border-t border-border-faint px-2 py-2">
                    <p class="truncate text-xs font-medium text-text-subtle">"数量"</p>
                    <p
                        class="mt-0.5 truncate text-xs font-semibold text-text"
                        title=format!("{}株", format_number_value(item.view.shares))
                    >
                        {format!("{}株", format_number_value(item.view.shares))}
                    </p>
                </div>
                <div class="min-w-0 border-l border-t border-l-border-subtle/80 border-t-border-faint px-2 py-2 text-xs text-text-muted">
                    <p class="truncate text-xs font-medium text-text-subtle">"1株配当"</p>
                    <p
                        class=move || dividend_class(dividend.with(|d| d.per_share.is_some()))
                        title=move || dividend.with(format_dividend_per_share)
                    >
                        {move || dividend.with(format_dividend_per_share)}
                    </p>
                </div>
                <div class="min-w-0 border-t border-border-faint px-2 py-2 text-xs text-text-muted">
                    <p class="truncate text-xs font-medium text-text-subtle">"年間配当"</p>
                    <p
                        class=move || dividend_class(dividend.with(|d| d.annual.is_some()))
                        title=move || dividend.with(format_dividend_annual)
                    >
                        {move || dividend.with(format_dividend_annual)}
                    </p>
                </div>
                <div class="min-w-0 border-l border-t border-l-border-subtle/80 border-t-border-faint px-2 py-2 text-xs text-text-muted">
                    <p class="truncate text-xs font-medium text-text-subtle">"配当利回り"</p>
                    <p
                        class=move || dividend_class(dividend.with(|d| d.yield_value.is_some()))
                        title=move || dividend.with(format_dividend_yield)
                    >
                        {move || dividend.with(format_dividend_yield)}
                    </p>
                </div>
            </Card>
        </Card>
    }
}

#[component]
pub(crate) fn HoldingMobileCard(
    item: ChartItem,
    index: usize,
    dividends: RwSignal<DividendMaps>,
) -> impl IntoView {
    let open = RwSignal::new(false);
    let detail_id = format!("portfolio-item-detail-{}", item.view.code);
    let valuation = holding_valuation(&ValuationInput::from(&item.view));
    let market_display = format_valuation_amount(valuation.market_value);
    let pnl_display = format_profit_loss(&valuation.profit_loss);
    let pnl_negative = is_negative_valuation(valuation.profit_loss.amount);
    let current_price_display = format_currency(item.view.current_price);
    // 基準日があれば「YYYY/M/D時点」、無ければ素のラベル(CSV取込時点かはサマリー側の注記が担う)
    let current_price_label = item.view.price_as_of.as_ref().map_or_else(
        || "現在値".to_string(),
        |date| format!("現在値（{}時点）", date.replace('-', "/")),
    );
    let composition = item.percentage.map_or("—".to_string(), |percentage| {
        format_fixed_percent(percentage, 1)
    });
    let name_text = item.view.name.clone();
    let code_text = item.view.code.clone();
    let band_class = holding_band_class(index);
    view! {
        <Card
            variant=CardVariant::Holding
            class=format!("border-l-4 {band_class} sm:hidden")
            testid="portfolio-holding-card"
        >
            <DisclosureToggle
                style=DisclosureStyle::AssetCard
                expanded=Signal::derive(move || open.get())
                controls=detail_id.clone()
                on_toggle=move || open.update(|value| *value = !*value)
            >
                <span class="flex min-w-0 items-center gap-2">
                    <CodeBadge flush=true testid="portfolio-holding-card-code">
                        {code_text}
                    </CodeBadge>
                    <span class="min-w-0 flex-1 break-words text-base font-semibold text-text">
                        {name_text}
                    </span>
                </span>
                <span class="mt-2 flex items-center justify-between gap-2">
                    <span class="shrink-0 text-xs font-medium text-text-subtle">"評価額"</span>
                    <span class="flex min-w-0 items-center gap-1">
                        <Amount
                            class="truncate text-base font-bold text-text"
                            text=market_display
                        />
                        <ChevronIcon
                            expanded=Signal::derive(move || open.get())
                            class="h-4 w-4 shrink-0 text-text-subtle transition-transform duration-200"
                        />
                    </span>
                </span>
                <span class="mt-1 flex items-center justify-end gap-2">
                    <Amount
                        class="truncate text-xs font-semibold text-text"
                        text=pnl_display
                        negative=pnl_negative
                    />
                </span>
            </DisclosureToggle>
            {move || {
                let dividend = holding_dividend(
                    &item.view.code,
                    item.view.shares,
                    item.view.average_price,
                    &dividends.get(),
                );
                // コンポーネントの children は FnOnce で move 捕捉されるため、
                // 反応的に呼ばれるこの中で使う値は呼び出しごとに作り直す
                let average_price = format_currency(item.view.average_price);
                let purchase = format_currency(item.view.purchase);
                let shares = format!("{}株", format_number_value(item.view.shares));
                let current_price = current_price_display.clone();
                let composition_text = composition.clone();
                let dividend_annual = format_dividend_annual(&dividend);
                let dividend_per_share = format_dividend_per_share(&dividend);
                let dividend_yield = format_dividend_yield(&dividend);
                open.get()
                    .then(|| {
                        view! {
                            <div id=detail_id.clone() class="border-t border-ink/10 px-3.5 py-3">
                                <dl class="space-y-1.5">
                                    <StatRow label="取得総額" numeric=true>
                                        {purchase}
                                    </StatRow>
                                    <StatRow label="取得単価" numeric=true>
                                        {average_price}
                                    </StatRow>
                                    <StatRow label="数量" numeric=true>
                                        {shares}
                                    </StatRow>
                                    <StatRow label=current_price_label.clone() numeric=true>
                                        {current_price}
                                    </StatRow>
                                    <StatRow label="取得額構成比" numeric=true>
                                        {composition_text}
                                    </StatRow>
                                    <StatRow label="予想年間配当">
                                        {dividend_annual}
                                    </StatRow>
                                    <StatRow label="1株配当">
                                        {dividend_per_share}
                                    </StatRow>
                                    <StatRow label="取得額基準利回り">
                                        {dividend_yield}
                                    </StatRow>
                                </dl>
                                <p class="mt-2.5 border-t border-border-faint pt-2.5 text-xs">
                                    <SecurityCodeLink
                                        value=item.view.code.clone()
                                        variant=SecurityLinkVariant::Details
                                        class="text-xs".to_string()
                                    />
                                </p>
                            </div>
                        }
                    })
            }}
        </Card>
    }
}
