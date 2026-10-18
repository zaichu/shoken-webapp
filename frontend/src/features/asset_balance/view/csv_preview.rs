use crate::features::asset_balance::csv::{AssetBalanceRow, AssetBalanceRowData};
use crate::features::asset_balance::format::{
    format_signed_valuation_amount, format_signed_valuation_rate, format_valuation_amount,
};
use crate::features::asset_balance::model::{ValuationInput, holding_valuation};
use crate::ui::button::{Button, ButtonVariant};
use crate::ui::card::{Card, CardVariant};
use leptos::prelude::*;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

const MOBILE_PREVIEW_LIMIT: usize = 5;

fn preview_row_data(
    row: &AssetBalanceRow,
) -> (
    String,
    String,
    String,
    String,
    String,
    String,
    Option<String>,
) {
    let shares_f64 = row.shares().to_f64().unwrap_or(0.0);
    let current_price_f64 = row.current_price().to_f64().unwrap_or(0.0);
    let total_purchase_f64 = row.total_purchase_amount().to_f64().unwrap_or(0.0);
    let valuation = holding_valuation(&ValuationInput {
        current_price: current_price_f64,
        shares: shares_f64,
        total_purchase_amount: total_purchase_f64,
    });
    let market_text = format_valuation_amount(valuation.market_value);
    let pnl_text = format_signed_valuation_amount(valuation.profit_loss.amount);
    let pnl_rate_text = valuation
        .profit_loss
        .rate
        .map(|v| format_signed_valuation_rate(Some(v), 1));
    let pnl_negative = valuation.profit_loss.amount.is_some_and(|v| v < 0.0);
    let pnl_positive = valuation.profit_loss.amount.is_some_and(|v| v > 0.0);
    let pnl_class = if pnl_negative {
        "text-negative".to_string()
    } else if pnl_positive {
        "text-positive".to_string()
    } else {
        "text-text-soft".to_string()
    };
    let security_name = row.security_name().to_string();
    let security_code = row.security_code().to_string();
    let shares_str = format!("{}株", format_number(row.shares()));
    (
        security_code,
        security_name,
        shares_str,
        market_text,
        pnl_class,
        pnl_text,
        pnl_rate_text,
    )
}

