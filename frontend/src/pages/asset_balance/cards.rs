use super::format::{
    format_currency, format_fixed_percent, format_valuation_amount, format_valuation_rate,
};
use super::holdings::{
    format_dividend_annual, format_dividend_per_share, format_dividend_yield, holding_dividend,
};
use super::summary::ChartItem;
use crate::asset_balance_domain::{calculate_valuation, format_number_value};
use crate::components::security_link::SecurityCodeLink;
use crate::dividend_per_share::DividendMaps;
use leptos::prelude::*;

const CHART_COLORS: [&str; 10] = [
    "#3b82f6", "#10b981", "#f59e0b", "#ef4444", "#8b5cf6", "#ec4899", "#06b6d4", "#f97316",
    "#84cc16", "#6366f1",
];

#[component]
pub(crate) fn HoldingCard(
    item: ChartItem,
    index: usize,
    dividends: RwSignal<DividendMaps>,
) -> impl IntoView {
    let color = CHART_COLORS[index % CHART_COLORS.len()];
    let code = item.view.code.clone();
    let shares = item.view.shares;
    let average_price = item.view.average_price;
    let dividend =
        Memo::new(move |_| holding_dividend(&code, shares, average_price, &dividends.get()));
    let percentage_text = item.percentage.map_or("-".to_string(), |percentage| {
        format_fixed_percent(percentage, 1)
    });
    let bar_width = item.percentage.map_or("NaN%".to_string(), |percentage| {
        format!("{}%", percentage.min(100.0))
    });
    let dividend_class = move |present: bool| {
        if present {
            "mt-0.5 truncate text-[12px] font-semibold text-emerald-600"
        } else {
            "mt-0.5 truncate text-[12px] font-semibold text-slate-500"
        }
    };
    view! {
        <div class="rounded-lg border border-slate-950/10 bg-white px-3.5 py-3 shadow-sm max-sm:hidden">
            <div class="flex items-start justify-between gap-3">
                <div class="min-w-0 flex-1">
                    <div class="flex items-center gap-2.5 min-w-0">
                        <span class="h-3 w-3 shrink-0 rounded-sm" style=format!("background-color: {color}") />
                        <div class="flex min-w-0 items-center gap-2" data-testid="portfolio-card-identity">
                            <span
                                class="inline-flex shrink-0 items-center rounded-full bg-blue-50 px-2 py-0.5"
                                data-testid="portfolio-card-code"
                            >
                                <SecurityCodeLink
                                    value=item.view.code.clone()
                                    class="text-[11px] font-semibold tracking-[0.16em] no-underline hover:underline"
                                        .to_string()
                                />
                            </span>
                            <p class="truncate text-[15px] font-semibold text-slate-800" title=item.view.name.clone()>
                                {item.view.name.clone()}
                            </p>
                        </div>
                    </div>
                </div>
                <div class="shrink-0 text-right">
                    <p class="text-xl font-bold text-slate-800">{percentage_text}</p>
                </div>
            </div>

            <div class="mt-2.5 h-2 w-full rounded-full bg-slate-100 max-sm:hidden">
                <div
                    class="h-full rounded-full transition-all duration-300"
                    style=format!("width: {bar_width}; background-color: {color}")
                />
            </div>

            <div
                class="mt-2 grid grid-cols-3 overflow-hidden rounded-md bg-slate-50"
                data-testid="portfolio-card-acquisition-stats"
            >
                <div class="min-w-0 px-2 py-2">
                    <p class="truncate text-[10px] font-medium text-slate-500">"取得総額"</p>
                    <p
                        class="mt-0.5 truncate text-[12px] font-semibold text-slate-800"
                        title=format_currency(item.view.purchase)
                    >
                        {format_currency(item.view.purchase)}
                    </p>
                </div>
                <div class="min-w-0 border-l border-slate-200/80 px-2 py-2">
                    <p class="truncate text-[10px] font-medium text-slate-500">"取得単価"</p>
                    <p
                        class="mt-0.5 truncate text-[12px] font-semibold text-slate-800"
                        title=format_currency(item.view.average_price)
                    >
                        {format_currency(item.view.average_price)}
                    </p>
                </div>
                <div class="min-w-0 border-l border-slate-200/80 px-2 py-2">
                    <p class="truncate text-[10px] font-medium text-slate-500">"数量"</p>
                    <p
                        class="mt-0.5 truncate text-[12px] font-semibold text-slate-800"
                        title=format!("{}株", format_number_value(item.view.shares))
                    >
                        {format!("{}株", format_number_value(item.view.shares))}
                    </p>
                </div>
            </div>

            <div class="mt-2 grid grid-cols-3 overflow-hidden rounded-md bg-emerald-50/55">
                <div class="min-w-0 px-2 py-2 text-xs text-slate-600">
                    <p class="truncate text-[10px] font-medium text-slate-500">"1株配当"</p>
                    <p
                        class=move || dividend_class(dividend.with(|d| d.per_share.is_some()))
                        title=move || dividend.with(format_dividend_per_share)
                    >
                        {move || dividend.with(format_dividend_per_share)}
                    </p>
                </div>
                <div class="min-w-0 border-l border-emerald-100/80 px-2 py-2 text-xs text-slate-600">
                    <p class="truncate text-[10px] font-medium text-slate-500">"年間配当"</p>
                    <p
                        class=move || dividend_class(dividend.with(|d| d.annual.is_some()))
                        title=move || dividend.with(format_dividend_annual)
                    >
                        {move || dividend.with(format_dividend_annual)}
                    </p>
                </div>
                <div class="min-w-0 border-l border-emerald-100/80 px-2 py-2 text-xs text-slate-600">
                    <p class="truncate text-[10px] font-medium text-slate-500">"配当利回り"</p>
                    <p
                        class=move || dividend_class(dividend.with(|d| d.yield_value.is_some()))
                        title=move || dividend.with(format_dividend_yield)
                    >
                        {move || dividend.with(format_dividend_yield)}
                    </p>
                </div>
            </div>
        </div>
    }
}

