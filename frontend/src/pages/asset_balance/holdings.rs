use super::format::{dec_to_f64, format_currency, format_percentage_value};
use crate::asset_balance_domain::normalize_display_name;
use crate::dividend_per_share::DividendMaps;
use crate::dto::AssetBalance;

#[derive(Clone, Debug)]
pub(crate) struct HoldingView {
    pub(crate) code: String,
    pub(crate) name: String,
    pub(crate) shares: f64,
    pub(crate) average_price: f64,
    pub(crate) purchase: f64,
    pub(crate) market: f64,
    pub(crate) current_price: f64,
}

pub(crate) fn holding_view(row: &AssetBalance) -> HoldingView {
    let name_source = if row.security_name.is_empty() {
        row.security_code.as_str()
    } else {
        row.security_name.as_str()
    };
    HoldingView {
        code: row.security_code.clone(),
        name: normalize_display_name(name_source),
        shares: dec_to_f64(&row.shares),
        average_price: dec_to_f64(&row.average_purchase_price),
        purchase: dec_to_f64(&row.total_purchase_amount),
        market: dec_to_f64(&row.market_value),
        current_price: dec_to_f64(&row.current_price),
    }
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

pub(crate) fn format_dividend_per_share(dividend: &HoldingDividend) -> String {
    match dividend.status.as_deref() {
        Some("pending") => "取得中...".to_string(),
        Some("error") => "取得失敗".to_string(),
        _ => match dividend.per_share {
            Some(value) => format_currency(value),
            None => "---".to_string(),
        },
    }
}

pub(crate) fn format_dividend_annual(dividend: &HoldingDividend) -> String {
    match dividend.status.as_deref() {
        Some("pending") => "取得中...".to_string(),
        Some("error") => "取得失敗".to_string(),
        _ => match dividend.annual {
            Some(value) => format_currency(value),
            None => "---".to_string(),
        },
    }
}

pub(crate) fn format_dividend_yield(dividend: &HoldingDividend) -> String {
    match dividend.status.as_deref() {
        Some("pending") => "取得中...".to_string(),
        Some("error") => "取得失敗".to_string(),
        _ => match dividend.yield_value {
            Some(value) => format_percentage_value(value),
            None => "---".to_string(),
        },
    }
}
