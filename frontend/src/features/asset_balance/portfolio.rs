//! 資産管理の構成比一覧と並べ替えの純粋ロジック。

use crate::features::asset_balance::fund::FundView;
use crate::features::asset_balance::holdings::{HoldingDividend, HoldingView, holding_stock_href};
use crate::features::asset_balance::model::{ValuationInput, chart_percentages, holding_valuation};
use crate::ui::badge::account_display_name;
use crate::ui::labels::{
    ACQUISITION_YIELD, ANNUAL_DIVIDEND, COMPOSITION_DIVIDEND_BASIS, COMPOSITION_GROUP_ACCOUNT,
    COMPOSITION_GROUP_HOLDING, COMPOSITION_GROUP_PRODUCT, DOMESTIC_STOCK, MARKET_VALUE, MISSING,
    MUTUAL_FUND, VALUATION_PROFIT_LOSS,
};
use crate::urls::fund_holding_href;
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
    pub(crate) const ALL: [Self; 4] = [
        Self::MarketValue,
        Self::ProfitLoss,
        Self::Dividend,
        Self::Yield,
    ];

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::MarketValue => MARKET_VALUE,
            Self::ProfitLoss => VALUATION_PROFIT_LOSS,
            Self::Dividend => ANNUAL_DIVIDEND,
            Self::Yield => ACQUISITION_YIELD,
        }
    }

    pub(crate) const fn value(self) -> &'static str {
        match self {
            Self::MarketValue => "market_value",
            Self::ProfitLoss => "profit_loss",
            Self::Dividend => "dividend",
            Self::Yield => "yield",
        }
    }

    pub(crate) fn from_value(value: &str) -> Option<Self> {
        match value {
            "market_value" => Some(Self::MarketValue),
            "profit_loss" => Some(Self::ProfitLoss),
            "dividend" => Some(Self::Dividend),
            "yield" => Some(Self::Yield),
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
    pub(crate) const fn value(self) -> &'static str {
        match self {
            Self::Desc => "desc",
            Self::Asc => "asc",
        }
    }

    pub(crate) fn from_value(value: &str) -> Option<Self> {
        match value {
            "desc" => Some(Self::Desc),
            "asc" => Some(Self::Asc),
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
    pub(crate) const ALL: [Self; 3] = [Self::Holding, Self::Product, Self::Account];

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Holding => COMPOSITION_GROUP_HOLDING,
            Self::Product => COMPOSITION_GROUP_PRODUCT,
            Self::Account => COMPOSITION_GROUP_ACCOUNT,
        }
    }

    pub(crate) const fn value(self) -> &'static str {
        match self {
            Self::Holding => "holding",
            Self::Product => "product",
            Self::Account => "account",
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
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::MarketValue => MARKET_VALUE,
            Self::Dividend => COMPOSITION_DIVIDEND_BASIS,
        }
    }

    pub(crate) const fn value(self) -> &'static str {
        match self {
            Self::MarketValue => "market_value",
            Self::Dividend => "dividend",
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
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Stock => DOMESTIC_STOCK,
            Self::Fund => MUTUAL_FUND,
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
            href: Some(holding_stock_href(&view.code)),
        })
        .chain(funds.iter().map(|view| CompositionSource {
            name: view.name.clone(),
            sub: view.course.clone().unwrap_or_default(),
            account: Some(view.account.clone()),
            product: CompositionProduct::Fund,
            href: view.id.as_ref().map(fund_holding_href),
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
            if let Some(&position) = positions.get(&key) {
                position
            } else {
                let position = entries.len();
                positions.insert(key.clone(), position);
                position
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
                        .map_or(MISSING, account_display_name)
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

/// ドーナツ横の凡例に出す上位の件数。比率の大きい順に必ずここまで出し、
/// 残りは「ほか N 件」にまとめる
pub const COMPOSITION_LEGEND_MAX: usize = 5;

/// 区分の色番号。`[0, 2, 4, 1, 3]` を回して隣り合う区分の色が並ばないようにする
pub const fn composition_color(position: usize) -> usize {
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

/// ドーナツ横の凡例に分ける。比率の降順のまま先頭から最大件数を取り、
/// 残りの件数を返す。5件以下なら残りは0
pub fn composition_legend_split(total_rows: usize) -> (usize, usize) {
    let visible = total_rows.min(COMPOSITION_LEGEND_MAX);
    (visible, total_rows.saturating_sub(visible))
}

/// ドーナツの外形。正方形で凡例と横に並べる
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DonutGeometry {
    pub width: f64,
    pub height: f64,
    pub center_x: f64,
    pub center_y: f64,
    pub radius: f64,
    pub stroke: f64,
}

impl DonutGeometry {
    pub const SQUARE: Self = Self {
        width: 220.0,
        height: 220.0,
        center_x: 110.0,
        center_y: 110.0,
        radius: 70.0,
        stroke: 30.0,
    };

    /// 切れの外縁の半径
    pub fn outer(&self) -> f64 {
        self.radius + self.stroke / 2.0
    }

    /// 切れの内縁の半径
    pub fn inner(&self) -> f64 {
        self.radius - self.stroke / 2.0
    }
}
