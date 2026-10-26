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
/// 平均取得単価(CSV エクスポート)
pub const AVERAGE_UNIT_PRICE: &str = "平均取得単価";
/// 取得額(CSV エクスポートの列)
pub const ACQUISITION_AMOUNT: &str = "取得額";
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
/// ログインが要る画面に出す案内の見出し
pub const LOGIN_REQUIRED_TITLE: &str = "ログインしてください";
/// ログインが要る画面に出す案内の説明
pub const LOGIN_REQUIRED_DESCRIPTION: &str = "保有資産と取引明細はログイン後に確認できます。";
/// Googleアカウントでのログイン操作
pub const GOOGLE_LOGIN: &str = "Googleでログイン";
/// 検索を実行するボタン
pub const SEARCH: &str = "検索";
/// 絞り込み中の表示(注記の行に出す)
pub const FILTERING: &str = "絞り込み中";
/// 検索を実行しているときのボタン名
pub const SEARCH_LOADING: &str = "検索中";
/// 銘柄検索の検索対象(検索欄の文言に使う)
pub const STOCK_SEARCH_TARGET: &str = "銘柄コードまたは銘柄名";
/// 銘柄検索の実行ボタン(実行中でないときの名前)
pub const STOCK_SEARCH_ACTION: &str = "銘柄を検索";
/// CSV 最新取得の実行中表示
pub const CSV_SYNC_RUNNING: &str = "取得中...";
/// 種類ごとの進行がまだ始まっていない表示
pub const CSV_SYNC_WAITING: &str = "待機中";
/// 拡張機能がログインを待っているときの案内
pub const CSV_SYNC_LOGIN_WAIT: &str = "楽天証券のタブでログインしてください";
/// 拡張機能との通信が切れたときのエラー
pub const CSV_SYNC_DISCONNECTED: &str = "拡張機能との通信が切れました";
/// 拡張機能が見つからないときの案内
pub const CSV_SYNC_MISSING_TITLE: &str = "CSV 取得用の拡張機能が見つかりません";
/// 拡張機能が見つからないときの説明
pub const CSV_SYNC_MISSING_DESCRIPTION: &str = "最新の CSV を受け取るには拡張機能が必要です。";
/// 拡張機能の導入リンク
pub const CSV_SYNC_STORE_LINK: &str = "Chrome ウェブストアで拡張機能を導入する";
/// 「ファイルの URL へのアクセス」が無効なときの見出し
pub const CSV_SYNC_FILE_ACCESS_TITLE: &str = "ファイルの URL へのアクセスが許可されていません";
/// 「ファイルの URL へのアクセス」が無効なときの手順
pub const CSV_SYNC_FILE_ACCESS_STEPS: &[&str] = &[
    "chrome://extensions/ を開く",
    "「楽天証券 CSV ダウンロード拡張機能」の「詳細」を開く",
    "「ファイルの URL へのアクセスを許可する」を ON にする",
];
/// 国内株式の保有残高(CSV 最新取得の種類名)
pub const CSV_KIND_STOCK_BALANCE: &str = "国内株式の保有";
/// 投資信託の保有残高(CSV 最新取得の種類名)
pub const CSV_KIND_FUND_BALANCE: &str = "投資信託の保有";
/// 国内株式の取引履歴(CSV 最新取得の種類名)
pub const CSV_KIND_STOCK_TRADE: &str = "国内株式の取引";
/// 投資信託の取引履歴(CSV 最新取得の種類名)
pub const CSV_KIND_FUND_TRADE: &str = "投資信託の取引";
/// 種類ごとの取り込みの実行中表示
pub const CSV_SYNC_IMPORTING: &str = "取り込み中...";
/// 件数激減の確認を出して回答を待っている表示
pub const CSV_SYNC_CONFIRMING: &str = "確認待ち";
/// 受け取った内容から送信ファイルを作れなかったときの文言
pub const CSV_SYNC_FILE_FAILED: &str = "ファイルを作成できませんでした";
/// 確認への「はい」の回答
pub const YES: &str = "はい";
/// 確認への「いいえ」の回答
pub const NO: &str = "いいえ";
/// 取り込まなかった理由: 空(ヘッダーだけ)
pub const CSV_SKIP_EMPTY: &str = "データ行がありません";
/// 取り込まなかった理由: HTML を受け取った
pub const CSV_SKIP_HTML: &str = "CSV ではない内容(HTML)を受け取りました";
/// 取り込まなかった理由: 文字化け
pub const CSV_SKIP_MOJIBAKE: &str = "文字化けしています";
/// 取り込まなかった理由: 必須列・見出しなし
pub const CSV_SKIP_COLUMNS: &str = "必要な列が見つかりません";
/// 取り込まなかった理由: 検証で有効行0件
pub const CSV_SKIP_NO_VALID: &str = "取り込める行がありませんでした";
/// 取り込まなかった理由: 全件置換で行エラーあり
pub const CSV_SKIP_ROW_ERRORS: &str = "エラーのある行が含まれます";
/// 取り込まなかった理由: 確認で取り込まない選択
pub const CSV_SKIP_DECLINED: &str = "確認で取り込みをやめました";
/// 取り込まなかった理由: 保存済み件数を取れなかった
pub const CSV_SKIP_COUNT_UNKNOWN: &str = "保存済みの件数を確認できませんでした";

