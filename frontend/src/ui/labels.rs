//! 画面の用語集。ホーム・資産管理・取引明細・詳細・CSVプレビュー・設定の全画面が
//! ここにある定数と関数を使う。画面側に同じ意味の文言を直接書かない。
//! 新しい文言はここに足してから使う( docs/ui-design.md の「用語と書式」)

use crate::ui::format::format_price_date;

/// 評価額(保有資産の時価評価額)
pub const MARKET_VALUE: &str = "評価額";
/// 評価額の合計(資産管理の集計帯)
pub const MARKET_VALUE_TOTAL: &str = "評価額合計";
/// 評価損益(評価額 − 取得総額)
pub const VALUATION_PROFIT_LOSS: &str = "評価損益";
/// 前日比(前営業日との差)
pub const DAY_CHANGE: &str = "前日比";
/// 取得総額
pub const TOTAL_ACQUISITION: &str = "取得総額";
/// 年間配当
pub const ANNUAL_DIVIDEND: &str = "年間配当";
/// 年間配当(会社予想)
pub const ANNUAL_DIVIDEND_FORECAST: &str = "年間配当(予想)";
/// 年間予想配当(税引前)(ホームの集計帯)
pub const ANNUAL_EXPECTED_DIVIDEND: &str = "年間予想配当(税引前)";
/// 年間配当参考額(保有銘柄詳細)
pub const ANNUAL_DIVIDEND_REFERENCE: &str = "年間配当参考額";
/// 取得額利回り(一覧・集計)
pub const ACQUISITION_YIELD: &str = "取得額利回り";
/// 取得額基準利回り(保有銘柄詳細)
pub const ACQUISITION_BASIS_YIELD: &str = "取得額基準利回り";
/// 1株配当
pub const PER_SHARE_DIVIDEND: &str = "1株配当";
/// 1株配当(スマホの行の短い表記)
pub const PER_SHARE_DIVIDEND_COMPACT: &str = "1株";
/// 取得額利回り(スマホの行の短い表記)
pub const ACQUISITION_YIELD_COMPACT: &str = "利回り";
/// 一株配当(取引明細の配当情報カード)
pub const PER_SHARE_DIVIDEND_CARD: &str = "一株配当";
/// 平均取得価格(取引明細の配当情報カード)
pub const AVERAGE_PURCHASE_PRICE: &str = "平均取得価格";
/// 保有数量(取引明細の配当情報カード)
pub const HOLDING_QUANTITY_CARD: &str = "保有数量(株)";
/// 配当金額と配当利回り(取引明細の配当情報カード)
pub const DIVIDEND_AMOUNT_AND_YIELD: &str = "配当金額 (配当利回り)";
/// 税引後と累積利回り(取引明細の配当情報カード)
pub const AFTER_TAX_AND_YIELD: &str = "税引後 (累積利回り)";
/// 配当(一覧の行に出すabr表記)
pub const DIVIDEND: &str = "配当";
/// 配当金(取引明細の配当金タブ・列見出し)
pub const DIVIDEND_AMOUNT: &str = "配当金";
/// 入金日(配当の入金日)
pub const PAYMENT_DATE: &str = "入金日";
/// 約定日(売買・解約の成立日)
pub const TRADE_DATE: &str = "約定日";
/// ファンド名
pub const FUND_NAME: &str = "ファンド名";
/// 売却単価
pub const SELLING_UNIT_PRICE: &str = "売却単価";
/// 売却額
pub const SELLING_AMOUNT: &str = "売却額";
/// 取得価額
pub const ACQUISITION_PRICE: &str = "取得価額";
/// 解約単価
pub const REDEMPTION_UNIT_PRICE: &str = "解約単価";
/// 解約額
pub const REDEMPTION_AMOUNT: &str = "解約額";
/// 数量
pub const QUANTITY: &str = "数量";
/// 保有数量(CSV プレビューの表の見出し)
pub const HOLDING_QUANTITY: &str = "保有数量";
/// 時価評価額(CSV プレビューの表の見出し)
pub const TIMED_MARKET_VALUE: &str = "時価評価額";
/// 税額
pub const TAX: &str = "税額";
/// 税引後
pub const AFTER_TAX: &str = "税引後";
/// 実現損益(売却・解約の損益)
pub const REALIZED_PROFIT_LOSS: &str = "実現損益";
/// 銘柄名
pub const SECURITY_NAME: &str = "銘柄名";
/// 銘柄コード
pub const SECURITY_CODE: &str = "銘柄コード";
/// 銘柄コードの列見出し(列の幅が狭い表で短くする)
pub const CODE_COLUMN: &str = "コード";
/// 銘柄名の列見出し(列の幅が狭い表で短くする)
pub const NAME_COLUMN: &str = "銘柄";
/// 銘柄とファンドを併せる列見出し
pub const SECURITY_AND_FUND: &str = "銘柄・ファンド";
/// 国内株式
pub const DOMESTIC_STOCK: &str = "国内株式";
/// 投資信託
pub const MUTUAL_FUND: &str = "投資信託";
/// 口座
pub const ACCOUNT: &str = "口座";
/// 商品
pub const PRODUCT: &str = "商品";
/// 期間
pub const PERIOD: &str = "期間";
/// 分配金コース(投資信託の分配金の受取方法)
pub const DISTRIBUTION_COURSE: &str = "分配金コース";
/// 分配金(投資信託の分配金コースの列見出し)
pub const DISTRIBUTION: &str = "分配金";
/// ファンド名の列見出し
pub const FUND: &str = "ファンド";
/// 保有口数(投資信託の口数の列見出し)
pub const FUND_SHARES: &str = "保有口数";
/// 現在値
pub const CURRENT_PRICE: &str = "現在値";
/// 価格基準日(株価の基準日)
pub const PRICE_AS_OF: &str = "価格基準日";
/// 取得単価
pub const AVERAGE_PRICE: &str = "取得単価";
/// 平均取得単価(AI 総評プロンプト・CSV エクスポート)
pub const AVERAGE_UNIT_PRICE: &str = "平均取得単価";
/// 取得額(CSV エクスポートの列)
pub const ACQUISITION_AMOUNT: &str = "取得額";
/// 取得額合計(AI 総評プロンプト)
pub const ACQUISITION_AMOUNT_TOTAL: &str = "取得額合計";
/// 保有株数(AI 総評プロンプト)
pub const SHARES: &str = "保有株数";
/// 保有数(CSV エクスポートの列)
pub const HOLDING_COUNT: &str = "保有数";
/// 単価(CSV エクスポートの列)
pub const UNIT_PRICE: &str = "単価";
/// 売却・解約額(CSV エクスポートの列)
pub const SELLING_OR_REDEMPTION_AMOUNT: &str = "売却・解約額";
/// 種別(CSV エクスポートの列)
pub const KIND: &str = "種別";
/// 基準価額(投資信託の1口あたりの価格)
pub const BASE_PRICE: &str = "基準価額";
/// 基準価額の前日比
pub const BASE_PRICE_DAY_CHANGE: &str = "基準価額(前日比)";
/// 基準価額の前月比
pub const BASE_PRICE_MONTH_CHANGE: &str = "基準価額(前月比)";
/// 基準価額の基準日
pub const BASE_PRICE_AS_OF: &str = "基準価額の基準日";
/// CSV を取り込んだ日
pub const CSV_IMPORTED_AT: &str = "CSV 取込日";
/// 平均取得価額(投資信託)
pub const AVERAGE_ACQUISITION_PRICE: &str = "平均取得価額";
/// トータルリターン(投資信託の累積の損益)
pub const TOTAL_RETURN: &str = "トータルリターン";
/// 値が無いときの表示
pub const MISSING: &str = "—";
/// 基準価額の更新を待つ行の表示
pub const PENDING_UPDATE: &str = "更新待ち";
/// 条件に合う行が無いときの空状態の見出し
pub const NO_MATCH_TITLE: &str = "条件に合う保有がありません";
/// 条件に合う行が無いときの空状態の説明
pub const NO_MATCH_DESCRIPTION: &str = "検索条件を変更するか、検索条件を解除してください。";
/// 検索条件の解除(操作のボタン)
pub const CLEAR_SEARCH_CONDITIONS: &str = "検索条件を解除";

/// 統一前の語。画面の文言には使わない(括弧内は現在の用語)。
/// `labels.rs` の単体テストが `frontend/src` の他のソースに現れないことを確かめる
#[cfg(test)]
pub const FORBIDDEN_TERMS: &[(&str, &str)] = &[
    ("含み損益", VALUATION_PROFIT_LOSS),
    ("評価額(投資信託)", MARKET_VALUE),
    ("絞り込み", "検索"),
];

/// 件数の書き方
pub fn count(count: usize) -> String {
    format!("{count}件")
}

/// 検索欄の入力欄に出す文言。対象名だけを引数にして画面ごとに使い回す
pub fn search_placeholder(target: &str) -> String {
    format!("{target}で検索")
}

/// 条件を選ぶ小窓の見出し。対象名だけを引数にして画面ごとに使い回す
pub fn pick_title(target: &str) -> String {
    format!("{target}を選ぶ")
}

/// グループ見出しの件数(「2026年10月 · 3件」)
pub fn group_count(label: &str, total: usize) -> String {
    format!("{label} · {}", count(total))
}

/// 基準日の書き方(「10/21 時点」)
pub fn price_as_of(date: &str) -> String {
    format!("{} 時点", format_price_date(date))
}

#[cfg(test)]
mod tests;
