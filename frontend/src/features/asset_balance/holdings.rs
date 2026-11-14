use super::format::dec_to_f64;
use crate::features::asset_balance::csv::AssetBalanceRow;
use crate::features::asset_balance::model::{ValuationInput, normalize_display_name};
use crate::features::dividend_per_share::DividendMaps;
use crate::ui::format::{format_currency, format_percentage_value, format_unit_price};
use crate::ui::labels;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HoldingView {
    /// 保存済みレコードの ID。詳細ページのパスに使う。
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
    /// 現在値の基準日（"YYYY-MM-DD"）。
    pub(crate) price_as_of: Option<String>,
}

impl From<&HoldingView> for ValuationInput {
    fn from(view: &HoldingView) -> Self {
        Self {
            current_price: view.current_price,
            shares: view.shares,
            total_purchase_amount: view.purchase,
        }
    }
}

pub(crate) fn holding_view(row: &AssetBalanceRow) -> HoldingView {
    let name_source = if row.security_name.is_empty() {
        row.security_code.as_str()
    } else {
        row.security_name.as_str()
    };
    HoldingView {
        id: None,
        code: row.security_code.as_str().to_string(),
        name: normalize_display_name(name_source),
        account: row.account.as_ref().map(|a| a.as_str().to_string()),
        shares: dec_to_f64(&row.shares),
        average_price: dec_to_f64(&row.average_purchase_price),
        purchase: dec_to_f64(&row.total_purchase_amount),
        current_price: dec_to_f64(&row.current_price),
        day_change: if row.price_as_of.is_some() && row.prev_close.is_none() {
            None
        } else {
            Some(dec_to_f64(&row.daily_change))
        },
        price_as_of: row.price_as_of.clone(),
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
        Some("pending" | "error") => HoldingDividend {
            per_share: None,
            annual: None,
            yield_value: None,
            status,
        },
        Some("zero") => HoldingDividend {
            per_share: Some(0.0),
            annual: Some(0.0),
            yield_value: Some(0.0),
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
    format: impl Fn(f64) -> String,
) -> String {
    match dividend.status.as_deref() {
        Some("pending") => labels::CSV_MANAGE_FETCH_RUNNING.to_string(),
        Some("error") => labels::HOLDING_DIVIDEND_FAILED.to_string(),
        _ => value.map_or_else(|| "—".to_string(), format),
    }
}

/// 1株配当。列の中で小数桁がそろうよう decimals を渡す
pub(crate) fn format_dividend_per_share(dividend: &HoldingDividend, decimals: u32) -> String {
    format_dividend_value(dividend, dividend.per_share, |value| {
        format_unit_price(value, decimals)
    })
}

pub(crate) fn format_dividend_annual(dividend: &HoldingDividend) -> String {
    // 行ごとの年間配当は1円単位。表示単位は集計(サマリー)だけに使う
    format_dividend_value(dividend, dividend.annual, format_currency)
}

pub(crate) fn format_dividend_yield(dividend: &HoldingDividend) -> String {
    format_dividend_value(dividend, dividend.yield_value, format_percentage_value)
}

/// 銘柄画面(`/search?code=`)へのパス。資産管理の行・構成比・ホームの順位から使う
pub(crate) fn holding_stock_href(code: &str) -> String {
    format!("/search?code={}", urlencoding::encode(code))
}

/// 同一銘柄の保有行の集まり。口座が複数の銘柄は合計と口座ごとの内訳を出す
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HoldingGroup {
    pub(crate) code: String,
    pub(crate) name: String,
    pub(crate) current_price: f64,
    pub(crate) price_as_of: Option<String>,
    pub(crate) rows: Vec<HoldingView>,
    pub(crate) total_shares: f64,
    pub(crate) total_purchase: f64,
}

impl HoldingGroup {
    pub(crate) fn average_price(&self) -> f64 {
        if self.total_shares > 0.0 {
            self.total_purchase / self.total_shares
        } else {
            0.0
        }
    }
}

/// 保有行を銘柄コードごとにまとめる。順番は最初の行の並びを保つ
pub(crate) fn group_holdings_by_code(views: &[HoldingView]) -> Vec<HoldingGroup> {
    let mut groups: Vec<HoldingGroup> = Vec::new();
    for view in views {
        if let Some(group) = groups.iter_mut().find(|group| group.code == view.code) {
            group.total_shares += view.shares;
            group.total_purchase += view.purchase;
            group.rows.push(view.clone());
        } else {
            groups.push(HoldingGroup {
                code: view.code.clone(),
                name: view.name.clone(),
                current_price: view.current_price,
                price_as_of: view.price_as_of.clone(),
                rows: vec![view.clone()],
                total_shares: view.shares,
                total_purchase: view.purchase,
            });
        }
    }
    groups
}

#[cfg(test)]
mod tests;
