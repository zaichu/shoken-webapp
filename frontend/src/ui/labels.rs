//! 画面の用語集。ホーム・資産管理・取引明細・詳細・CSVプレビュー・設定の全画面が
//! ここにある定数と関数を使う。画面側に同じ意味の文言を直接書かない。
//! 新しい文言はここに足してから使う( docs/ui-design.md の「用語と書式」)

use crate::ui::format::format_price_date;

/// 評価額(保有資産の時価評価額)
pub const MARKET_VALUE: &str = "評価額";
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
/// 配当金(取引明細の配当金タブ・列見出し)
pub const DIVIDEND_AMOUNT: &str = "配当金";
/// 配当金(税引前)(取引明細の帯の内訳用)
pub const DIVIDEND_BEFORE_TAX: &str = "配当金(税引前)";
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
/// 税額
pub const TAX: &str = "税額";
/// 税引後
pub const AFTER_TAX: &str = "税引後";
/// 実現損益(売却・解約の損益)
pub const REALIZED_PROFIT_LOSS: &str = "実現損益";
/// 実現損益(税引前)(取引明細の帯の内訳用)
pub const REALIZED_BEFORE_TAX: &str = "実現損益(税引前)";
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
/// 構成比(資産管理の内訳画面)
pub const COMPOSITION: &str = "構成比";
/// 構成比画面への入口(資産管理の一覧見出しの右)
pub const COMPOSITION_VIEW_LINK: &str = "構成比を見る";
/// 構成比の区分切り替え: 銘柄別
pub const COMPOSITION_GROUP_HOLDING: &str = "銘柄別";
/// 構成比の区分切り替え: 商品別
pub const COMPOSITION_GROUP_PRODUCT: &str = "商品別";
/// 構成比の区分切り替え: 口座別
pub const COMPOSITION_GROUP_ACCOUNT: &str = "口座別";
/// 構成比の区分切り替えのラベル
pub const COMPOSITION_GROUP_LABEL: &str = "構成比の区分";
/// 構成比の基準切り替えのラベル
pub const COMPOSITION_BASIS_LABEL: &str = "構成比の基準";
/// 構成比の基準: 年間配当(税引前)
pub const COMPOSITION_DIVIDEND_BASIS: &str = "年間配当(税引前)";
/// 構成比の基準注記(「評価額の比率・合計 ¥… ・保有全体」)
pub fn composition_basis_note(basis: &str, total: &str) -> String {
    format!("{basis}の比率・合計 {total} ・保有全体")
}

/// 構成比で小さい区分をまとめたときの名前(「その他 3銘柄」)
pub fn composition_other(count: usize) -> String {
    format!("その他 {count}銘柄")
}

/// 構成比の中央に出す全体の件数(「36銘柄」)
pub fn composition_total(count: usize) -> String {
    format!("{count}銘柄")
}

/// 構成比の中央に出す区分の数(「2種類」)
pub fn composition_kinds(count: usize) -> String {
    format!("{count}種類")
}

