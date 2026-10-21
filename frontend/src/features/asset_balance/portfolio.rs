//! 資産管理の構成比一覧と並べ替えの純粋ロジック。

use crate::features::asset_balance::holdings::HoldingDividend;
use crate::features::asset_balance::model::{ValuationInput, chart_percentages, holding_valuation};
use std::cmp::Ordering;

/// 並べ替えの指標。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortKey {
    MarketValue,
    ProfitLoss,
    Dividend,
    Yield,
}

impl SortKey {
    /// 絞り込み行の選択肢
    pub(crate) const ALL: [SortKey; 4] = [
        SortKey::MarketValue,
        SortKey::ProfitLoss,
        SortKey::Dividend,
        SortKey::Yield,
    ];

    pub(crate) fn label(self) -> &'static str {
        match self {
            SortKey::MarketValue => "評価額",
            SortKey::ProfitLoss => "評価損益",
            SortKey::Dividend => "年間配当",
            SortKey::Yield => "取得額利回り",
        }
    }

    pub(crate) fn value(self) -> &'static str {
        match self {
            SortKey::MarketValue => "market_value",
            SortKey::ProfitLoss => "profit_loss",
            SortKey::Dividend => "dividend",
            SortKey::Yield => "yield",
        }
    }

    pub(crate) fn from_value(value: &str) -> Option<Self> {
        match value {
            "market_value" => Some(SortKey::MarketValue),
            "profit_loss" => Some(SortKey::ProfitLoss),
            "dividend" => Some(SortKey::Dividend),
            "yield" => Some(SortKey::Yield),
            _ => None,
        }
    }
}

/// 並べ替えの方向。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortOrder {
    Desc,
    Asc,
}

impl SortOrder {
    pub(crate) fn label(self) -> &'static str {
        match self {
            SortOrder::Desc => "高い順 ↓",
            SortOrder::Asc => "低い順 ↑",
        }
    }

    /// localStorage への保存値
    pub(crate) fn value(self) -> &'static str {
        match self {
            SortOrder::Desc => "desc",
            SortOrder::Asc => "asc",
        }
    }

    pub(crate) fn from_value(value: &str) -> Option<Self> {
        match value {
            "desc" => Some(SortOrder::Desc),
            "asc" => Some(SortOrder::Asc),
            _ => None,
        }
    }
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
        (Some(a), Some(b)) => {
            let ordering = b.total_cmp(&a);
            match order {
                SortOrder::Desc => ordering,
                SortOrder::Asc => ordering.reverse(),
            }
        }
        // 算出不能は昇降どちらでも末尾に置くため方向を反転しない
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
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

/// 構成比一覧の並び結果。`order` は表示順に並んだ全入力のインデックス、
/// `percentages` は同じ並びの未丸め構成比(取得額0・分母0で算出不可は `None`)。
#[derive(Clone, Debug)]
pub struct ChartPlan {
    pub order: Vec<usize>,
    pub percentages: Vec<Option<f64>>,
}

/// 構成比一覧の並びを求める。`values` は各行の取得総額。
/// `order` は全行を含む(算出不可の行も一覧に残すため)。
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

#[cfg(test)]
mod tests;
