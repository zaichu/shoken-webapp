use crate::features::asset_balance::csv::{AssetBalanceRow, AssetBalanceRowData};
use crate::features::asset_balance::format::{
    format_signed_valuation_amount, format_signed_valuation_rate, format_valuation_amount,
};
use crate::features::asset_balance::model::{ValuationInput, holding_valuation};
use crate::ui::card::{Card, CardVariant};
use leptos::prelude::*;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

const MOBILE_PREVIEW_LIMIT: usize = 5;

#[component]
pub fn AssetBalanceCsvPreview(rows: Vec<AssetBalanceRow>) -> impl IntoView {
    let total = rows.len();
    let (mobile_rows, has_more) = if total > MOBILE_PREVIEW_LIMIT {
        (rows[..MOBILE_PREVIEW_LIMIT].to_vec(), true)
    } else {
        (rows.clone(), false)
    };

    view! {
        <section data-testid="csv-preview">
            <h2 class="text-sm font-bold text-text">"プレビュー"</h2>
            <p class="mt-1 text-xs text-text-soft">
                "表は取り込むファイルの内容です。保存するまで登録済みのデータは変わりません。"
            </p>
            <div class="mt-2">
                // PC: テーブル表示
                <div class="hidden lg:block overflow-x-auto">
                    <table class="receipts-table w-full">
                        <thead>
                            <tr class="bg-surface-raised text-text-soft font-semibold text-xs">
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
                            {rows.into_iter().map(|row| {
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
                                let pnl_rate_text = valuation.profit_loss.rate.map(|v| format_signed_valuation_rate(Some(v), 1));
                                let pnl_negative = valuation.profit_loss.amount.is_some_and(|v| v < 0.0);
                                let pnl_positive = valuation.profit_loss.amount.is_some_and(|v| v > 0.0);
                                let pnl_class = if pnl_negative { "text-negative" } else if pnl_positive { "text-positive" } else { "text-text-soft" };
                                let security_code = row.security_code().to_string();
                                let security_name = row.security_name().to_string();
                                let shares_str = format!("{}株", format_number(row.shares()));
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
                // Mobile: カード表示（先頭5件）
                <div class="lg:hidden space-y-2">
                    {mobile_rows.into_iter().map(|row| {
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
                        let pnl_rate_text = valuation.profit_loss.rate.map(|v| format_signed_valuation_rate(Some(v), 1));
                        let pnl_negative = valuation.profit_loss.amount.is_some_and(|v| v < 0.0);
                        let pnl_positive = valuation.profit_loss.amount.is_some_and(|v| v > 0.0);
                        let pnl_class = if pnl_negative { "text-negative" } else if pnl_positive { "text-positive" } else { "text-text-soft" };
                        let security_name = row.security_name().to_string();
                        let security_code = row.security_code().to_string();
                        let shares_str = format!("{}株", format_number(row.shares()));
                        view! {
                            <Card variant=CardVariant::Item class="p-3">
                                <div class="flex items-baseline justify-between gap-2">
                                    <span class="min-w-0 truncate font-medium text-text">{security_name}</span>
                                    <span class="shrink-0 font-bold tabular-nums text-brand">{market_text}</span>
                                </div>
                                <div class="mt-1 flex items-baseline justify-between gap-2 text-xs">
                                    <span class="shrink-0 text-text-muted">{security_code} &middot; {shares_str}</span>
                                    <span class=format!("flex shrink-0 items-baseline {}", pnl_class)>
                                        {pnl_text}
                                        {pnl_rate_text.map(|rate| view! { " " <span class="asset-badge">{rate}</span> })}
                                    </span>
                                </div>
                            </Card>
                        }
                    }).collect_view()}
                    {has_more.then(|| {
                        view! {
                            <a href="#" class="block text-center py-2 text-sm text-link underline" data-testid="csv-preview-show-all">
                                {format!("残り{}件をすべて表示", total - MOBILE_PREVIEW_LIMIT)}
                            </a>
                        }
                    })}
                </div>
            </div>
        </section>
    }
}

fn format_number(value: Decimal) -> String {
    let s = value.to_string();
    if s.contains('.') {
        let parts: Vec<&str> = s.split('.').collect();
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
        format!("{}.{}", result, dec_part)
    } else {
        let mut result = String::new();
        let chars: Vec<char> = s.chars().collect();
        for (i, ch) in chars.iter().rev().enumerate() {
            result.insert(0, *ch);
            if (i + 1) % 3 == 0 && i + 1 != chars.len() {
                result.insert(0, ',');
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(dec!(1000)), "1,000");
        assert_eq!(format_number(dec!(1000000)), "1,000,000");
        assert_eq!(format_number(dec!(1234.56)), "1,234.56");
        assert_eq!(format_number(dec!(0)), "0");
    }
}
