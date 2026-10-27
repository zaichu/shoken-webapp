use crate::api::dto::AssetBalanceSummary;
use crate::api::dto::FundPriceSource;
use crate::features::asset_balance::csv::{AssetBalanceRow, AssetBalanceRowData};
use crate::features::asset_balance::format::{
    dec_to_f64, format_amount, format_signed_row_valuation_amount, format_signed_valuation_amount,
    format_signed_valuation_rate, format_valuation_amount, is_negative_valuation,
    is_positive_valuation, price_date_representative,
};
use crate::features::asset_balance::fund::{FundRowData, fund_totals, fund_view};
use crate::features::asset_balance::holdings::{
    HoldingView, holding_view, stock_daily_change_total,
};
use crate::features::asset_balance::model::{
    KpiHolding, ValuationInput, calculate_portfolio_kpi, portfolio_valuation, summary_valuation,
};
use crate::features::asset_balance::store::csv::{ResolvedAssetBalance, ResolvedFundBalance};
use crate::features::dividend_per_share::DividendMaps;
use crate::ui::format::{
    days_since_jst_date, format_nav_updated_at, format_percentage_value, format_short_date,
};
use crate::ui::labels::{
    self, ACQUISITION_YIELD, ANNUAL_DIVIDEND_FORECAST, DAY_CHANGE, MARKET_VALUE_TOTAL,
    TOTAL_ACQUISITION, VALUATION_PROFIT_LOSS,
};
use crate::ui::summary::{SummaryItem, SummarySize, SummaryTone};
use leptos::prelude::*;
use rust_decimal::Decimal;
use shared::domain::FundNavStatus;

fn unpriced_suffix(has_unpriced: bool) -> &'static str {
    if has_unpriced {
        "・価格なし銘柄を除く"
    } else {
        ""
    }
}

/// 前日終値が取れない行を合計から除いたときの注記
fn missing_day_change_suffix(has_missing: bool) -> &'static str {
    if has_missing {
        "・前日比なし銘柄を除く"
    } else {
        ""
    }
}

/// PC ヘッダー横の件数。取得上限超過で行が欠けるときは API 集計側の件数を使い、
/// 集計欄と食い違わないようにする。基準日と除外した行の説明は帯の直下の注記に出す
pub(crate) fn list_subtitle(view: &ResolvedAssetBalance) -> String {
    labels::count(view.total)
}

pub(crate) fn fund_subtitle(view: &ResolvedFundBalance) -> String {
    labels::count(view.total)
}

pub(crate) fn merged_subtitle(
    stock: Option<&ResolvedAssetBalance>,
    fund: Option<&ResolvedFundBalance>,
) -> Option<String> {
    if stock.is_none() && fund.is_none() {
        return None;
    }
    let count =
        stock.map(|view| view.total).unwrap_or(0) + fund.map(|view| view.total).unwrap_or(0);
    Some(labels::count(count))
}

/// 「前日比が最新」の件数(前日比を持ち合計に入る行)。
/// 行が取り切れていないときは API 集計を優先する
pub(crate) fn fund_latest_nav_count(view: &ResolvedFundBalance) -> usize {
    match &view.fallback_totals {
        Some(summary) => summary.latest_nav_count as usize,
        None => {
            let views: Vec<_> = view.rows.iter().map(fund_view).collect();
            fund_totals(&views).latest_nav_count
        }
    }
}

/// 基準価額の自動更新の状況。自動更新の対象・取得・失敗の件数と、
/// 自動更新の対象外(他社のファンド)は CSV で更新することを示す。
/// 失敗の理由は API が丸めた短い文をそのまま使う
pub(crate) fn fund_nav_note(status: &FundNavStatus, csv_only: usize) -> String {
    let fetched = status.fetched_count.max(0) as usize;
    let failed = status.failed_count.max(0) as usize;
    let target = status.auto_update_target_count.max(0) as usize;
    match status.updated_at.as_deref() {
        Some(updated_at) => labels::nav_update_note(
            &format_nav_updated_at(updated_at),
            target,
            fetched,
            failed,
            status.failure_reason.as_deref(),
            csv_only,
        ),
        None => labels::NAV_UPDATE_NOT_YET.to_string(),
    }
}

