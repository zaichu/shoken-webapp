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

/// スマホの左右列に出す注釈の片側の上限
pub const COMPOSITION_SIDE_ANNOTATION_MAX: usize = 2;

/// 矩形と円が重なるか。ラベルがドーナツの輪に乗らないようにする
fn rect_overlaps_circle(
    rect: (f64, f64, f64, f64),
    center_x: f64,
    center_y: f64,
    radius: f64,
) -> bool {
    let (x, y, width, height) = rect;
    let nearest_x = center_x.clamp(x, x + width);
    let nearest_y = center_y.clamp(y, y + height);
    (nearest_x - center_x).hypot(nearest_y - center_y) < radius
}

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

/// ドーナツ注釈の配置先。スマホは小さく、PCは広く出す
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DonutGeometry {
    pub width: f64,
    pub height: f64,
    pub center_x: f64,
    pub center_y: f64,
    pub radius: f64,
    pub stroke: f64,
    pub label_radius: f64,
}

impl DonutGeometry {
    pub const COMPACT: Self = Self {
        width: 360.0,
        height: 240.0,
        center_x: 180.0,
        center_y: 120.0,
        radius: 70.0,
        stroke: 28.0,
        label_radius: 96.0,
    };
    pub const WIDE: Self = Self {
        width: 640.0,
        height: 300.0,
        center_x: 320.0,
        center_y: 150.0,
        radius: 80.0,
        stroke: 36.0,
        label_radius: 130.0,
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

/// 注釈ラベルの文字寄せ。切れの外側の向きで決める
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AnnotationAnchor {
    Start,
    Middle,
    End,
}

/// ドーナツの注釈1件の配置。ラベル矩形の基準点と引き出し線の折れ点を持つ
#[derive(Clone, Debug, PartialEq)]
pub struct PlacedAnnotation {
    pub index: usize,
    pub text_x: f64,
    pub text_y: f64,
    pub anchor: AnnotationAnchor,
    pub edge_x: f64,
    pub edge_y: f64,
    pub elbow_x: f64,
    pub elbow_y: f64,
    pub end_x: f64,
}

/// 注釈の配置入力。ラベルは短縮済みの「名前 比率」、割合は角度計算用
pub struct AnnotationRequest<'a> {
    pub label: &'a str,
    pub percentage: f64,
    pub computable: bool,
}

/// 注釈ラベルの推定幅。描画と同じ12px基準で見積もる
pub fn annotation_label_width(label: &str) -> f64 {
    const FULL_WIDTH: f64 = 12.0;
    const HALF_WIDTH: f64 = 6.5;
    label
        .chars()
        .map(|c| if c.is_ascii() { HALF_WIDTH } else { FULL_WIDTH })
        .sum()
}

/// ドーナツの注釈配置を決める。比率の大きい順に下限以上を拾い、
/// ラベル矩形が重ならず枠に収まる分だけ残す。中央の件数表示とも重ねない
pub fn place_annotations(
    requests: &[AnnotationRequest<'_>],
    geometry: &DonutGeometry,
    center_label: &str,
) -> Vec<PlacedAnnotation> {
    const LABEL_HEIGHT: f64 = 18.0;
    const MARGIN: f64 = 1.0;
    let mut starts = Vec::with_capacity(requests.len());
    let mut cursor = 0.0;
    for request in requests {
        starts.push(cursor);
        cursor += request.percentage;
    }
    let mut placed: Vec<(f64, f64, f64, f64)> = Vec::new();
    let center_half_width = center_label.chars().count() as f64 * 9.0 + 8.0;
    let center_rect = (
        geometry.center_x - center_half_width,
        geometry.center_y - 18.0,
        center_half_width * 2.0,
        36.0,
    );
    placed.push(center_rect);
    let mut selected = Vec::new();
    for (index, request) in requests.iter().enumerate() {
        if selected.len() >= COMPOSITION_ANNOTATION_MAX {
            break;
        }
        if !request.computable || request.percentage < COMPOSITION_ANNOTATION_MIN_PERCENT {
            continue;
        }
        let mid = starts[index] + request.percentage / 2.0;
        let angle = (mid / 100.0 * 360.0 - 90.0).to_radians();
        let (sine, cosine) = angle.sin_cos();
        let label_x = geometry.center_x + geometry.label_radius * cosine;
        let label_y = geometry.center_y + geometry.label_radius * sine;
        let width = annotation_label_width(request.label);
        let anchor = if cosine >= 0.5 {
            AnnotationAnchor::Start
        } else if cosine <= -0.5 {
            AnnotationAnchor::End
        } else {
            AnnotationAnchor::Middle
        };
        let rect_x = match anchor {
            AnnotationAnchor::Start => label_x.min(geometry.width - width - MARGIN).max(MARGIN),
            AnnotationAnchor::Middle => {
                (label_x - width / 2.0).clamp(MARGIN, geometry.width - width - MARGIN)
            }
            AnnotationAnchor::End => (label_x - width).max(MARGIN),
        };
        let rect_y =
            (label_y - LABEL_HEIGHT / 2.0).clamp(MARGIN, geometry.height - LABEL_HEIGHT - MARGIN);
        if width + 2.0 * MARGIN > geometry.width || LABEL_HEIGHT + 2.0 * MARGIN > geometry.height {
            continue;
        }
        if rect_overlaps_circle(
            (rect_x, rect_y, width, LABEL_HEIGHT),
            geometry.center_x,
            geometry.center_y,
            geometry.outer() + MARGIN,
        ) {
            continue;
        }
        if placed.iter().any(|&(x, y, w, h)| {
            rect_x < x + w && x < rect_x + width && rect_y < y + h && y < rect_y + LABEL_HEIGHT
        }) {
            continue;
        }
        let edge_x = geometry.center_x + geometry.outer() * cosine;
        let edge_y = geometry.center_y + geometry.outer() * sine;
        let bend = geometry.label_radius - 12.0;
        let elbow_x = geometry.center_x + bend * cosine;
        let elbow_y = geometry.center_y + bend * sine;
        let (end_x, text_x, text_y) = match anchor {
            AnnotationAnchor::Start => (rect_x, rect_x, rect_y + LABEL_HEIGHT / 2.0),
            AnnotationAnchor::Middle => {
                let center = rect_x + width / 2.0;
                (center, center, rect_y + LABEL_HEIGHT / 2.0)
            }
            AnnotationAnchor::End => (rect_x + width, rect_x + width, rect_y + LABEL_HEIGHT / 2.0),
        };
        placed.push((rect_x, rect_y, width, LABEL_HEIGHT));
        selected.push(PlacedAnnotation {
            index,
            text_x,
            text_y,
            anchor,
            edge_x,
            edge_y,
            elbow_x,
            elbow_y,
            end_x,
        });
    }
    selected
}

/// スマホ用の注釈配置。ドーナツの左右の外側の列にだけ置き、
/// 比率の大きい順に右・左へ交互に割り当てて片側2つまで残す
pub fn place_side_annotations(
    requests: &[AnnotationRequest<'_>],
    geometry: &DonutGeometry,
    center_label: &str,
) -> Vec<PlacedAnnotation> {
    const LABEL_HEIGHT: f64 = 18.0;
    const MARGIN: f64 = 0.0;
    let mut starts = Vec::with_capacity(requests.len());
    let mut cursor = 0.0;
    for request in requests {
        starts.push(cursor);
        cursor += request.percentage;
    }
    let center_half_width = center_label.chars().count() as f64 * 9.0 + 8.0;
    let mut placed = vec![(
        geometry.center_x - center_half_width,
        geometry.center_y - 18.0,
        center_half_width * 2.0,
        36.0,
    )];
    let mut selected = Vec::new();
    let mut right_count = 0;
    let mut left_count = 0;
    for (index, request) in requests.iter().enumerate() {
        if right_count >= COMPOSITION_SIDE_ANNOTATION_MAX
            && left_count >= COMPOSITION_SIDE_ANNOTATION_MAX
        {
            break;
        }
        if !request.computable || request.percentage < COMPOSITION_ANNOTATION_MIN_PERCENT {
            continue;
        }
        let right_side = (right_count + left_count) % 2 == 0;
        if right_side && right_count >= COMPOSITION_SIDE_ANNOTATION_MAX {
            continue;
        }
        if !right_side && left_count >= COMPOSITION_SIDE_ANNOTATION_MAX {
            continue;
        }
        let mid = starts[index] + request.percentage / 2.0;
        let angle = (mid / 100.0 * 360.0 - 90.0).to_radians();
        let (sine, cosine) = angle.sin_cos();
        let width = annotation_label_width(request.label);
        let edge_x = geometry.center_x + geometry.outer() * cosine;
        let edge_y = geometry.center_y + geometry.outer() * sine;
        let (rect_x, anchor) = if right_side {
            (geometry.width - width, AnnotationAnchor::Start)
        } else {
            (0.0, AnnotationAnchor::End)
        };
        let rect_y =
            (edge_y - LABEL_HEIGHT / 2.0).clamp(MARGIN, geometry.height - LABEL_HEIGHT - MARGIN);
        if width + 2.0 * MARGIN > geometry.width || LABEL_HEIGHT + 2.0 * MARGIN > geometry.height {
            continue;
        }
        if rect_x < MARGIN || rect_x + width > geometry.width - MARGIN {
            continue;
        }
        if rect_overlaps_circle(
            (rect_x, rect_y, width, LABEL_HEIGHT),
            geometry.center_x,
            geometry.center_y,
            geometry.outer() + MARGIN,
        ) {
            continue;
        }
        if placed.iter().any(|&(x, y, w, h)| {
            rect_x < x + w && x < rect_x + width && rect_y < y + h && y < rect_y + LABEL_HEIGHT
        }) {
            continue;
        }
        let end_x = if right_side { rect_x } else { rect_x + width };
        placed.push((rect_x, rect_y, width, LABEL_HEIGHT));
        if right_side {
            right_count += 1;
        } else {
            left_count += 1;
        }
        selected.push(PlacedAnnotation {
            index,
            text_x: end_x,
            text_y: rect_y + LABEL_HEIGHT / 2.0,
            anchor,
            edge_x,
            edge_y,
            elbow_x: end_x,
            elbow_y: rect_y + LABEL_HEIGHT / 2.0,
            end_x,
        });
    }
    selected
}
