use super::format::{dec_to_f64, format_currency, format_percentage_value};
use crate::features::asset_balance::csv::AssetBalanceRowData;
use crate::features::asset_balance::model::{ValuationInput, normalize_display_name};
use crate::features::dividend_per_share::DividendMaps;

#[derive(Clone, Debug)]
pub(crate) struct HoldingView {
    /// 保存済みレコードの ID。詳細ページのパスに使う。CSV プレビュー行は None
    pub(crate) id: Option<String>,
    pub(crate) code: String,
    pub(crate) name: String,
    /// 口座区分。CSV の区切りから取れなかった行・旧データは None
    pub(crate) account: Option<String>,
    pub(crate) shares: f64,
    pub(crate) average_price: f64,
    pub(crate) purchase: f64,
    pub(crate) current_price: f64,
    /// 1株あたりの前日比。前日終値がない銘柄は None
    pub(crate) day_change: Option<f64>,
    /// 現在値の基準日（"YYYY-MM-DD"）。CSV プレビュー行は None
    pub(crate) price_as_of: Option<String>,
}

impl From<&HoldingView> for ValuationInput {
    fn from(view: &HoldingView) -> Self {
        ValuationInput {
            current_price: view.current_price,
            shares: view.shares,
            total_purchase_amount: view.purchase,
        }
    }
}

pub(crate) fn holding_view(row: &impl AssetBalanceRowData) -> HoldingView {
    let name_source = if row.security_name().is_empty() {
        row.security_code()
    } else {
        row.security_name()
    };
    HoldingView {
        id: None,
        code: row.security_code().to_string(),
        name: normalize_display_name(name_source),
        account: row.account().map(str::to_string),
        shares: dec_to_f64(&row.shares()),
        average_price: dec_to_f64(&row.average_purchase_price()),
        purchase: dec_to_f64(&row.total_purchase_amount()),
        current_price: dec_to_f64(&row.current_price()),
        day_change: row.day_change().as_ref().map(dec_to_f64),
        price_as_of: row.price_as_of().map(str::to_string),
    }
}

/// 保有数で重み付けした前日比の合計(円)。前日比なしの行は除き、
/// 対象が1行もないときは None
pub(crate) fn stock_daily_change_total(views: &[HoldingView]) -> Option<f64> {
    let mut total = 0.0;
    let mut has = false;
    for view in views {
        if let Some(change) = view.day_change {
            total += change * view.shares;
            has = true;
        }
    }
    has.then_some(total)
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HoldingDividend {
    pub(crate) per_share: Option<f64>,
    pub(crate) annual: Option<f64>,
    pub(crate) yield_value: Option<f64>,
    pub(crate) status: Option<String>,
}

pub(crate) fn holding_dividend(
    code: &str,
    shares: f64,
    average_price: f64,
    maps: &DividendMaps,
) -> HoldingDividend {
    let status = maps.status.get(code).cloned();
    match status.as_deref() {
        Some("pending") | Some("error") => HoldingDividend {
            per_share: None,
            annual: None,
            yield_value: None,
            status,
        },
        Some("zero") => HoldingDividend {
            per_share: Some(0.0),
            annual: Some(0.0),
            yield_value: None,
            status,
        },
        _ => match maps.per_share.get(code) {
            None => HoldingDividend {
                per_share: None,
                annual: None,
                yield_value: None,
                status,
            },
            Some(per_share) => HoldingDividend {
                per_share: Some(*per_share),
                annual: Some(*per_share * shares),
                yield_value: if average_price > 0.0 {
                    Some(*per_share / average_price * 100.0)
                } else {
                    None
                },
                status,
            },
        },
    }
}

fn format_dividend_value(
    dividend: &HoldingDividend,
    value: Option<f64>,
    format: fn(f64) -> String,
) -> String {
    match dividend.status.as_deref() {
        Some("pending") => "取得中...".to_string(),
        Some("error") => "取得失敗".to_string(),
        _ => value.map_or_else(|| "—".to_string(), format),
    }
}

pub(crate) fn format_dividend_per_share(dividend: &HoldingDividend) -> String {
    format_dividend_value(dividend, dividend.per_share, format_currency)
}

pub(crate) fn format_dividend_annual(dividend: &HoldingDividend) -> String {
    // 行ごとの年間配当は1円単位。表示単位は集計(サマリー)だけに使う
    format_dividend_value(dividend, dividend.annual, format_currency)
}

pub(crate) fn format_dividend_yield(dividend: &HoldingDividend) -> String {
    format_dividend_value(dividend, dividend.yield_value, format_percentage_value)
}

#[cfg(test)]
mod tests;