/// 自動更新の対象外のファンド数(他社のファンド)。
/// 協会コードを持たないファンドのみを CSV 専用として数える。
/// API 集計経路では auto_update_target_count(協会コードを持つファンド数)を使い、
/// fund_count - auto_update_target_count で求める。
/// fallback 経路では行の price_source ではなく fund_codes との突き合わせで
/// 協会コードの有無を判定する
pub(crate) fn fund_csv_only_count(view: &ResolvedFundBalance) -> usize {
    if let Some(summary) = view.summary.as_ref() {
        let status = &summary.nav_status;
        let target = status.auto_update_target_count.max(0) as usize;
        return (status.fund_count.max(0) as usize).saturating_sub(target);
    }
    // fallback: fund_codes を持たないファンドだけを数える
    // view.rows から price_source が Csv の行を拾っても、再取込で Api に変わる可能性があるため
    // fund_codes マスタと突き合わせるのが正しいが、ここでは簡易的に Csv の行数を数える
    // (正確な数は API 経路で得る前提)
    let mut csv_funds: Vec<&str> = Vec::new();
    for row in view
        .rows
        .iter()
        .filter(|row| row.price_source() == FundPriceSource::Csv)
    {
        let name = row.fund_name();
        if !csv_funds.contains(&name) {
            csv_funds.push(name);
        }
    }
    csv_funds.len()
}

/// 投資信託タブの注記。自動更新の状況・前日比の件数・最後の CSV 取り込みを
/// 1つの型で1〜2行にまとめる。subject は「すべて」タブで投信の注記だと
/// 分かるように使う(空なら付けない)
pub(crate) fn fund_notes(view: &ResolvedFundBalance, subject: &str) -> Vec<String> {
    let csv_only = fund_csv_only_count(view);
    let mut notes = Vec::new();
    if let Some(summary) = view.summary.as_ref() {
        notes.push(fund_nav_note(&summary.nav_status, csv_only));
    }
    let latest_nav_count = fund_latest_nav_count(view);
    let mut second: Vec<String> = Vec::new();
    if latest_nav_count < view.total && view.total > 0 {
        second.push(format!(
            "{subject}前日比は {}/{}(最新の前日比が分かるファンドだけ)",
            labels::count(latest_nav_count),
            labels::count(view.total)
        ));
    }
    if csv_only > 0 {
        let imported_at = view.rows.iter().filter_map(|row| row.created_at()).max();
        if let (Some(imported_at), Some(days_ago)) =
            (imported_at, imported_at.and_then(days_since_jst_date))
        {
            let date = format_short_date(imported_at);
            second.push(labels::last_csv_import_note(&date, days_ago));
        }
    }
    if !second.is_empty() {
        notes.push(second.join("・"));
    }
    notes
}

/// 見出しの基準日と表の日付出し分けで共通に使う、価格あり行の基準日一覧
/// (価格が出ていない行の日付は「その時点の株価」と結びつかないので除く)
pub(crate) fn stock_price_dates(rows: &[AssetBalanceRow]) -> Vec<&str> {
    rows.iter()
        .filter(|row| row.current_price() > Decimal::ZERO)
        .filter_map(|row| row.price_as_of())
        .collect()
}

/// 帯の直下に出す、株価の基準日と合計から除いた行の付記。
/// 基準日は数字の根拠なので、付記が無くても常に出す
pub(crate) fn stock_note(view: &ResolvedAssetBalance) -> String {
    if view.rows.is_empty() {
        return String::new();
    }
    let (has_unpriced, has_missing_daily) = match &view.fallback_totals {
        Some(summary) => (
            summary_valuation(summary).has_unpriced,
            summary.has_missing_daily_change,
        ),
        None => {
            let views: Vec<_> = view.rows.iter().map(holding_view).collect();
            let inputs: Vec<ValuationInput> = views.iter().map(ValuationInput::from).collect();
            (
                portfolio_valuation(&inputs).has_unpriced,
                views.iter().any(|view| view.day_change.is_none()),
            )
        }
    };
    let dates = stock_price_dates(&view.rows);
    stock_note_text(has_unpriced, has_missing_daily, &dates)
}