/// 構成比の中央に出す口座の数(「3口座」)
pub fn composition_accounts(count: usize) -> String {
    format!("{count}口座")
}
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
/// 検索条件の解除(操作のボタン)
pub const CLEAR_SEARCH_CONDITIONS: &str = "検索条件を解除";
/// ログインが要る画面に出す案内の見出し
pub const LOGIN_REQUIRED_TITLE: &str = "ログインしてください";
/// Googleアカウントでのログイン操作
pub const GOOGLE_LOGIN: &str = "Googleでログイン";
/// 上バーのアカウントボタン(丸いアイコンの右に出す文字)
pub const ACCOUNT_BUTTON: &str = "アカウント";
/// 検索を実行するボタン
pub const SEARCH: &str = "検索";
/// スマホの検索・並べ替えの入口とシートの見出し
pub const SEARCH_AND_SORT: &str = "検索・並べ替え";
/// シートの下書きと適用中の条件を消すボタン
pub const SHEET_CLEAR: &str = "クリア";
/// ダイアログの変更を捨てて閉じるボタン
pub const DIALOG_CLOSE: &str = "閉じる";
/// 並べ替えの対象の見出し
pub const SORT_TARGET: &str = "並べ替え対象";
/// 並べ替えの順序の見出し
pub const SORT_ORDER_LABEL: &str = "順序";
/// 並べ替えの向き(シートの選択肢)
pub const SORT_HIGH_TO_LOW: &str = "高い順";
/// 並べ替えの向き(シートの選択肢)
pub const SORT_LOW_TO_HIGH: &str = "低い順";
/// 並べ替えの向き(取引明細の日付)
pub const SORT_NEWEST_FIRST: &str = "新しい順";
/// 並べ替えの向き(取引明細の日付)
pub const SORT_OLDEST_FIRST: &str = "古い順";
/// 並べ替えの向き(文字の列の選択肢)
pub const SORT_ALPHA_FIRST: &str = "あいうえお順";
/// 並べ替えの向き(文字の列の選択肢)
pub const SORT_ALPHA_LAST: &str = "逆順";
pub const ASSET_SEARCH_TARGET: &str = "銘柄・ファンド名・コード・口座";
pub const RECEIPT_SEARCH_TARGET: &str = "銘柄名・コード";
/// 絞り込み中の表示(注記の行に出す)
pub const FILTERING: &str = "絞り込み中";
/// 検索を実行しているときのボタン名
pub const SEARCH_LOADING: &str = "検索中";
/// 銘柄検索の検索対象(検索欄の文言に使う)
pub const STOCK_SEARCH_TARGET: &str = "銘柄コードまたは銘柄名";
/// 銘柄検索の実行ボタン(実行中でないときの名前)
pub const STOCK_SEARCH_ACTION: &str = "銘柄を検索";
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
/// 必須列なしの失敗理由に添える受け取った CSV の先頭行
pub fn csv_columns_first_line(line: &str) -> String {
    format!("受け取った先頭行: 「{line}」")
}
/// 取り込まなかった理由: 検証で有効行0件
pub const CSV_SKIP_NO_VALID: &str = "取り込める行がありませんでした";
/// 取り込まなかった理由: 全件置換で行エラーあり
pub const CSV_SKIP_ROW_ERRORS: &str = "エラーのある行が含まれます";
/// 取り込まなかった理由: 確認で取り込まない選択
pub const CSV_SKIP_DECLINED: &str = "確認で取り込みをやめました";
/// 取り込まなかった理由: 保存済み件数を取れなかった
pub const CSV_SKIP_COUNT_UNKNOWN: &str = "保存済みの件数を確認できませんでした";

/// 件数激減の確認文。「はい/いいえ」で答える
pub fn csv_sync_plunge_question(kind: &str, new_rows: usize, saved_rows: usize) -> String {
    format!(
        "{kind}: 件数が大きく減っています(保存済み{} → {})。取り込みますか?",
        count(saved_rows),
        count(new_rows)
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

pub fn previous_dividend_span(current_month: u32, amount: &str) -> String {
    let months = match current_month {
        1..=10 => format!("{}〜12月", current_month + 1),
        11 => "12月".to_string(),
        _ => return String::new(),
    };
    format!("・ 前年の{months}は {amount} 入金 ")
}

/// 条件を選ぶ小窓の見出し。対象名だけを引数にして画面ごとに使い回す
pub fn pick_title(target: &str) -> String {
    format!("{target}を選ぶ")
}

/// グループ見出しの件数(「2026年10月 · 3件」)
pub fn group_count(label: &str, total: usize) -> String {
    format!("{label} · {}", count(total))
}

/// 基準日の書き方(「価格基準日 10/21」)
pub fn price_as_of(date: &str) -> String {
    format!("{PRICE_AS_OF} {}", format_price_date(date))
}

/// 絞り込み中の件数(「絞り込み中(4件 / 77件)」)
pub fn filtering_count(filtered: usize, total: usize) -> String {
    format!("{FILTERING}({} / {})", count(filtered), count(total))
}

/// 一部の基準価額・株価が最新でないときの小さな表示(帯のラベルの横に出す)
pub const PARTIAL_PENDING_UPDATE: &str = "一部更新待ち";

/// CSV管理: 取得カードの見出し
pub const CSV_MANAGE_FETCH_TITLE: &str = "楽天証券から最新のデータを取得";
/// CSV管理: 取得ボタン(通常)
pub const CSV_MANAGE_FETCH_BUTTON: &str = "すべて取得";
/// CSV管理: 取得ボタン(実行中)
pub const CSV_MANAGE_FETCH_RUNNING: &str = "取得中…";
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
    format!("今日 {time} に更新")
}
/// CSV管理: 状態「N日前(M/D HH:MM)に更新」(薄い色)
pub fn csv_manage_status_updated_days_ago(days: i64, date: &str, time: &str) -> String {
    format!("{days}日前({date} {time})に更新")
}
pub const CSV_MANAGE_STATUS_UPDATE_REQUIRED: &str = "要更新";
/// CSV管理: 状態「取得できず・楽天証券のタブを再読み込みしてください」(赤)
pub const CSV_MANAGE_STATUS_FAILED: &str = "取得できず・楽天証券のタブを再読み込みしてください";
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
    total: usize,
) -> String {
    let range = range.map(|range| format!("({range})")).unwrap_or_default();
    format!(
        "{kind}の保存済み {}{}を削除します。元に戻せません",
        count(total),
        range
    )
}
/// CSV管理: 削除確認モーダルの実行ボタン
pub fn csv_manage_delete_confirm_button(total: usize) -> String {
    format!("{}を削除する", count(total))
}

