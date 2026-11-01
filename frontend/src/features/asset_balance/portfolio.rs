//! 資産管理の構成比一覧と並べ替えの純粋ロジック。

use crate::features::asset_balance::holdings::{HoldingDividend, HoldingView};
use crate::features::asset_balance::model::{ValuationInput, chart_percentages, holding_valuation};
use crate::ui::badge::account_display_name;
use crate::ui::labels::{
    ACQUISITION_YIELD, ANNUAL_DIVIDEND, COMPOSITION_DIVIDEND_BASIS, COMPOSITION_GROUP_ACCOUNT,
    COMPOSITION_GROUP_HOLDING, COMPOSITION_GROUP_PRODUCT, MARKET_VALUE, MISSING,
    VALUATION_PROFIT_LOSS, composition_other,
};
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
            SortKey::MarketValue => MARKET_VALUE,
            SortKey::ProfitLoss => VALUATION_PROFIT_LOSS,
            SortKey::Dividend => ANNUAL_DIVIDEND,
            SortKey::Yield => ACQUISITION_YIELD,
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

/// 構成比の区分切り替え(銘柄別/商品別/口座別)。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompositionGroup {
    Holding,
    Product,
    Account,
}

impl CompositionGroup {
    pub(crate) const ALL: [CompositionGroup; 3] = [
        CompositionGroup::Holding,
        CompositionGroup::Product,
        CompositionGroup::Account,
    ];

    pub(crate) fn label(self) -> &'static str {
        match self {
            CompositionGroup::Holding => COMPOSITION_GROUP_HOLDING,
            CompositionGroup::Product => COMPOSITION_GROUP_PRODUCT,
            CompositionGroup::Account => COMPOSITION_GROUP_ACCOUNT,
        }
    }

    pub(crate) fn value(self) -> &'static str {
        match self {
            CompositionGroup::Holding => "holding",
            CompositionGroup::Product => "product",
            CompositionGroup::Account => "account",
        }
    }
}

/// 構成比の基準切り替え(評価額/年間配当)。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompositionBasis {
    MarketValue,
    Dividend,
}

impl CompositionBasis {
    pub(crate) fn label(self) -> &'static str {
        match self {
            CompositionBasis::MarketValue => MARKET_VALUE,
            CompositionBasis::Dividend => COMPOSITION_DIVIDEND_BASIS,
        }
    }

    pub(crate) fn value(self) -> &'static str {
        match self {
            CompositionBasis::MarketValue => "market_value",
            CompositionBasis::Dividend => "dividend",
        }
    }
}

/// 構成比の区分1行。`value` は基準の金額で、算出不可は `None`(一覧の末尾に置く)。
#[derive(Clone, Debug, PartialEq)]
pub struct CompositionEntry {
    pub name: String,
    pub sub: String,
    pub value: Option<f64>,
    pub holdings: usize,
    pub href: Option<String>,
}

/// 区分を金額の降順に並べる。算出不可は末尾、同値は入力順を保つ
fn sort_composition_entries(entries: &mut [CompositionEntry]) {
    entries.sort_by(|a, b| match (b.value, a.value) {
        (Some(rhs), Some(lhs)) => rhs.total_cmp(&lhs),
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Less,
        (Some(_), None) => Ordering::Greater,
    });
}

/// 保有行を区分にまとめる。`basis` は各行の基準金額(算出不可は `None`)
/// で、区分の金額は行の合計(行が全て算出不可なら区分も算出不可)になる。
/// 商品別は同一銘柄コード、口座別は同一口座でまとめ、銘柄別は行のまま出す
pub fn composition_entries(
    views: &[HoldingView],
    group: CompositionGroup,
    basis: &[Option<f64>],
) -> Vec<CompositionEntry> {
    fn entry_key(view: &HoldingView, group: CompositionGroup) -> String {
        match group {
            CompositionGroup::Holding => String::new(),
            CompositionGroup::Product => view.code.clone(),
            CompositionGroup::Account => view.account.clone().unwrap_or_default(),
        }
    }
    let mut entries: Vec<CompositionEntry> = Vec::new();
    let mut positions: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for (view, value) in views.iter().zip(basis.iter().copied()) {
        let key = entry_key(view, group);
        let is_new_holding = matches!(group, CompositionGroup::Holding);
        let position = if is_new_holding {
            entries.len()
        } else {
            match positions.get(&key) {
                Some(&position) => position,
                None => {
                    let position = entries.len();
                    positions.insert(key.clone(), position);
                    position
                }
            }
        };
        if position == entries.len() {
            entries.push(CompositionEntry {
                name: match group {
                    CompositionGroup::Account => view
                        .account
                        .as_deref()
                        .map(account_display_name)
                        .unwrap_or(MISSING)
                        .to_string(),
                    _ => view.name.clone(),
                },
                sub: match group {
                    CompositionGroup::Holding | CompositionGroup::Product => view.code.clone(),
                    CompositionGroup::Account => String::new(),
                },
                value: None,
                holdings: 0,
                href: None,
            });
        }
        let entry = &mut entries[position];
        if entry.holdings == 0 {
            entry.value = value;
        } else if let Some(value) = value {
            entry.value = Some(entry.value.unwrap_or(0.0) + value);
        }
        entry.holdings += 1;
        if entry.holdings == 1 {
            entry.href = view
                .id
                .as_ref()
                .map(|id| format!("/assetbalance/holdings/{id}"));
        } else {
            entry.href = None;
        }
    }
    sort_composition_entries(&mut entries);
    entries
}