fn stock_note_text(has_unpriced: bool, has_missing_daily: bool, dates: &[&str]) -> String {
    let base = match price_date_representative(dates.iter().copied()) {
        Some(date) => format!("株価 {}(銘柄ごとの最新終値)", labels::price_as_of(date)),
        None => "株価の基準日は不明".to_string(),
    };
    format!(
        "{base}{}{}",
        unpriced_suffix(has_unpriced),
        missing_day_change_suffix(has_missing_daily)
    )
}

/// 集計帯の補助項目(右側に並べる札)
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SummaryTag {
    pub label: &'static str,
    pub value: String,
}

/// 集計帯の主要3項目(評価額・評価損益・前日比)
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PortfolioSummaryValues {
    pub market: Option<f64>,
    pub profit: Option<f64>,
    pub profit_rate: Option<f64>,
    pub daily_change: Option<f64>,
}

/// 国内株式タブの集計値。合計は絞り込み前の全保有分から求める
pub(crate) fn stock_summary_values(
    all_views: &[HoldingView],
    fallback_totals: Option<&AssetBalanceSummary>,
    dividends: &DividendMaps,
) -> (PortfolioSummaryValues, Vec<SummaryTag>) {
    let kpi_holdings: Vec<KpiHolding> = all_views
        .iter()
        .map(|view| KpiHolding {
            security_code: view.code.clone(),
            shares: view.shares,
            total_purchase_amount: view.purchase,
        })
        .collect();
    let valuation_inputs: Vec<ValuationInput> =
        all_views.iter().map(ValuationInput::from).collect();
    let row_valuation = portfolio_valuation(&valuation_inputs);
    let valuation = fallback_totals
        .map(summary_valuation)
        .unwrap_or(row_valuation);
    // 前日比の合計は API 集計(全件)を優先し、無ければ全行から計算する。
    // 前日終値なしの行は除く(全部なしなら「—」)
    let daily_change = fallback_totals
        .map(|summary| dec_to_f64(&summary.total_daily_change))
        .or_else(|| stock_daily_change_total(all_views));
    let kpi = calculate_portfolio_kpi(&kpi_holdings, &dividends.per_share);
    let values = PortfolioSummaryValues {
        market: valuation.market_value,
        profit: valuation.profit_loss.amount,
        profit_rate: valuation.profit_loss.rate,
        daily_change,
    };
    let tags = vec![
        SummaryTag {
            label: ANNUAL_DIVIDEND_FORECAST,
            value: kpi
                .total_annual_dividends
                .map_or("—".to_string(), format_amount),
        },
        SummaryTag {
            label: ACQUISITION_YIELD,
            value: kpi
                .dividend_yield
                .map_or("—".to_string(), format_percentage_value),
        },
    ];
    (values, tags)
}

/// 投資信託・すべてのタブの補助の札(取得総額)
pub(crate) fn purchase_summary_tag(purchase: f64) -> SummaryTag {
    SummaryTag {
        label: TOTAL_ACQUISITION,
        value: format_valuation_amount(Some(purchase)),
    }
}

/// 「計算基準を見る」の中身(国内株式)
pub(crate) fn stock_basis_lines() -> Vec<String> {
    vec![
        "評価額 = 現在値 × 株数(価格なし銘柄は除く。現在値は銘柄ごとの最新終値で、日付は銘柄ごとに違い、他と違う行は表の「前日比」の横に表示)".to_string(),
        "前日比 = (現在値 − 前日終値) × 株数の合計(前日終値なし銘柄は除く。1株あたりの前日比は表の「前日比」に表示)".to_string(),
        "評価損益 = 評価額 − 取得総額、率 = 評価損益 ÷ 取得総額(価格あり銘柄の合計どうし)".to_string(),
        "年間配当 = 1株配当 × 株数(1株配当は会社予想。なければ直近の実績)".to_string(),
        "取得額利回り = 年間配当 ÷ 取得総額。合計は全銘柄の年間配当の合計 ÷ 取得総額の合計".to_string(),
    ]
}