/// ログイン履歴: 設定内の履歴セクションの見出し
pub const LOGIN_HISTORY_SECTION: &str = "ログイン履歴";
/// ログイン履歴: 登録した端末セクションの見出し
pub const REGISTERED_DEVICES_SECTION: &str = "登録した端末";
/// ログイン履歴: Cookie を消すと新しい端末になる旨の注意書き
pub const LOGIN_HISTORY_COOKIE_NOTE: &str =
    "Cookie を消すと同じ端末でも新しい端末として登録されます";
/// ログイン履歴: 結果「成功」
pub const LOGIN_RESULT_SUCCESS: &str = "成功";
/// ログイン履歴: 結果「新しい端末」
pub const LOGIN_RESULT_NEW_DEVICE: &str = "新しい端末";
/// ログイン履歴: 結果「場所の変化」
pub const LOGIN_RESULT_LOCATION_CHANGED: &str = "場所の変化";
/// ログイン履歴: 結果「失敗」
pub const LOGIN_RESULT_FAILURE: &str = "失敗";
/// ログイン通知: トグルの文言
pub const LOGIN_NOTIFY_TITLE: &str = "新しい端末からログインしたときに通知する";
/// ログイン通知: トグルの補足
pub const LOGIN_NOTIFY_NOTE: &str = "オフでも、ログインの記録は残ります";
/// ログイン通知: 使い捨てリンクの確認画面の見出し
pub const NOTIFY_LOGOUT_TITLE: &str = "心当たりのないログインがありましたか";
/// ログイン通知: 使い捨てリンクの確認画面の説明
pub const NOTIFY_LOGOUT_DESCRIPTION: &str =
    "次の操作でこのアカウントのすべての端末をログアウトします";
/// ログイン通知: 使い捨てリンクの実行ボタン
pub const NOTIFY_LOGOUT_CONFIRM: &str = "すべての端末をログアウトする";
/// ログイン通知: ログアウト完了の文言
pub const NOTIFY_LOGOUT_DONE: &str = "すべての端末からログアウトしました";
/// ログイン通知: リンクが無効・期限切れ・使用済みのときの文言
pub const NOTIFY_LOGOUT_INVALID: &str = "リンクが無効か期限切れです";
/// ログイン履歴: 列見出し「日時」
pub const LOGIN_HISTORY_COL_DATETIME: &str = "日時";
/// ログイン履歴: 列見出し「結果」
pub const LOGIN_HISTORY_COL_RESULT: &str = "結果";
/// ログイン履歴: 列見出し「場所」
pub const LOGIN_HISTORY_COL_PLACE: &str = "場所";
/// ログイン履歴: 列見出し「ブラウザ・OS」
pub const LOGIN_HISTORY_COL_AGENT: &str = "ブラウザ・OS";
/// 登録した端末: 列見出し「最終利用」
pub const DEVICE_COL_LAST_USED: &str = "最終利用";
/// 登録した端末: 自分が今使っている端末の印
pub const DEVICE_CURRENT: &str = "この端末";
/// 登録した端末: 1台の解除操作
pub const DEVICE_REVOKE: &str = "解除";
/// 登録した端末: 確認ダイアログの実行ボタン
pub fn device_revoke_confirm_button(place: &str) -> String {
    format!("{place}を解除する")
}
/// 登録した端末: すべての端末のログアウト操作
pub const DEVICE_LOGOUT_ALL: &str = "すべての端末をログアウト";
/// 登録した端末: 履歴も端末も無いときの空表示
pub const LOGIN_HISTORY_EMPTY: &str = "ログイン履歴はまだありません";
/// ログイン履歴: 追加の履歴を読み込む操作
pub const LOGIN_HISTORY_SHOW_MORE: &str = "もっと見る";
/// 登録した端末: 有効な端末が無いときの空表示
pub const DEVICE_EMPTY: &str = "登録した端末はありません";