/// 基準の合計と区分ごとの未丸め構成比。基準が0の区分は `None`、
/// 合計が0以下なら算出不可(全て `None`)
pub fn composition_percentages(values: &[Option<f64>]) -> (f64, Vec<Option<f64>>) {
    let raw: Vec<f64> = values.iter().map(|value| value.unwrap_or(0.0)).collect();
    let total: f64 = raw.iter().map(|value| value.abs()).sum();
    if total <= 0.0 {
        return (total, vec![None; values.len()]);
    }
    let percentages = chart_percentages(&raw);
    let percentages = percentages
        .into_iter()
        .enumerate()
        .map(
            |(index, percentage)| {
                if raw[index] == 0.0 { None } else { percentage }
            },
        )
        .collect();
    (total, percentages)
}

/// ドーナツに個別で出す割合の下限。未満は「その他」にまとめる
pub const COMPOSITION_MAJOR_THRESHOLD: f64 = 9.0;

/// ドーナツの環の半径。ラベルの収まり判定と描画で同じ値を使う
pub const COMPOSITION_DONUT_RADIUS: f64 = 80.0;

/// ドーナツ内の1行ラベルが区分に収まるか。文字幅の見積もりが区分の弧の長さに
/// 収まるときだけ文字を出す(収まらない区分は一覧で分かる)。
/// 文字は弧に沿わせるため、弧に収まれば帯の外に出ない
pub fn composition_label_fits(label: &str, percentage: f64) -> bool {
    let width: f64 = label
        .chars()
        .map(|c| if c.is_ascii() { 5.0 } else { 9.0 })
        .sum();
    let arc = COMPOSITION_DONUT_RADIUS * (percentage / 100.0 * std::f64::consts::TAU);
    width <= arc
}

/// ドーナツの区分の弧パスと文字の向き。真下をまたぐ区分は読みやすいよう
/// 逆向き(反時計回り)に描く。角度は上を起点に時計回りの度数。
/// 100% の区分も2つの弧で描くため、欠けなく1周する
pub fn composition_slice_arc(start: f64, percentage: f64) -> (String, bool) {
    fn point(angle: f64) -> (f64, f64) {
        let radians = angle.to_radians();
        (
            100.0 + COMPOSITION_DONUT_RADIUS * radians.sin(),
            100.0 - COMPOSITION_DONUT_RADIUS * radians.cos(),
        )
    }
    let from = start / 100.0 * 360.0;
    let to = (start + percentage) / 100.0 * 360.0;
    let middle = (from + to) / 2.0;
    let reversed = 90.0 < middle && middle <= 270.0;
    let (head, middle_point, tail) = if reversed {
        (point(to), point(middle), point(from))
    } else {
        (point(from), point(middle), point(to))
    };
    let sweep = if reversed { 0 } else { 1 };
    let path = format!(
        "M {} {} A {} {} 0 0 {} {} {} A {} {} 0 0 {} {} {}",
        head.0,
        head.1,
        COMPOSITION_DONUT_RADIUS,
        COMPOSITION_DONUT_RADIUS,
        sweep,
        middle_point.0,
        middle_point.1,
        COMPOSITION_DONUT_RADIUS,
        COMPOSITION_DONUT_RADIUS,
        sweep,
        tail.0,
        tail.1
    );
    (path, reversed)
}

/// 区分の色番号。`[0, 2, 4, 1, 3]` を回して隣り合う区分の色が並ばないようにする
pub fn composition_color(position: usize) -> usize {
    const ORDER: [usize; 5] = [0, 2, 4, 1, 3];
    ORDER[position % ORDER.len()]
}

/// その他の区分の色番号(中立色)。通常の色番号(0〜4)と重ねない
pub const COMPOSITION_OTHER_COLOR: usize = usize::MAX;

/// 構成比の表示1行。割合は未丸めで、算出不可の行は `computable` が false
#[derive(Clone, Debug, PartialEq)]
pub struct CompositionRow {
    pub name: String,
    pub sub: String,
    pub value: Option<f64>,
    pub percentage: f64,
    pub computable: bool,
    pub color: usize,
    pub holdings: usize,
    pub href: Option<String>,
    pub is_other: bool,
}

/// 区分をドーナツと一覧に出す行に分ける。割合が下限以上の区分は個別に、
/// それ未満と算出不可は「その他 ◯銘柄」にまとめる。その他自体の色は中立のままにする
pub fn composition_rows(
    entries: &[CompositionEntry],
    percentages: &[Option<f64>],
) -> Vec<CompositionRow> {
    let mut rows = Vec::new();
    let mut other_holdings = 0;
    let mut other_percentage = 0.0;
    let mut other_value: Option<f64> = None;
    let mut other_computable = false;
    for (entry, percentage) in entries.iter().zip(percentages.iter().copied()) {
        if percentage.is_some_and(|value| value >= COMPOSITION_MAJOR_THRESHOLD) {
            rows.push(CompositionRow {
                name: entry.name.clone(),
                sub: entry.sub.clone(),
                value: entry.value,
                percentage: percentage.unwrap_or(0.0),
                computable: true,
                color: composition_color(rows.len()),
                holdings: entry.holdings,
                href: entry.href.clone(),
                is_other: false,
            });
            continue;
        }
        other_holdings += entry.holdings;
        if let Some(percentage) = percentage {
            other_percentage += percentage;
            other_computable = true;
        }
        if let Some(value) = entry.value {
            other_value = Some(other_value.unwrap_or(0.0) + value);
        }
    }
    if other_holdings > 0 {
        rows.push(CompositionRow {
            name: composition_other(other_holdings),
            sub: String::new(),
            value: other_value,
            percentage: other_percentage,
            computable: other_computable,
            color: COMPOSITION_OTHER_COLOR,
            holdings: other_holdings,
            href: None,
            is_other: true,
        });
    }
    rows
}
