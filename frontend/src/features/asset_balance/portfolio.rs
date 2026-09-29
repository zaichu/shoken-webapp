//! 資産管理の構成比チャートの並びと表示計画の純粋ロジック。

use crate::features::asset_balance::model::chart_percentages;
use std::cmp::Ordering;

/// デフォルト表示件数。
pub const TOP_ITEMS: usize = 20;

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
