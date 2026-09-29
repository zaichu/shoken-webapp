use super::palette::{holding_band_class, holding_bar_class};
use super::summary::ChartItem;
use crate::features::asset_balance::format::{
    format_currency, format_fixed_percent, format_valuation_amount, format_valuation_rate,
    valuation_tone,
};
use crate::features::asset_balance::holdings::{
    format_dividend_annual, format_dividend_per_share, format_dividend_yield, holding_dividend,
};
use crate::features::asset_balance::model::{
    calculate_valuation_from_decimal, format_number_value,
};
use crate::features::dividend_per_share::DividendMaps;
use crate::ui::amount::Amount;
use crate::ui::badge::CodeBadge;
use crate::ui::card::{Card, CardVariant};
use crate::ui::disclosure::{DisclosureStyle, DisclosureToggle};
use crate::ui::security_link::SecurityCodeLink;
use leptos::prelude::*;

struct ValuationDisplay {
    market: String,
    profit_loss: String,
    class: &'static str,
    negative: Option<&'static str>,
}

fn valuation_display(item: &ChartItem) -> ValuationDisplay {
    let valuation = calculate_valuation_from_decimal(item.view.market_dec, item.view.purchase_dec);
    let (class, negative) = valuation_tone(valuation.amount);
    let profit_loss = match valuation.amount {
        None => "—".to_string(),
        Some(amount) => match valuation.rate {
            None => format!("{}（算出不可）", format_valuation_amount(Some(amount))),
            Some(rate) => format!(
                "{}（{}）",
                format_valuation_amount(Some(amount)),
                format_valuation_rate(Some(rate), 1),
            ),
        },
    };
    ValuationDisplay {
        market: format_currency(item.view.market),
        profit_loss,
        class,
        negative,
    }
}