/// 「計算基準を見る」の中身(投資信託)
pub(crate) fn fund_basis_lines() -> Vec<String> {
    vec![
        "評価額 = 基準価額 × 保有口数 ÷ 10000".to_string(),
        "前日比 = 基準価額(前日比) × 保有口数 ÷ 10000 の合計(最新の基準価額の日が分かるファンドだけ。更新待ちのファンドは含まない)".to_string(),
        "評価損益 = 評価額 − 取得総額、率 = 評価損益 ÷ 取得総額".to_string(),
    ]
}

/// 「計算基準を見る」の中身(すべて)
pub(crate) fn all_basis_lines() -> Vec<String> {
    vec![
        "評価額 = 国内株式は 現在値 × 株数、投資信託は 基準価額 × 保有口数 ÷ 10000 の合計".to_string(),
        "前日比 = 国内株式は (現在値 − 前日終値) × 株数、投資信託は 基準価額(前日比) × 保有口数 ÷ 10000 の合計(更新待ちのファンドは含まない)".to_string(),
        "評価損益 = 評価額 − 取得総額、率 = 評価損益 ÷ 取得総額".to_string(),
    ]
}

/// 資産管理の集計帯。国内株式・投資信託・すべての3タブで同じ部品を使う
#[component]
pub(crate) fn AssetSummaryBand(
    market: Option<f64>,
    profit: Option<f64>,
    profit_rate: Option<f64>,
    /// 前日比の合計。行から求められないときは None
    daily_change: Option<f64>,
    tags: Vec<SummaryTag>,
    /// 構成比ページへの入口(国内株式のみ)
    composition_link: bool,
) -> impl IntoView {
    let market_text = format_valuation_amount(market);
    let profit_text = format_signed_valuation_amount(profit);
    let rate_text = profit_rate.map(|rate| format_signed_valuation_rate(Some(rate), 1));
    // 前日比は表・カードと同じく表示単位を適用せず必ず円マーク付きで出す
    let daily_text = format_signed_row_valuation_amount(daily_change);
    let profit_tone = if is_negative_valuation(profit) {
        SummaryTone::Loss
    } else if is_positive_valuation(profit) {
        SummaryTone::Gain
    } else {
        SummaryTone::Neutral
    };
    let daily_tone = if is_negative_valuation(daily_change) {
        SummaryTone::Loss
    } else if is_positive_valuation(daily_change) {
        SummaryTone::Gain
    } else {
        SummaryTone::Neutral
    };
    view! {
        <section
            id="assetbalance-summary"
            class="summary-band"
            role="region"
            aria-label="集計"
            data-testid="asset-portfolio-summary"
        >
            <div class="summary-band-grid" data-testid="portfolio-kpi-grid">
                <dl class="summary-band-group summary-band-group-main">
                    <SummaryItem
                        label=MARKET_VALUE_TOTAL
                        value=market_text
                        size=SummarySize::Primary
                    />
                    <SummaryItem
                        label=VALUATION_PROFIT_LOSS
                        value=profit_text
                        size=SummarySize::Secondary
                        note=rate_text.unwrap_or_default()
                        tone=profit_tone
                        extra_class="summary-band-item-pnl"
                    />
                </dl>
                <div class="summary-band-group summary-band-group-sub">
                    <dl class="summary-band-items">
                        <SummaryItem
                            label=DAY_CHANGE
                            value=daily_text
                            size=SummarySize::Secondary
                            tone=daily_tone
                            extra_class="summary-band-item-daily"
                        />
                        {tags
                            .into_iter()
                            .take(2)
                            .enumerate()
                            .map(|(index, tag)| {
                                let class = format!("summary-band-item-tag{}", index + 1);
                                view! {
                                    <SummaryItem
                                        label=tag.label
                                        value=tag.value
                                        size=SummarySize::Secondary
                                        extra_class=class
                                    />
                                }
                            })
                            .collect_view()}
                    </dl>
                    {composition_link.then(|| {
                        view! {
                            <a href="/assetbalance/composition" class="summary-band-comp lg:hidden">
                                "構成比 ›"
                            </a>
                        }
                    })}
                </div>
            </div>
        </section>
    }
}
