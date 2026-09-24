//! 資産管理の構成比チャートの並びと表示計画の純粋ロジック。
//!
//! 受け入れ条件「React と同じ入力で同じ結果」のため、React 側の正本との対応:
//! - [`chart_plan`] は `AssetPortfolioSummary.tsx` の `chartData`
//!   (除外条件・`b.value - a.value` ソート) と `PortfolioPieChart.tsx` の
//!   `percentage` 付与・`b.percentage - a.percentage` ソートに対応する。
//!   どちらも安定ソートなので、同率は入力順を保つ。
//! - [`chart_display`] は `PortfolioPieChart.tsx` の `displayData` /
//!   `othersPercentage` / 全件表示トグル / グリッドクラスに対応する。

use crate::asset_balance_domain::{chart_percentages, should_include_chart_item};
use std::cmp::Ordering;

/// デフォルト表示件数。`PortfolioPieChart.tsx` の `TOP_N` に対応する。
pub const TOP_ITEMS: usize = 20;

/// チャートの並び結果。`order` は表示順に並んだ入力インデックス、
/// `percentages` は同じ並びの未丸め構成比(分母0で算出不可は `None`、
/// React の `NaN` に相当)。
#[derive(Clone, Debug)]
pub struct ChartPlan {
    pub order: Vec<usize>,
    pub percentages: Vec<Option<f64>>,
}

/// チャートの並びを求める。`values`(取得総額)と `markets`(評価額)は同じ長さで、
/// インデックスが同じ要素どうし対応する。
///
/// React と同じく取得額降順→構成比降順の二段の安定ソートを踏む。
/// 合計が正なら構成比降順は取得額降順と同じ並びだが、合計が0以下でも
/// React と同じ結果になるよう同じ手順にしている(構成比が `NaN` 同士の比較は
/// 「等しい」扱いで入力順を保つ)。
pub fn chart_plan(values: &[f64], markets: &[Option<f64>]) -> ChartPlan {
    debug_assert_eq!(values.len(), markets.len());
    let mut order: Vec<usize> = (0..values.len())
        .filter(|&index| should_include_chart_item(Some(values[index]), markets[index]))
        .collect();
    order.sort_by(|&a, &b| values[b].total_cmp(&values[a]));
    let kept_values: Vec<f64> = order.iter().map(|&index| values[index]).collect();
    let kept_markets: Vec<Option<f64>> = order.iter().map(|&index| markets[index]).collect();
    let percentages = chart_percentages(&kept_values, &kept_markets);
    let mut paired: Vec<(usize, Option<f64>)> = order.into_iter().zip(percentages).collect();
    paired.sort_by(|a, b| match (b.1, a.1) {
        (Some(rhs), Some(lhs)) => rhs.total_cmp(&lhs),
        _ => Ordering::Equal,
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

/// 表示計画。`PortfolioPieChart.tsx` の表示分岐に対応する。
#[derive(Clone, Debug, PartialEq)]
pub struct ChartDisplay {
    /// 先頭から表示する件数(`order` の先頭 `visible_count` 件を出す)。
    pub visible_count: usize,
    /// 「その他」カード。`None` なら表示しない。
    pub others: Option<OthersAggregate>,
    /// 全件表示トグルのラベル。`None` ならボタンを出さない。
    pub toggle_label: Option<String>,
    /// グリッドの CSS クラス。
    pub grid_class: &'static str,
}

/// 折りたたみ・「その他」・全件表示トグルの表示計画を求める。
/// `percentages` は [`chart_plan`] の並び済み構成比で、`total` と同じ長さ。
/// 「その他」は構成比の合計が正のときだけ出す(React の `othersPercentage > 0`)。
pub fn chart_display(total: usize, percentages: &[Option<f64>], show_all: bool) -> ChartDisplay {
    debug_assert_eq!(total, percentages.len());
    let collapsed = !show_all && total > TOP_ITEMS;
    let visible_count = if collapsed { TOP_ITEMS } else { total };
    let others_sum: f64 = percentages
        .iter()
        .skip(visible_count)
        .filter_map(|percentage| *percentage)
        .sum();
    let others = if collapsed && others_sum > 0.0 {
        Some(OthersAggregate {
            count: total - TOP_ITEMS,
            percentage: others_sum,
        })
    } else {
        None
    };
    let toggle_label = if total > TOP_ITEMS {
        Some(if show_all {
            format!("上位{TOP_ITEMS}件のみ表示")
        } else {
            format!("残り{}銘柄を表示（全{total}）", total - TOP_ITEMS)
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