#[component]
pub fn AssetBalanceCsvPreview(rows: Vec<AssetBalanceRow>) -> impl IntoView {
    let total = rows.len();
    let rows_for_pc = rows.clone();
    let show_all = RwSignal::new(false);
    let mobile_rows = Memo::new(move |_| {
        if show_all.get() || total <= MOBILE_PREVIEW_LIMIT {
            rows.clone()
        } else {
            rows[..MOBILE_PREVIEW_LIMIT].to_vec()
        }
    });
    let has_more = Memo::new(move |_| total > MOBILE_PREVIEW_LIMIT && !show_all.get());

    let render_rows = move |rows: Vec<AssetBalanceRow>| {
        rows.into_iter().map(|row| {
            let (security_code, security_name, shares_str, market_text, pnl_class, pnl_text, pnl_rate_text) = preview_row_data(&row);
            view! {
                <Card variant=CardVariant::Item class="p-3">
                    <div class="flex items-baseline justify-between gap-2">
                        <span class="min-w-0 truncate font-medium text-text">{security_name}</span>
                        <span class="shrink-0 font-bold tabular-nums text-brand">{market_text}</span>
                    </div>
                    <div class="mt-1 flex items-baseline justify-between gap-2 text-xs">
                        <span class="shrink-0 text-text-muted">{security_code} " · " {shares_str}</span>
                        <span class=format!("flex shrink-0 items-baseline {}", pnl_class)>
                            {pnl_text}
                            {pnl_rate_text.map(|rate| view! { " " <span class="asset-badge">{rate}</span> })}
                        </span>
                    </div>
                </Card>
            }
        }).collect_view()
    };

    view! {
        <section data-testid="csv-preview">
            <h2 class="text-sm font-bold text-text">"プレビュー"</h2>
            <p class="mt-1 text-xs text-text-soft">
                "表は取り込むファイルの内容です。保存するまで登録済みのデータは変わりません。"
            </p>
            <div class="mt-2">
                // PC: テーブル表示
                <div class="hidden lg:block">
                    // 全行をそのまま置くと保存ボタンが遠くなるため、枠内スクロールにする
                    <div class="csv-preview-scroll max-h-96 overflow-y-auto">
                        <table class="receipts-table w-full">
                            <thead>
                            <tr class="text-text-soft font-semibold text-xs">
                                <th class="px-3 py-2 text-left">"銘柄コード"</th>
                                <th class="px-3 py-2 text-left">"銘柄名"</th>
                                <th class="px-3 py-2 text-right">"保有数量"</th>
                                <th class="px-3 py-2 text-right">"平均取得価額"</th>
                                <th class="px-3 py-2 text-right">"取得総額"</th>
                                <th class="px-3 py-2 text-right">"現在値"</th>
                                <th class="px-3 py-2 text-right">"時価評価額"</th>
                                <th class="px-3 py-2 text-right">"評価損益"</th>
                            </tr>
                        </thead>
                        <tbody>
                            {rows_for_pc.into_iter().map(|row| {
                                let (security_code, security_name, shares_str, market_text, pnl_class, pnl_text, pnl_rate_text) = preview_row_data(&row);
                                let avg_price_str = format!("¥{}", format_number(row.average_purchase_price()));
                                let total_purchase_str = format!("¥{}", format_number(row.total_purchase_amount()));
                                let current_price_str = format!("¥{}", format_number(row.current_price()));
                                view! {
                                    <tr class="border-t border-border">
                                        <td class="px-3 py-2 text-sm font-mono text-text">{security_code}</td>
                                        <td class="px-3 py-2 text-sm text-text">{security_name}</td>
                                        <td class="px-3 py-2 text-sm text-right tabular-nums text-text">{shares_str}</td>
                                        <td class="px-3 py-2 text-sm text-right tabular-nums text-text">{avg_price_str}</td>
                                        <td class="px-3 py-2 text-sm text-right tabular-nums text-text">{total_purchase_str}</td>
                                        <td class="px-3 py-2 text-sm text-right tabular-nums text-text">{current_price_str}</td>
                                        <td class="px-3 py-2 text-sm text-right tabular-nums text-brand">{market_text}</td>
                                        <td class=format!("px-3 py-2 text-sm text-right tabular-nums {}", pnl_class)>
                                            {pnl_text}
                                            {pnl_rate_text.map(|rate| view! { " " <span class="asset-badge">{rate}</span> })}
                                        </td>
                                    </tr>
                                }
                            }).collect_view()}
                        </tbody>
                        </table>
                    </div>
                </div>
                // Mobile: カード表示（先頭5件、クリックで全件表示）
                <div class="lg:hidden space-y-2">
                    {move || render_rows(mobile_rows.get())}
                    {move || has_more.get().then(|| {
                        view! {
                            <Button
                                variant=ButtonVariant::Ghost
                                class="block min-h-11 w-full text-center"
                                testid="csv-preview-show-all"
                                on_click=move |_| show_all.set(true)
                            >
                                {format!("残り{}件をすべて表示", total - MOBILE_PREVIEW_LIMIT)}
                            </Button>
                        }
                    })}
                </div>
            </div>
        </section>
    }
}

fn format_number(value: Decimal) -> String {
    let s = value.to_string();
    let (sign, abs_s) = if let Some(stripped) = s.strip_prefix('-') {
        ("-", stripped)
    } else {
        ("", s.as_str())
    };
    if abs_s.contains('.') {
        let parts: Vec<&str> = abs_s.split('.').collect();
        let int_part = parts[0];
        let dec_part = parts[1];
        let mut result = String::new();
        let chars: Vec<char> = int_part.chars().collect();
        for (i, ch) in chars.iter().rev().enumerate() {
            result.insert(0, *ch);
            if (i + 1) % 3 == 0 && i + 1 != chars.len() {
                result.insert(0, ',');
            }
        }
        format!("{}{}.{}", sign, result, dec_part)
    } else {
        let mut result = String::new();
        let chars: Vec<char> = abs_s.chars().collect();
        for (i, ch) in chars.iter().rev().enumerate() {
            result.insert(0, *ch);
            if (i + 1) % 3 == 0 && i + 1 != chars.len() {
                result.insert(0, ',');
            }
        }
        format!("{}{}", sign, result)
    }
}

#[cfg(test)]
mod tests;