#[component]
pub(crate) fn HoldingValuationCard(
    item: ChartItem,
    dividends: RwSignal<DividendMaps>,
) -> impl IntoView {
    let open = RwSignal::new(false);
    let detail_id = format!("portfolio-item-detail-{}", item.view.code);
    let valuation = calculate_valuation(
        &serde_json::json!(item.view.market),
        &serde_json::json!(item.view.purchase),
    );
    let market_display = format_currency(item.view.market);
    let current_price_display = format_currency(item.view.current_price);
    let profit_loss = match valuation.amount {
        None => "—".to_string(),
        Some(amount) => match valuation.rate {
            None => format!("{}（算出不可）", format_valuation_amount(Some(amount))),
            Some(rate) => {
                format!(
                    "{}（{}）",
                    format_valuation_amount(Some(amount)),
                    format_valuation_rate(Some(rate), 1),
                )
            }
        },
    };
    let composition = item.percentage.map_or("—".to_string(), |percentage| {
        format_fixed_percent(percentage, 1)
    });
    view! {
        <div class="rounded-lg border border-slate-950/10 bg-white shadow-sm sm:hidden" data-testid="portfolio-valuation-card">
            <button
                type="button"
                aria-expanded=move || open.get()
                aria-controls=detail_id.clone()
                on:click=move |_| open.update(|value| *value = !*value)
                class="block min-h-[44px] w-full px-3.5 py-4 text-left"
            >
                <span class="flex min-w-0 items-center gap-2">
                    <span class="min-w-0 flex-1 truncate text-[15px] font-semibold text-slate-800">
                        {item.view.name.clone()}
                    </span>
                    <span
                        class="code-badge"
                        data-testid="portfolio-valuation-card-code"
                    >
                        {item.view.code.clone()}
                    </span>
                </span>
                <span class="mt-2 flex items-baseline justify-between gap-2">
                    <span class="shrink-0 text-xs font-medium text-slate-500">"評価額"</span>
                    <span class="truncate text-base font-bold tabular-nums text-slate-800">
                        {market_display}
                    </span>
                </span>
                <span class="mt-2 flex items-center justify-between gap-2">
                    <span class="shrink-0 text-xs font-medium text-slate-500">"評価損益"</span>
                    <span class="flex min-w-0 items-center gap-1">
                        <span
                            class=format!(
                                "truncate text-sm font-bold tabular-nums {}",
                                if valuation.amount.is_some_and(|amount| amount < 0.0) {
                                    "text-red-800"
                                } else {
                                    "text-slate-800"
                                },
                            )
                            data-negative=valuation
                                .amount
                                .is_some_and(|amount| amount < 0.0)
                                .then_some("true")
                        >
                            {profit_loss}
                        </span>
                        <span aria-hidden="true" class="shrink-0 text-xs text-slate-400">
                            {move || if open.get() { "▴" } else { "▾" }}
                        </span>
                    </span>
                </span>
            </button>
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
                            <div id=detail_id.clone() class="border-t border-slate-950/10 px-3.5 py-3">
                                <dl class="space-y-1.5 text-xs text-slate-600">
                                    <div class="flex items-start justify-between gap-2">
                                        <dt class="shrink-0 font-medium text-slate-500">"銘柄名"</dt>
                                        <dd class="min-w-0 break-words text-right font-semibold text-slate-800">
                                            {item.view.name.clone()}
                                        </dd>
                                    </div>
                                    <div class="flex items-center justify-between gap-2">
                                        <dt class="shrink-0 font-medium text-slate-500">"取得総額"</dt>
                                        <dd class="truncate font-semibold tabular-nums text-slate-800">
                                            {format_currency(item.view.purchase)}
                                        </dd>
                                    </div>
                                    <div class="flex items-center justify-between gap-2">
                                        <dt class="shrink-0 font-medium text-slate-500">"取得単価"</dt>
                                        <dd class="truncate font-semibold tabular-nums text-slate-800">
                                            {format_currency(item.view.average_price)}
                                        </dd>
                                    </div>
                                    <div class="flex items-center justify-between gap-2">
                                        <dt class="shrink-0 font-medium text-slate-500">"数量"</dt>
                                        <dd class="truncate font-semibold tabular-nums text-slate-800">
                                            {format!("{}株", format_number_value(item.view.shares))}
                                        </dd>
                                    </div>
                                    <div class="flex items-center justify-between gap-2">
                                        <dt class="shrink-0 font-medium text-slate-500">"現在値"</dt>
                                        <dd class="truncate font-semibold tabular-nums text-slate-800">
                                            {current_price_display.clone()}
                                        </dd>
                                    </div>
                                    <div class="flex items-center justify-between gap-2">
                                        <dt class="shrink-0 font-medium text-slate-500">"取得額構成比"</dt>
                                        <dd class="truncate font-semibold tabular-nums text-slate-800">
                                            {composition.clone()}
                                        </dd>
                                    </div>
                                    <div class="flex items-center justify-between gap-2">
                                        <dt class="shrink-0 font-medium text-slate-500">"予想年間配当"</dt>
                                        <dd class="truncate font-semibold text-emerald-600">
                                            {format_dividend_annual(&dividend)}
                                        </dd>
                                    </div>
                                    <div class="flex items-center justify-between gap-2">
                                        <dt class="shrink-0 font-medium text-slate-500">"1株配当"</dt>
                                        <dd class="truncate font-semibold text-emerald-600">
                                            {format_dividend_per_share(&dividend)}
                                        </dd>
                                    </div>
                                    <div class="flex items-center justify-between gap-2">
                                        <dt class="shrink-0 font-medium text-slate-500">"取得額基準利回り"</dt>
                                        <dd class="truncate font-semibold text-emerald-600">
                                            {format_dividend_yield(&dividend)}
                                        </dd>
                                    </div>
                                </dl>
                                <p class="mt-2.5 border-t border-slate-100 pt-2.5 text-xs">
                                    <SecurityCodeLink
                                        value=item.view.code.clone()
                                        class="text-xs".to_string()
                                    />
                                    <span class="ml-1 text-slate-500">"の銘柄情報を見る"</span>
                                </p>
                            </div>
                        }
                    })
            }}
        </div>
    }
}