pub const ADMIN_LOGINS: &str = "ログイン一覧(管理者)";
pub const ADMIN_LOGIN_EMAIL: &str = "メール";
pub const ADMIN_LOGIN_SEARCH: &str = "メール・場所";
pub const ADMIN_LOGIN_EMPTY: &str = "条件に一致するログイン記録はありません";
pub const ADMIN_FILTER_APPLY: &str = "この条件で絞る";
pub const SORT_BY: &str = "並べ替え";
/// 税引前の金額の頭に付ける短い表記(行の補助の金額用)
pub const BEFORE_TAX: &str = "税引前";
/// 取引明細の帯の主役(配当金タブ)
pub const RECEIVED_DIVIDEND_AFTER_TAX: &str = "受け取った配当金(税引後)";
/// 取引明細の帯の主役(国内株式・投資信託タブ)
pub const REALIZED_AFTER_TAX: &str = "実現損益(税引後)";
/// 月見出しの右に出す短い受取表記(配当金タブ)
pub const RECEIPT_SHORT: &str = "受取";
/// 月見出しの右に出す短い損益表記(国内株式・投資信託タブ。#1634 の指定による例外)
pub const PROFIT_SHORT: &str = "損益";
/// 取引明細の画面名
pub const RECEIPTS_TITLE: &str = "取引明細";
/// CSV管理の画面名
pub const CSV_MANAGE_TITLE: &str = "CSV管理";
/// すべて(期間なし・選択なしの状態)
pub const ALL: &str = "すべて";
/// 条件(検索ダイアログの節見出し)
pub const CONDITION: &str = "条件";
/// 集計(帯の領域名)
pub const SUMMARY: &str = "集計";
/// 空の表に出す文言
pub const NO_DATA: &str = "データがありません";
/// 失敗表示の頭に付ける文言
pub const ERROR_PREFIX: &str = "エラー:";
/// 日付(日単位の絞り込みの見出し)
pub const DATE: &str = "日付";
/// 月(月選択の見出し)
pub const MONTH: &str = "月";
/// 開始日(範囲絞り込みの見出し)
pub const START_DATE: &str = "開始日";
/// 終了日(範囲絞り込みの見出し)
pub const END_DATE: &str = "終了日";
/// 期間の年単位
pub const PERIOD_YEAR: &str = "年";
/// 期間の年月単位
pub const PERIOD_YEAR_MONTH: &str = "年月";
/// 期間の日単位
pub const PERIOD_DAY: &str = "日";
/// 期間の範囲指定
pub const PERIOD_RANGE: &str = "範囲";
/// 期間の種類(絞り込みの小窓の見出し)
pub const PERIOD_KIND_LABEL: &str = "期間の種類";
/// 取引明細の検索領域名
pub const RECEIPT_SEARCH_REGION: &str = "取引明細の検索";
/// 期間の絞り込みを確定するボタン
pub const RECEIPT_APPLY_PERIOD: &str = "この期間で絞る";
/// 年の選択肢が無いときの表示
pub const RECEIPT_NO_YEARS: &str = "該当する年がありません";
/// 年の選択肢の初期表示
pub const RECEIPT_YEAR_SELECT: &str = "年を選択";
/// 月の選択肢の初期表示
pub const RECEIPT_MONTH_SELECT: &str = "月を選択";
/// 円(金額の表示単位)
pub const YEN: &str = "円";
/// 千円(金額の表示単位)
pub const THOUSAND_YEN: &str = "千円";
/// 万円(金額の表示単位)
pub const TEN_THOUSAND_YEN: &str = "万円";
/// CSVを取り込む操作
pub const CSV_IMPORT: &str = "CSVを取り込む";
/// 範囲の区切り
pub const RANGE_SEPARATOR: &str = "〜";

/// 年の書き方(「2024年」)
pub fn receipt_year_label(year: &str) -> String {
    format!("{year}年")
}

/// 月の書き方(「3月」)
pub fn receipt_month_label(month: u32) -> String {
    format!("{month}月")
}