#[component]
pub(crate) fn HoldingCard(
    item: ChartItem,
    index: usize,
    dividends: RwSignal<DividendMaps>,
) -> impl IntoView {
    let band_class = holding_band_class(index);
    let bar_class = holding_bar_class(index);
    let valuation = valuation_display(&item);
    let code = item.view.code.clone();
    let shares = item.view.shares;
    let average_price = item.view.average_price;
    let dividend =
        Memo::new(move |_| holding_dividend(&code, shares, average_price, &dividends.get()));
    let percentage_text = item.percentage.map_or("—".to_string(), |percentage| {
        format_fixed_percent(percentage, 1)
    });
    let bar_width = item.percentage.map_or("NaN%".to_string(), |percentage| {
        format!("{}%", percentage.min(100.0))
    });
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

            <div class="mt-2 flex flex-wrap items-start justify-between gap-x-3 gap-y-1" data-testid="portfolio-card-valuation">
                <div>
                    <p class="text-xs font-medium text-text-subtle">"評価額"</p>
                    <Amount
                        block=true
                        text=valuation.market
                        class="whitespace-nowrap text-lg font-bold text-ink"
                    />
                </div>
                <div class="ml-auto text-right">
                    <p class="text-xs font-medium text-text-subtle">"評価損益"</p>
                    <Amount
                        block=true
                        text=valuation.profit_loss
                        class=format!("whitespace-nowrap text-sm font-bold {}", valuation.class)
                        negative=valuation.negative.is_some()
                    />
                </div>
            </div>

            <div class="mt-2.5 flex items-center gap-2">
                <div class="h-2 flex-1 rounded-full bg-surface-raised">
                    <div
                        class=format!("h-full rounded-full transition-all duration-300 {bar_class}")
                        style=format!("width: {bar_width}")
                    />
                </div>
                <span class="shrink-0 text-xs font-medium tabular-nums text-text-subtle">
                    "構成比 "
                    {percentage_text}
                </span>
            </div>

            <Card
                variant=CardVariant::Strip
                class="mt-2 grid grid-cols-3"
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
                <div class="min-w-0 border-l border-border-subtle/80 px-2 py-2">
                    <p class="truncate text-xs font-medium text-text-subtle">"数量"</p>
                    <p
                        class="mt-0.5 truncate text-xs font-semibold text-text"
                        title=format!("{}株", format_number_value(item.view.shares))
                    >
                        {format!("{}株", format_number_value(item.view.shares))}
                    </p>
                </div>
            </Card>

            <Card variant=CardVariant::Strip class="mt-2 grid grid-cols-3">
                <div class="min-w-0 px-2 py-2 text-xs text-text-muted">
                    <p class="truncate text-xs font-medium text-text-subtle">"1株配当"</p>
                    <p
                        class=move || dividend_class(dividend.with(|d| d.per_share.is_some()))
                        title=move || dividend.with(format_dividend_per_share)
                    >
                        {move || dividend.with(format_dividend_per_share)}
                    </p>
                </div>
                <div class="min-w-0 border-l border-border-subtle px-2 py-2 text-xs text-text-muted">
                    <p class="truncate text-xs font-medium text-text-subtle">"年間配当"</p>
                    <p
                        class=move || dividend_class(dividend.with(|d| d.annual.is_some()))
                        title=move || dividend.with(format_dividend_annual)
                    >
                        {move || dividend.with(format_dividend_annual)}
                    </p>
                </div>
                <div class="min-w-0 border-l border-border-subtle px-2 py-2 text-xs text-text-muted">
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
pub(crate) fn HoldingValuationCard(
    item: ChartItem,
    index: usize,
    dividends: RwSignal<DividendMaps>,
) -> impl IntoView {
    let open = RwSignal::new(false);
    let detail_id = format!("portfolio-item-detail-{}", item.view.code);
    let ValuationDisplay {
        market: market_display,
        profit_loss,
        class: valuation_class,
        negative: valuation_negative,
    } = valuation_display(&item);
    let current_price_display = format_currency(item.view.current_price);
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
            testid="portfolio-valuation-card"
        >
            <DisclosureToggle
                style=DisclosureStyle::AssetCard
                expanded=Signal::derive(move || open.get())
                controls=detail_id.clone()
                on_toggle=move || open.update(|value| *value = !*value)
            >
                <span class="flex min-w-0 items-center gap-2">
                    <span class="min-w-0 flex-1 truncate text-base font-semibold text-text">
                        {name_text}
                    </span>
                    <CodeBadge flush=true testid="portfolio-valuation-card-code">
                        {code_text}
                    </CodeBadge>
                </span>
                <span class="mt-2 flex items-baseline justify-between gap-2">
                    <span class="shrink-0 text-xs font-medium text-text-subtle">"評価額"</span>
                    <Amount
                        class="truncate text-base font-bold text-text"
                        text=market_display
                    />
                </span>
                <span class="mt-2 flex items-center justify-between gap-2">
                    <span class="shrink-0 text-xs font-medium text-text-subtle">"評価損益"</span>
                    <span class="flex min-w-0 items-center gap-1">
                        <Amount
                            class=format!("truncate text-sm font-bold {valuation_class}")
                            negative=valuation_negative.is_some()
                            text=profit_loss
                        />
                        <span aria-hidden="true" class="shrink-0 text-xs text-text-faint">
                            {move || if open.get() { "▴" } else { "▾" }}
                        </span>
                    </span>
                </span>
            </DisclosureToggle>
            {move || {
                let dividend = holding_dividend(
                    &item.view.code,
                    item.view.shares,
                    item.view.average_price,
                    &dividends.get(),
                );
                open.get()
                    .then(|| {
                        view! {
                            <div id=detail_id.clone() class="border-t border-ink/10 px-3.5 py-3">
                                <dl class="space-y-1.5 text-xs text-text-muted">
                                    <div class="flex items-start justify-between gap-2">
                                        <dt class="shrink-0 font-medium text-text-subtle">"銘柄名"</dt>
                                        <dd class="min-w-0 break-words text-right font-semibold text-text">
                                            {item.view.name.clone()}
                                        </dd>
                                    </div>
                                    <div class="flex items-center justify-between gap-2">
                                        <dt class="shrink-0 font-medium text-text-subtle">"取得総額"</dt>
                                        <dd class="truncate font-semibold tabular-nums text-text">
                                            {format_currency(item.view.purchase)}
                                        </dd>
                                    </div>
                                    <div class="flex items-center justify-between gap-2">
                                        <dt class="shrink-0 font-medium text-text-subtle">"取得単価"</dt>
                                        <dd class="truncate font-semibold tabular-nums text-text">
                                            {format_currency(item.view.average_price)}
                                        </dd>
                                    </div>
                                    <div class="flex items-center justify-between gap-2">
                                        <dt class="shrink-0 font-medium text-text-subtle">"数量"</dt>
                                        <dd class="truncate font-semibold tabular-nums text-text">
                                            {format!("{}株", format_number_value(item.view.shares))}
                                        </dd>
                                    </div>
                                    <div class="flex items-center justify-between gap-2">
                                        <dt class="shrink-0 font-medium text-text-subtle">"現在値"</dt>
                                        <dd class="truncate font-semibold tabular-nums text-text">
                                            {current_price_display.clone()}
                                        </dd>
                                    </div>
                                    <div class="flex items-center justify-between gap-2">
                                        <dt class="shrink-0 font-medium text-text-subtle">"取得額構成比"</dt>
                                        <dd class="truncate font-semibold tabular-nums text-text">
                                            {composition.clone()}
                                        </dd>
                                    </div>
                                    <div class="flex items-center justify-between gap-2">
                                        <dt class="shrink-0 font-medium text-text-subtle">"予想年間配当"</dt>
                                        <dd class="truncate font-semibold text-text">
                                            {format_dividend_annual(&dividend)}
                                        </dd>
                                    </div>
                                    <div class="flex items-center justify-between gap-2">
                                        <dt class="shrink-0 font-medium text-text-subtle">"1株配当"</dt>
                                        <dd class="truncate font-semibold text-text">
                                            {format_dividend_per_share(&dividend)}
                                        </dd>
                                    </div>
                                    <div class="flex items-center justify-between gap-2">
                                        <dt class="shrink-0 font-medium text-text-subtle">"取得額基準利回り"</dt>
                                        <dd class="truncate font-semibold text-text">
                                            {format_dividend_yield(&dividend)}
                                        </dd>
                                    </div>
                                </dl>
                                <p class="mt-2.5 border-t border-border-faint pt-2.5 text-xs">
                                    <SecurityCodeLink
                                        value=item.view.code.clone()
                                        class="text-xs".to_string()
                                    />
                                    <span class="ml-1 text-text-subtle">"の銘柄情報を見る"</span>
                                </p>
                            </div>
                        }
                    })
            }}
        </Card>
    }
}