/// 受け取った行数の書き方(「受け取った(12行)」)
pub fn csv_received(lines: usize) -> String {
    format!("受け取った({lines}行)")
}

/// 種類ごとの取得失敗の書き方
pub fn csv_sync_failed(reason: &str) -> String {
    format!("失敗: {reason}")
}

/// 種類ごとの取り込み結果の書き方(「取り込んだ(32件)」)
pub fn csv_imported(inserted: usize, skipped: usize, errors: usize) -> String {
    let mut text = format!("取り込んだ({inserted}件");
    if skipped > 0 {
        text.push_str(&format!("・{skipped}件スキップ"));
    }
    if errors > 0 {
        text.push_str(&format!("・{errors}件エラー"));
    }
    format!("{text})")
}

/// 取り込まなかった理由の書き方
pub fn csv_sync_skipped(reason: &str) -> String {
    format!("取り込まなかった({reason})")
}

/// 件数激減の確認文。「はい/いいえ」で答える
pub fn csv_sync_plunge_question(kind: &str, new_rows: usize, saved_rows: usize) -> String {
    format!(
        "{kind}: 件数が大きく減っています(保存済み{saved_rows}件 → {new_rows}件)。取り込みますか?"
    )
}

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

/// 絞り込み中の件数(注記の行に出す「絞り込み中(4件 / 77件)」)
pub fn filtering_count(filtered: usize, total: usize) -> String {
    format!("{FILTERING}({} / {})", count(filtered), count(total))
}

/// 基準価額の自動更新の注記。自動更新の対象件数・取得件数・失敗件数と、
/// 自動更新の対象外のファンド(他社のファンド)は CSV で更新することを示す
pub fn nav_update_note(
    // 自動更新の時刻("10/22 20:00")
    updated_at: &str,
    // 自動更新の対象のファンド数
    target: usize,
    // 取得できたファンド数
    fetched: usize,
    // 取得できなかったファンド数
    failed: usize,
    // 失敗の理由
    failure_reason: Option<&str>,
    // 自動更新の対象外のファンド数
    csv_only: usize,
) -> String {
    let mut counts = format!("対象 {target}件(取得 {fetched}件");
    if failed > 0 {
        counts.push_str(&format!("・失敗 {failed}件"));
        if let Some(reason) = failure_reason {
            counts.push_str(&format!(": {reason}"));
        }
    }
    counts.push(')');
    format!("自動更新 {updated_at}・{counts}・他社のファンド {csv_only}件は CSV で更新")
}

/// 基準価額の自動更新がまだ走っていないときの注記
pub const NAV_UPDATE_NOT_YET: &str = "自動更新: まだ取得されていません";

/// 最後の CSV 取り込みの注記。「N日前」は暦日の差
pub fn last_csv_import_note(date: &str, days_ago: i64) -> String {
    if days_ago <= 0 {
        format!("最後の CSV 取り込み: {date}(今日)")
    } else {
        format!("最後の CSV 取り込み: {date}({days_ago}日前)")
    }
}

/// 集計は一覧と同じ対象であることの注記
pub const SUMMARY_SAME_TARGET: &str =
    "集計は一覧と同じ対象です(検索・条件で絞り込むと集計も変わります)";

/// 対象期間の注記(明細の最初と最後の日)
pub fn target_period_note(from: &str, to: &str) -> String {
    format!("対象期間: {from} 〜 {to}")
}