/// 期間の開始だけあるときの表示(「2024-03〜」)
pub fn receipt_range_from(start: &str) -> String {
    format!("{start}〜")
}

/// 期間の終了だけあるときの表示(「〜2024-03」)
pub fn receipt_range_until(end: &str) -> String {
    format!("〜{end}")
}

/// 期間の開始と終了があるときの表示(「2024-01〜2024-03」)
pub fn receipt_range_between(start: &str, end: &str) -> String {
    format!("{start}〜{end}")
}

/// 帯に出す期間の書き方(「2026/01 〜 2026/12」)
pub fn receipt_period_short(from: &str, to: &str) -> String {
    format!("{from} 〜 {to}")
}

/// コピー操作の読み上げ文言(「◯◯ をコピー」)
pub fn copy_aria_label(text: &str) -> String {
    format!("{text} をコピー")
}

/// 配当情報に保有があるときの印
pub const DIVIDEND_HELD_BADGE: &str = "保有銘柄";
/// 配当情報で保有がないときの案内
pub const DIVIDEND_BALANCE_HINT: &str = "資産管理にCSVを取り込むと表示されます";
/// 配当情報で自動取得されるときの案内
pub const DIVIDEND_AUTO_HINT: &str = "自動で取得されます";
/// 配当情報の集計区画の見出し
pub const DIVIDEND_SUMMARY_TITLE: &str = "集計情報";
/// 配当金明細が空のときの案内
pub const RECEIPT_DIVIDEND_EMPTY_HINT: &str = "配当金明細をCSVで追加してください";
/// 国内株式明細が空のときの案内
pub const RECEIPT_STOCK_EMPTY_HINT: &str = "国内株式明細をCSVで追加してください";
/// 投資信託明細が空のときの案内
pub const RECEIPT_FUND_EMPTY_HINT: &str = "投資信託明細をCSVで追加してください";
/// 一覧が上限で切れたときの警告(取引明細用)
pub fn receipt_truncated_warning(total: &str) -> String {
    format!("一覧は最大{total}件まで表示しています。検索条件を絞り込んでください。")
}

/// 年月の書き方(「2024年3月」)
pub fn receipt_year_month_label(year: &str, month: u32) -> String {
    format!("{year}年{month}月")
}

/// 年月日の書き方(「2024年3月1日」)
pub fn receipt_year_month_day_label(year: &str, month: u32, day: u32) -> String {
    format!("{year}年{month}月{day}日")
}

/// 設定: 画面名
pub const SETTINGS_TITLE: &str = "設定";
/// 設定: 目次の nav の読み上げ名
pub const SETTINGS_TOC_LABEL: &str = "設定セクション";
/// 設定: 表示セクションの見出し
pub const SETTINGS_DISPLAY_SECTION: &str = "表示";
/// 設定: データセクションの見出し
pub const SETTINGS_DATA_SECTION: &str = "データ";
/// 設定: セキュリティセクションの見出し
pub const SETTINGS_SECURITY_SECTION: &str = "セキュリティ";
/// 設定: 危険な操作セクションの見出し
pub const SETTINGS_DANGER_SECTION: &str = "危険な操作";
/// 設定: ログイン手段の表示
pub const SETTINGS_GOOGLE_LOGIN: &str = "Google でログイン中";
/// 設定: テーマの行名
pub const SETTINGS_THEME: &str = "テーマ";
/// 設定: テーマの選択肢(端末に合わせる)
pub const SETTINGS_THEME_SYSTEM: &str = "端末に合わせる";
/// 設定: テーマの選択肢(ライト)
pub const SETTINGS_THEME_LIGHT: &str = "ライト";
/// 設定: テーマの選択肢(ダーク)
pub const SETTINGS_THEME_DARK: &str = "ダーク";
/// 設定: 資産管理の並び順の行名
pub const SETTINGS_ASSET_SORT: &str = "資産管理の並び順";
/// 設定: 資産管理の並び順の選択肢の読み上げ名
pub const SETTINGS_ASSET_SORT_INITIAL: &str = "資産管理の初期の並び順";
/// 設定: 取引明細の初期タブの行名
pub const SETTINGS_RECEIPTS_TAB: &str = "取引明細の初期タブ";
/// 設定: 金額の表示単位の行名
pub const SETTINGS_AMOUNT_UNIT: &str = "金額の表示単位";
/// 設定: CSV エクスポートの行名
pub const SETTINGS_EXPORT_CSV: &str = "CSVでエクスポート";
/// 設定: CSV の取り込み・保存済みデータの削除の行名
pub const SETTINGS_CSV_IMPORT_DELETE: &str = "CSVの取り込み・保存済みデータの削除";
/// 設定: CSV管理へのリンク名
pub const SETTINGS_CSV_MANAGE_LINK: &str = "CSV管理へ";
/// 設定: 自動ログアウトの行名
pub const SETTINGS_IDLE_LOGOUT: &str = "操作がないときの自動ログアウト";
/// 設定: 自動ログアウトまでの時間の選択肢の読み上げ名
pub const SETTINGS_IDLE_LOGOUT_TIME: &str = "自動ログアウトまでの時間";
/// 設定: 自動ログアウトの選択肢(15分)
pub const SETTINGS_IDLE_15: &str = "15分";
/// 設定: 自動ログアウトの選択肢(30分)
pub const SETTINGS_IDLE_30: &str = "30分";
/// 設定: 自動ログアウトの選択肢(60分)
pub const SETTINGS_IDLE_60: &str = "60分";
/// 設定: 削除操作のボタン名
pub const SETTINGS_DELETE: &str = "削除…";
/// 設定: 取引明細(配当金)の削除の行名
pub const SETTINGS_DELETE_DIVIDEND: &str = "取引明細：配当金を削除";
/// 設定: 取引明細(配当金)の削除の説明
pub const SETTINGS_DELETE_DIVIDEND_NOTE: &str =
    "取り込んだ配当金データ。アカウントは残ります。取り消せません";
