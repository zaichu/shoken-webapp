//! 資産管理の構成比一覧と並べ替えの純粋ロジック。

use crate::features::asset_balance::fund::FundView;
use crate::features::asset_balance::holdings::{HoldingDividend, HoldingView};
use crate::features::asset_balance::model::{ValuationInput, chart_percentages, holding_valuation};
use crate::ui::badge::account_display_name;
use crate::ui::labels::{
    ACQUISITION_YIELD, ANNUAL_DIVIDEND, COMPOSITION_DIVIDEND_BASIS, COMPOSITION_GROUP_ACCOUNT,
    COMPOSITION_GROUP_HOLDING, COMPOSITION_GROUP_PRODUCT, DOMESTIC_STOCK, MARKET_VALUE, MISSING,
    MUTUAL_FUND, VALUATION_PROFIT_LOSS,
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

/// 構成比の商品種別。商品別の区分名になる
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompositionProduct {
    Stock,
    Fund,
}

impl CompositionProduct {
    pub(crate) fn label(self) -> &'static str {
        match self {
            CompositionProduct::Stock => DOMESTIC_STOCK,
            CompositionProduct::Fund => MUTUAL_FUND,
        }
    }
}

/// 構成比の入力1行(国内株式・投資信託共通)。`basis` と並べて渡す
#[derive(Clone, Debug, PartialEq)]
pub struct CompositionSource {
    pub name: String,
    pub sub: String,
    pub account: Option<String>,
    pub product: CompositionProduct,
    pub href: Option<String>,
}

/// 国内株式と投資信託の保有行を構成比の入力にそろえる。
/// 投資信託の sub は分配金コース(銘柄名だけでは区別できない行があるため)
pub fn composition_sources(views: &[HoldingView], funds: &[FundView]) -> Vec<CompositionSource> {
    views
        .iter()
        .map(|view| CompositionSource {
            name: view.name.clone(),
            sub: view.code.clone(),
            account: view.account.clone(),
            product: CompositionProduct::Stock,
            href: view
                .id
                .as_ref()
                .map(|id| format!("/assetbalance/holdings/{id}")),
        })
        .chain(funds.iter().map(|view| {
            CompositionSource {
                name: view.name.clone(),
                sub: view.course.clone().unwrap_or_default(),
                account: Some(view.account.clone()),
                product: CompositionProduct::Fund,
                href: view
                    .id
                    .as_ref()
                    .map(|id| format!("/assetbalance/funds/{id}")),
            }
        }))
        .collect()
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
/// 基準が0の行は算出不可として扱い、末尾に置く。
/// 銘柄別は行のまま、商品別は商品種別、口座別は同一口座でまとめる
pub fn composition_entries(
    sources: &[CompositionSource],
    group: CompositionGroup,
    basis: &[Option<f64>],
) -> Vec<CompositionEntry> {
    fn entry_key(source: &CompositionSource, group: CompositionGroup) -> String {
        match group {
            CompositionGroup::Holding => String::new(),
            CompositionGroup::Product => source.product.label().to_string(),
            CompositionGroup::Account => source.account.clone().unwrap_or_default(),
        }
    }
    let mut entries: Vec<CompositionEntry> = Vec::new();
    let mut positions: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for (source, value) in sources.iter().zip(basis.iter().copied()) {
        let value = value.filter(|value| *value != 0.0);
        let key = entry_key(source, group);
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
                    CompositionGroup::Holding => source.name.clone(),
                    CompositionGroup::Product => source.product.label().to_string(),
                    CompositionGroup::Account => source
                        .account
                        .as_deref()
                        .map(account_display_name)
                        .unwrap_or(MISSING)
                        .to_string(),
                },
                sub: match group {
                    CompositionGroup::Holding => source.sub.clone(),
                    CompositionGroup::Product | CompositionGroup::Account => String::new(),
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
            entry.href = source.href.clone();
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

/// ドーナツの注釈に出す割合の下限(以上だけ出す)
pub const COMPOSITION_ANNOTATION_MIN_PERCENT: f64 = 3.0;

/// ドーナツの注釈の上限(重ならずに置ける分だけ、多いときはここまで)
pub const COMPOSITION_ANNOTATION_MAX: usize = 10;

/// 区分の色番号。`[0, 2, 4, 1, 3]` を回して隣り合う区分の色が並ばないようにする
pub fn composition_color(position: usize) -> usize {
    const ORDER: [usize; 5] = [0, 2, 4, 1, 3];
    ORDER[position % ORDER.len()]
}

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
}

/// 区分をドーナツと一覧に出す行に分ける。全区分を1行ずつ出し、
/// 「その他」にはまとめない。並びは entries のまま(比率の降順)で、
/// 色もその順に割り当てる
pub fn composition_rows(
    entries: &[CompositionEntry],
    percentages: &[Option<f64>],
) -> Vec<CompositionRow> {
    entries
        .iter()
        .zip(percentages.iter().copied())
        .enumerate()
        .map(|(position, (entry, percentage))| CompositionRow {
            name: entry.name.clone(),
            sub: entry.sub.clone(),
            value: entry.value,
            percentage: percentage.unwrap_or(0.0),
            computable: percentage.is_some(),
            color: composition_color(position),
            holdings: entry.holdings,
            href: entry.href.clone(),
        })
        .collect()
}

/// 切れの中央の角度から注釈の上下位置と左右を出す。ドーナツの描画と
/// 注釈の配置で同じ値を使い、線の引き先と判定を一致させる
pub fn composition_annotation_side(start: f64, percentage: f64) -> (f64, f64, bool) {
    let mid = start + percentage / 2.0;
    let angle = (mid / 100.0 * 360.0 - 90.0).to_radians();
    let (sine, cosine) = angle.sin_cos();
    (sine, cosine, cosine >= 0.0)
}

/// ドーナツの注釈に出す行の位置を選ぶ。比率の大きい順に下限以上を拾い、
/// 同じ側で重なるものは飛ばして上限まで残す
pub fn composition_annotation_indices(rows: &[CompositionRow]) -> Vec<usize> {
    const MIN_GAP: f64 = 0.12;
    let mut starts = Vec::with_capacity(rows.len());
    let mut cursor = 0.0;
    for row in rows {
        starts.push(cursor);
        cursor += row.percentage;
    }
    let mut selected: Vec<(usize, f64, bool)> = Vec::new();
    for (index, row) in rows.iter().enumerate() {
        if selected.len() >= COMPOSITION_ANNOTATION_MAX {
            break;
        }
        if !row.computable || row.percentage < COMPOSITION_ANNOTATION_MIN_PERCENT {
            continue;
        }
        let (sine, _, right) = composition_annotation_side(starts[index], row.percentage);
        let overlaps = selected.iter().any(|(_, placed, placed_right)| {
            *placed_right == right && (sine - placed).abs() < MIN_GAP
        });
        if overlaps {
            continue;
        }
        selected.push((index, sine, right));
    }
    selected.into_iter().map(|(index, _, _)| index).collect()
}