/// CSV管理: 取得カードの見出し
pub const CSV_MANAGE_FETCH_TITLE: &str = "楽天証券から最新のデータを取得";
/// CSV管理: 取得カードの説明
pub const CSV_MANAGE_FETCH_HINT: &str = "拡張機能を呼び出して、楽天証券の最新 CSV を受け取ります。";
/// CSV管理: 取得ボタン(通常)
pub const CSV_MANAGE_FETCH_BUTTON: &str = "最新を取得する";
/// CSV管理: 取得ボタン(実行中)
pub const CSV_MANAGE_FETCH_RUNNING: &str = "取得中...";
/// CSV管理: 取得前の初期メッセージ
pub fn csv_manage_last_fetched(date: &str, time: &str) -> String {
    format!("最終取得: {date} {time}")
}
/// CSV管理: 取得完了メッセージ(成功・失敗の組み合わせ)
pub fn csv_manage_fetch_result(updated: usize, failed: usize) -> String {
    let mut parts = Vec::new();
    if updated > 0 {
        parts.push(format!("{updated}種類を更新しました"));
    }
    if failed > 0 {
        parts.push(format!("{failed}種類は取得できませんでした"));
    }
    parts.join("・")
}
/// CSV管理: 表ヘッダー「データの種類」
pub const CSV_MANAGE_COL_KIND: &str = "データの種類";
/// CSV管理: 表ヘッダー「保存済み」
pub const CSV_MANAGE_COL_COUNT: &str = "保存済み";
/// CSV管理: 表ヘッダー「状態」
pub const CSV_MANAGE_COL_STATUS: &str = "状態";
/// CSV管理: 表ヘッダー「メニュー」
pub const CSV_MANAGE_COL_MENU: &str = "⋯";
/// CSV管理: 状態「今日 HH:MM に更新」(緑)
pub fn csv_manage_status_updated_today(time: &str) -> String {
    format!("● 今日 {time} に更新")
}
/// CSV管理: 状態「N日前(M/D HH:MM)に更新」(薄い色)
pub fn csv_manage_status_updated_days_ago(days: i64, date: &str, time: &str) -> String {
    format!("○ {days}日前({date} {time})に更新")
}
/// CSV管理: 状態「取得できず・楽天証券のタブを再読み込みしてください 詳細」(赤)
pub const CSV_MANAGE_STATUS_FAILED: &str =
    "取得できず・楽天証券のタブを再読み込みしてください 詳細";
/// CSV管理: 状態「取り込まなかった(理由)」(薄い色)
pub fn csv_manage_status_skipped(reason: &str) -> String {
    format!("取り込まなかった({reason})")
}
/// CSV管理: 状態「まだ取得していません」(薄い色)
pub const CSV_MANAGE_STATUS_NEVER: &str = "まだ取得していません";
/// CSV管理: 取得失敗の詳細ラベル
pub const CSV_MANAGE_FAILED_DETAIL_LABEL: &str = "詳細";
/// CSV管理: メニュー項目「楽天証券から再取得」
pub const CSV_MANAGE_MENU_REFETCH: &str = "楽天証券から再取得";
/// CSV管理: メニュー項目「ファイルから取り込み」
pub const CSV_MANAGE_MENU_FILE_IMPORT: &str = "ファイルから取り込み";
/// CSV管理: メニュー項目「削除」(赤)
pub const CSV_MANAGE_MENU_DELETE: &str = "削除";
/// CSV管理: 削除確認モーダルのタイトル
pub fn csv_manage_delete_confirm_title(kind: &str) -> String {
    format!("{kind}を削除しますか?")
}
/// CSV管理: 削除確認モーダルの説明(件数付き)。range は保存データの期間
pub fn csv_manage_delete_confirm_description(
    kind: &str,
    range: Option<&str>,
    count: usize,
) -> String {
    let range = range.map(|range| format!("({range})")).unwrap_or_default();
    format!("{kind}の保存済み {count}件{range}を削除します。元に戻せません")
}
/// CSV管理: 削除確認モーダルの実行ボタン
pub fn csv_manage_delete_confirm_button(count: usize) -> String {
    format!("{count}件を削除する")
}
/// CSV管理: 保存方式「追加」
pub const CSV_MANAGE_SAVE_MODE_APPEND: &str = "追加";
/// CSV管理: 保存方式「全件置換」
pub const CSV_MANAGE_SAVE_MODE_REPLACE: &str = "全件置換";

#[cfg(test)]
mod tests;