/// 設定: 取引明細(配当金)の削除リンクの読み上げ名
pub const SETTINGS_DELETE_DIVIDEND_ACTION: &str = "配当金データを削除";
/// 設定: 取引明細(国内株式)の削除の行名
pub const SETTINGS_DELETE_STOCK: &str = "取引明細：国内株式を削除";
/// 設定: 取引明細(国内株式)の削除の説明
pub const SETTINGS_DELETE_STOCK_NOTE: &str =
    "取り込んだ国内株式売買データ。アカウントは残ります。取り消せません";
/// 設定: 取引明細(国内株式)の削除リンクの読み上げ名
pub const SETTINGS_DELETE_STOCK_ACTION: &str = "国内株式データを削除";
/// 設定: 取引明細(投資信託)の削除の行名
pub const SETTINGS_DELETE_FUND: &str = "取引明細：投資信託を削除";
/// 設定: 取引明細(投資信託)の削除の説明
pub const SETTINGS_DELETE_FUND_NOTE: &str =
    "取り込んだ投資信託売買データ。アカウントは残ります。取り消せません";
/// 設定: 取引明細(投資信託)の削除リンクの読み上げ名
pub const SETTINGS_DELETE_FUND_ACTION: &str = "投資信託データを削除";
/// 設定: 資産管理データの削除の行名
pub const SETTINGS_DELETE_ASSETS: &str = "資産管理データを削除";
/// 設定: 資産管理データの削除の説明
pub const SETTINGS_DELETE_ASSETS_NOTE: &str =
    "保存済みの保有銘柄データ。アカウントは残ります。取り消せません";
/// 設定: アカウント削除の行名
pub const SETTINGS_DELETE_ACCOUNT: &str = "アカウントを削除";
/// 設定: アカウント削除の説明
pub const SETTINGS_DELETE_ACCOUNT_NOTE: &str = "データもすべて消えます。取り消せません";
/// CSV エクスポート: 種類名(資産残高)
pub const EXPORT_ASSET_BALANCE: &str = "資産残高";
/// CSV エクスポート: 種類名(売買履歴)
pub const EXPORT_TRANSACTIONS: &str = "売買履歴";
/// CSV エクスポート: 件数超過の失敗文
pub const EXPORT_TRUNCATED: &str = "件数が上限を超えたため、全件を取得できませんでした";

/// CSV エクスポートの保存完了文(「◯◯ を保存しました」)
pub fn settings_export_saved(filename: &str) -> String {
    format!("{filename} を保存しました")
}

/// CSV エクスポートの失敗文(「◯◯のエクスポートに失敗しました: 理由」)
pub fn settings_export_failed(kind: &str, error: &str) -> String {
    format!("{kind}のエクスポートに失敗しました: {error}")
}

