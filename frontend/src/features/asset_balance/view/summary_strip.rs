use crate::features::asset_balance::format::{
    format_signed_row_valuation_amount, format_signed_valuation_amount,
    format_signed_valuation_rate, format_valuation_amount, is_negative_valuation,
    is_positive_valuation,
};
use crate::ui::amount::Amount;
use leptos::prelude::*;

/// 投資信託・「すべて」タブの4項目集計帯。
/// 評価額・評価損益・取得総額・前日比を国内株式の集計と同じ面に出す
#[component]
pub(crate) fn AssetSummaryStrip(
    /// 見出しに付ける種別名("投資信託" / "すべて")
    label: &'static str,
    market: f64,
    purchase: f64,
    profit: f64,
    /// 行から合計できない(未取得・全行前日比なし)ときは None
    daily_change: Option<f64>,
    /// 取得総額が確定できない(truncated など)ときの評価損益率
    profit_rate: Option<f64>,
    /// スマホの帯右上に出す注記(「前日比なし銘柄を除く」など)。空なら出さない
    #[prop(optional, into)]
    note: String,
) -> impl IntoView {
    let market_text = format_valuation_amount(Some(market));
    let profit_text = format_signed_valuation_amount(Some(profit));
    let rate_text = profit_rate.map(|rate| format_signed_valuation_rate(Some(rate), 1));
    let purchase_text = format_valuation_amount(Some(purchase));
    // 前日比は表・カードと同じく表示単位を適用せず必ず円マーク付きで出す
    let daily_text = format_signed_row_valuation_amount(daily_change);
    let profit_tone = if is_negative_valuation(Some(profit)) {
        "asset-summary-loss"
    } else if is_positive_valuation(Some(profit)) {
        "asset-summary-gain"
    } else {
        ""
    };
    let daily_tone = daily_change.map_or("", |value| {
        if is_negative_valuation(Some(value)) {
            "asset-summary-loss"
        } else if is_positive_valuation(Some(value)) {
            "asset-summary-gain"
        } else {
            ""
        }
    });
    view! {
        <section
            class="asset-summary asset-summary-band"
            role="region"
            aria-label="集計"
            data-testid="asset-summary-strip"
        >
            {(!note.is_empty()).then(|| {
                view! { <p class="asset-summary-asof lg:hidden">{note}</p> }
            })}
            <dl class="asset-summary-grid">
                <div class="asset-summary-item asset-summary-item-primary">
                    <dt>{format!("評価額({label})")}</dt>
                    <dd>
                        <Amount text=market_text class="asset-summary-primary-amount".to_string() />
                    </dd>
                </div>
                <div class="asset-summary-item asset-summary-item-pnl">
                    <dt class="sr-only lg:not-sr-only">"評価損益"</dt>
                    <dd>
                        <Amount
                            text=profit_text
                            class=format!("asset-summary-pnl-amount {profit_tone}")
                        />
                        {rate_text.map(|rate| {
                            view! {
                                <span class=format!("asset-summary-pnl-note {profit_tone}")>
                                    {rate}
                                </span>
                            }
                        })}
                    </dd>
                </div>
                <div class="asset-summary-item asset-summary-item-sub2 asset-summary-item-daily">
                    <dt>"前日比"</dt>
                    <dd>
                        <Amount
                            text=daily_text
                            class=format!("asset-summary-sub-amount {daily_tone}")
                        />
                    </dd>
                </div>
                <div class="asset-summary-tags">
                    <div class="asset-summary-item asset-summary-item-sub1 asset-summary-item-tag">
                        <dt>"取得総額"</dt>
                        <dd>
                            <Amount text=purchase_text class="asset-summary-sub-amount".to_string() />
                        </dd>
                    </div>
                </div>
            </dl>
        </section>
    }
}
