//! 資産管理の構成比チャートの並びと表示計画の純粋ロジック。

use crate::features::asset_balance::holdings::HoldingDividend;
use crate::features::asset_balance::model::{ValuationInput, chart_percentages, holding_valuation};
use crate::features::dividend_per_share::DividendMaps;
use std::cmp::Ordering;

/// デフォルト表示件数。
pub const TOP_ITEMS: usize = 20;

/// 並べ替えの指標。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortKey {
    MarketValue,
    ProfitLoss,
    Dividend,
    Yield,
}

/// 並べ替えの方向。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortOrder {
    Desc,
    Asc,
}

/// 並べ替えの入力1件。
#[derive(Clone)]
pub struct SortInput<'a> {
    pub code: &'a str,
    pub valuation: ValuationInput,
    pub dividend: &'a HoldingDividend,
}

/// 並べ替えの比較値。算出不能は `None`。
fn sort_value(input: &SortInput<'_>, key: SortKey) -> Option<f64> {
    match key {
        SortKey::MarketValue => holding_valuation(&input.valuation).market_value,
        SortKey::ProfitLoss => holding_valuation(&input.valuation).profit_loss.amount,
        SortKey::Dividend => input.dividend.annual,
        SortKey::Yield => input.dividend.yield_value,
    }
}

/// 並べ替えの方向に従って比較する。
/// 算出不能値は昇降どちらでも末尾。同値はコード順。
fn compare_sort(a: &SortInput<'_>, b: &SortInput<'_>, key: SortKey, order: SortOrder) -> Ordering {
    let (a_value, b_value) = (sort_value(a, key), sort_value(b, key));
    let ordering = match (a_value, b_value) {
        (Some(a), Some(b)) => b.total_cmp(&a),
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
    };
    let ordering = match order {
        SortOrder::Desc => ordering,
        SortOrder::Asc => ordering.reverse(),
    };
    if ordering == Ordering::Equal {
        a.code.cmp(b.code)
    } else {
        ordering
    }
}

/// 保有銘柄を並べ替える。検索後の全対象を並べてから件数制限する。
pub fn sort_holdings<'a, I>(inputs: I, key: SortKey, order: SortOrder) -> Vec<usize>
where
    I: IntoIterator<Item = SortInput<'a>>,
{
    let mut indexed: Vec<(usize, SortInput<'_>)> = inputs.into_iter().enumerate().collect();
    indexed.sort_by(|(a_index, a), (b_index, b)| {
        let ordering = compare_sort(a, b, key, order);
        if ordering == Ordering::Equal {
            a_index.cmp(b_index)
        } else {
            ordering
        }
    });
    indexed.into_iter().map(|(index, _)| index).collect()
}

/// 配当の取得状態を表示用にまとめる。
pub fn dividend_status_summary(dividends: &DividendMaps) -> (usize, usize, usize) {
    let mut pending = 0;
    let mut error = 0;
    let mut zero = 0;
    for status in dividends.status.values() {
        match status.as_str() {
            "pending" => pending += 1,
            "error" => error += 1,
            "zero" => zero += 1,
            _ => {}
        }
    }
    (pending, error, zero)
}

/// チャートの並び結果。`order` は表示順に並んだ全入力のインデックス、
/// `percentages` は同じ並びの未丸め構成比(取得額0・分母0で算出不可は `None`)。
#[derive(Clone, Debug)]
pub struct ChartPlan {
    pub order: Vec<usize>,
    pub percentages: Vec<Option<f64>>,
}

/// チャートの並びを求める。`values` は各行の取得総額。
/// `order` は全行を含む(保有カードは構成比を持たない行も表示するため)。
/// 構成比は取得額の絶対値で出し、取得額 0 の行は分母に入らないため `None` とし、
/// `order` では `Some` の行より後ろに置く。
///
/// 取得額降順→構成比降順の二段の安定ソートを踏む。構成比が `None` 同士の
/// 比較は「等しい」扱いで入力順を保つ。
pub fn chart_plan(values: &[f64]) -> ChartPlan {
    let mut order: Vec<usize> = (0..values.len()).collect();
    order.sort_by(|&a, &b| values[b].total_cmp(&values[a]));
    let all_percentages = chart_percentages(values);
    let mut paired: Vec<(usize, Option<f64>)> = order
        .into_iter()
        .map(|index| {
            let percentage = all_percentages
                .get(index)
                .copied()
                .flatten()
                .filter(|_| values[index] != 0.0);
            (index, percentage)
        })
        .collect();
    paired.sort_by(|a, b| match (b.1, a.1) {
        (Some(rhs), Some(lhs)) => rhs.total_cmp(&lhs),
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Less,
        (Some(_), None) => Ordering::Greater,
    });
    let (order, percentages) = paired.into_iter().unzip();
    ChartPlan { order, percentages }
}

/// 「その他」カードの集計値。
#[derive(Clone, Debug, PartialEq)]
pub struct OthersAggregate {
    /// 上位20件に入らなかった銘柄数。
    pub count: usize,
    /// その他にまとめた構成比の合計(未丸め)。
    pub percentage: f64,
}

/// 表示計画。
#[derive(Clone, Debug, PartialEq)]
pub struct ChartDisplay {
    /// 先頭から表示する件数。これに加えて構成比 `None` の行(取得額0など)は
    /// 折りたたみ対象外として常に表示する。
    pub visible_count: usize,
    /// 「その他」カード。`None` なら表示しない。
    pub others: Option<OthersAggregate>,
    /// 全件表示トグルのラベル。`None` ならボタンを出さない。
    pub toggle_label: Option<String>,
    pub grid_class: &'static str,
}

/// 折りたたみ・「その他」・全件表示トグルの表示計画を求める。
/// `percentages` は [`chart_plan`] の並び済み構成比で、`total` と同じ長さ。
/// 「その他」は構成比の合計が正のときだけ出す。
/// 構成比 `None` の行は折りたたまず常に表示し、「その他」・「残りN銘柄」の
/// 件数にも含めない(帯・凡例の「その他 N銘柄」と件数を揃えるため)。
pub fn chart_display(total: usize, percentages: &[Option<f64>], show_all: bool) -> ChartDisplay {
    debug_assert_eq!(total, percentages.len());
    let valued = percentages.iter().filter(|p| p.is_some()).count();
    let collapsed = !show_all && valued > TOP_ITEMS;
    let visible_count = if collapsed { TOP_ITEMS } else { total };
    let others_sum: f64 = percentages
        .iter()
        .skip(visible_count)
        .filter_map(|percentage| *percentage)
        .sum();
    let others = if collapsed && others_sum > 0.0 {
        Some(OthersAggregate {
            count: valued - TOP_ITEMS,
            percentage: others_sum,
        })
    } else {
        None
    };
    let toggle_label = if valued > TOP_ITEMS {
        Some(if show_all {
            format!("上位{TOP_ITEMS}件のみ表示")
        } else {
            format!("残り{}銘柄を表示（全{total}）", valued - TOP_ITEMS)
        })
    } else {
        None
    };
    let grid_class = if visible_count <= 1 {
        "grid grid-cols-1 gap-3"
    } else if visible_count == 2 && !show_all {
        "grid grid-cols-1 gap-3 xl:grid-cols-2"
    } else {
        "grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-3"
    };
    ChartDisplay {
        visible_count,
        others,
        toggle_label,
        grid_class,
    }
}

#[cfg(test)]
mod tests;