/// ホームの残りが無いときの表示
pub const HOME_REMAINING_EMPTY: &str = "残り —";

/// ホームの残りの書き方(「残り ¥12,000」)
pub fn home_remaining(amount: &str) -> String {
    format!("残り {amount}")
}

/// ホームで受取が予想を超えたときの書き方(「予想を ¥17,500 上回り」)
pub fn home_remaining_over(amount: &str) -> String {
    format!("予想を {amount} 上回り")
}

/// 資産管理: 画面名
pub const ASSET_BALANCE_TITLE: &str = "資産管理";
/// 資産管理: 保有ファンド詳細の画面名
pub const ASSET_FUND_DETAIL_TITLE: &str = "保有ファンド詳細";
/// 資産管理: 保有銘柄詳細の画面名
pub const ASSET_HOLDING_DETAIL_TITLE: &str = "保有銘柄詳細";
/// 資産管理: 一覧へ戻る操作
pub const ASSET_BALANCE_BACK: &str = "資産管理に戻る";
/// 資産管理: 空状態の見出し
pub const ASSET_EMPTY_TITLE: &str = "資産管理データがありません";
/// 資産管理: 空状態の説明
pub const ASSET_EMPTY_DESCRIPTION: &str =
    "CSVファイルをインポートするか、データを登録してください。";
/// 資産管理: ファンドが無いときの空表示
pub const ASSET_FUND_NOT_FOUND: &str = "ファンドが見つかりません";
/// 資産管理: 銘柄が無いときの空表示
pub const ASSET_HOLDING_NOT_FOUND: &str = "銘柄が見つかりません";
/// 注意表示の頭に付ける文言
pub const WARNING_PREFIX: &str = "注意:";
/// 構成比: 対象の注記
pub const COMPOSITION_TARGET: &str = "対象: 保存済みの保有全体";
/// 構成比: 算出できないときの表示
pub const COMPOSITION_UNAVAILABLE: &str = "算出不可";
/// 資産管理: 検索領域名
pub const ASSET_SEARCH_REGION: &str = "資産の検索";
/// タグの区切り
pub const TAG_SEPARATOR: &str = "・";
/// 投資信託のCSV見出し(種別の列)
pub const FUND_KIND: &str = "投資信託種別";
/// 基準価額の出どころ(公式APIの共有値)
pub const FUND_PRICE_SOURCE_AUTO: &str = "自動更新";
/// 基準価額の出どころ(CSVの取り込み値)
pub const FUND_PRICE_SOURCE_CSV: &str = "CSV 取り込み";
/// 保有種別の切り替えの読み上げ名
pub const ASSET_KIND_TABS_LABEL: &str = "保有の種類";
/// 前日比列の見出しの説明
pub const DAY_CHANGE_COLUMN_NOTE: &str =
    "最新の基準価額の日の前日比がわかる行だけ出します。更新待ちの行は CSV を取り込むと出ます";
/// 配当の取得失敗の表示
pub const HOLDING_DIVIDEND_FAILED: &str = "取得失敗";

/// 株数の書き方(「100株」)
pub fn stock_shares_label(shares: &str) -> String {
    format!("{shares}株")
}

/// 口数の書き方(「10000口」)
pub fn fund_units_label(units: &str) -> String {
    format!("{units}口")
}

/// 基準価額の前日比の出どころと日付(「CSV 取り込み 1/5 の前日比」)
pub fn fund_day_change_title(source: &str, date: &str) -> String {
    format!("{source} {date} の前日比")
}

/// 更新待ちの理由(「更新待ち: 最新の基準価額の日は 1/6、この値はCSV 取り込み 1/5」)
pub fn fund_pending_detail(expected: &str, source: &str, as_of: &str) -> String {
    format!("更新待ち: 最新の基準価額の日は {expected}、この値は{source} {as_of}")
}

/// 一覧が上限で切れたときの警告(資産管理用)
pub fn asset_truncated_warning(total: &str) -> String {
    format!("一覧は最大{total}件まで表示しています。未表示の銘柄がある可能性があります。")
}

/// 分配金コースの表示(「・ 受取型」)
pub fn course_with_dot(course: &str) -> String {
    format!("・ {course}")
}

/// 構成比の基準見出し(「基準(取得総額): 」)
pub fn composition_basis_heading(basis: &str) -> String {
    format!("基準({basis}): ")
}

#[cfg(test)]
mod tests;
