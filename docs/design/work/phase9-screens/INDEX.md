# Phase 9 画面見本: 完成形と状態の一覧

完成形は普段のデータが入った画面だけを載せる。状態(空・読み込み中・取得失敗など)は
画面ごとに「その状態のときの画面全体」を別に作る。画面の枠(ヘッダー・ナビ・上部の集計など)は
そのままで、データの部分だけがその状態になっている。状態の選び方は
`../phase1-codex/screen-inventory.md` の状態の章と各画面の README の「状態の例」を根拠にしている。

撮影: 完成形はスマホ 390・PC 1280 × ライト・ダーク。状態はスマホ 390・PC 1280 のライトだけ。
ファイル名は `<画面>-<mobile|pc>-<light|dark>.png`(完成形)、
`<画面>-state-<状態名英字>-<mobile|pc>.png`(状態)。
`?theme=light|dark` と `prefers-color-scheme` でライト/ダーク切替。
生成は各 `tools/gen.py`(phase8 は `tools/mkstates.py`)、撮影は各 `tools/shoot.py`(Playwright)。

## ホーム(`/`、完成形)

#1973 で本番に入った画面を正とする。静的見本・スクリーンショットは置かない。
資産管理・取引明細も本番画面を正とする。静的見本・スクリーンショットは置かない。

## 資産管理(`/assetbalance`、完成形)

旧phase9の完成形・状態HTML/PNGは削除済み。本番画面に変更を入れて撮影した画像を見本にする。

既存(`../phase1-codex/shots/asset-all-1440x900-pc-light.png`・`asset-all-mobile-light.png`)との対応:

| 既存から残したもの | 新しく取り入れたもの |
|---|---|
| 表・一覧はカードで囲まない(見出し行の下線と行の区切りだけ) | 上部の集計は帯なし・紺の文字(評価額・評価損益・前日比・取得総額。スマホは前日比・取得総額を開閉で見せる) |
| PCの表は既存の9列・順序・列名クリックの並べ替え(コード・銘柄/ファンド・種別・口座・分配金・1株配当・前日比・評価額・評価損益) | 文字の大きさの段・余白(見出し20px・行15px・補足12px) |
| スマホの行の型(1段目 名前と評価額、2段目 口座・数量、右に損益と率、名前は2行まで折り返し) | タブの下線・検索欄と並べ替えボタンの形と余白 |
| 構成比を見る・CSV管理への入口(見出しの右に文字リンクで) | 口座の札は色付きをやめ灰色の文字に |

| 状態 | 画像(mobile / pc) | いつ出るか |
|---|---|---|
| empty(データなし) | `assetbalance-state-empty-mobile.png` / `assetbalance-state-empty-pc.png` | CSV未取り込みで保有がないとき |
| loading(読み込み中) | `assetbalance-state-loading-mobile.png` / `assetbalance-state-loading-pc.png` | データ取得中 |
| failed(読み込み失敗) | `assetbalance-state-failed-mobile.png` / `assetbalance-state-failed-pc.png` | 取得に失敗したとき |

## 構成比(`/assetbalance/composition`、完成形 4 枚 + 状態 4 種 ×2 = 8 枚)

完成形: `opencode/shots/composition-mobile-light.png`、`composition-mobile-dark.png`、
`composition-pc-light.png`、`composition-pc-dark.png`

| 状態 | 画像(mobile / pc) | いつ出るか |
|---|---|---|
| empty(データなし) | `composition-state-empty-mobile.png` / `composition-state-empty-pc.png` | 保有残高がなく割合を出せないとき |
| loading(読み込み中) | `composition-state-loading-mobile.png` / `composition-state-loading-pc.png` | データ取得中 |
| failed(読み込み失敗) | `composition-state-failed-mobile.png` / `composition-state-failed-pc.png` | 取得に失敗したとき |
| uncalculable(算出不可) | `composition-state-uncalculable-mobile.png` / `composition-state-uncalculable-pc.png` | 合計が 0 で割合を出せないとき |

## 保有ファンド詳細(`/assetbalance/funds/:id`、完成形 4 枚 + 状態 4 種 ×2 = 8 枚)

完成形: `opencode/shots/fund-detail-mobile-light.png`、`fund-detail-mobile-dark.png`、
`fund-detail-pc-light.png`、`fund-detail-pc-dark.png`

| 状態 | 画像(mobile / pc) | いつ出るか |
|---|---|---|
| empty(見つからない) | `fund-detail-state-empty-mobile.png` / `fund-detail-state-empty-pc.png` | IDに一致するファンドがないとき |
| loading(読み込み中) | `fund-detail-state-loading-mobile.png` / `fund-detail-state-loading-pc.png` | データ取得中 |
| failed(読み込み失敗) | `fund-detail-state-failed-mobile.png` / `fund-detail-state-failed-pc.png` | 取得に失敗したとき |
| waiting(更新待ち) | `fund-detail-state-waiting-mobile.png` / `fund-detail-state-waiting-pc.png` | 基準価額が最新に更新されていないとき |

## 取引明細(`/receipts`、完成形)

旧phase9の完成形・状態HTML/PNGは削除済み。本番画面に変更を入れて撮影した画像を見本にする。

## CSV管理(`/csv`、完成形 4 枚 + 状態 5 種 ×2 = 10 枚)

完成形: `opencode/shots/csv-mobile-light.png`、`csv-mobile-dark.png`、
`csv-pc-light.png`、`csv-pc-dark.png`

| 状態 | 画像(mobile / pc) | いつ出るか |
|---|---|---|
| empty(データなし) | `csv-state-empty-mobile.png` / `csv-state-empty-pc.png` | まだ何も取得していないとき |
| loading(取得中) | `csv-state-loading-mobile.png` / `csv-state-loading-pc.png` | 取得中・取り込み中・確認待ちの行があるとき |
| failed(取得失敗) | `csv-state-failed-mobile.png` / `csv-state-failed-pc.png` | 拡張機能との通信が切れたとき |
| menu(行メニュー) | `csv-state-menu-mobile.png` / `csv-state-menu-pc.png` | 行のメニュー(再取得・取り込み・削除)を開いたとき |
| delete(削除確認) | `csv-state-delete-mobile.png` / `csv-state-delete-pc.png` | 削除の確認を出すとき |

## 銘柄検索(`/search`、完成形 4 枚 + 状態 5 種 ×2 = 10 枚)

完成形: `rest/shots/search-mobile-light.png`、`search-mobile-dark.png`、
`search-pc-light.png`、`search-pc-dark.png`

| 状態 | 画像(mobile / pc) | いつ出るか |
|---|---|---|
| initial(初期) | `search-state-initial-mobile.png` / `search-state-initial-pc.png` | まだ検索していないとき |
| nomatch(該当なし) | `search-state-nomatch-mobile.png` / `search-state-nomatch-pc.png` | 条件に合う銘柄がないとき |
| loading(読み込み中) | `search-state-loading-mobile.png` / `search-state-loading-pc.png` | 検索の取得中 |
| failed(取得失敗) | `search-state-failed-mobile.png` / `search-state-failed-pc.png` | 取得に失敗したとき |
| invalid(不正なコード) | `search-state-invalid-mobile.png` / `search-state-invalid-pc.png` | 不正な銘柄コードの URL で開いたとき |

## 設定(`/settings`、完成形 4 枚 + 状態 5 種 ×2 = 10 枚)

完成形: `rest/shots/settings-mobile-light.png`、`settings-mobile-dark.png`、
`settings-pc-light.png`、`settings-pc-dark.png`

| 状態 | 画像(mobile / pc) | いつ出るか |
|---|---|---|
| loading(読み込み中) | `settings-state-loading-mobile.png` / `settings-state-loading-pc.png` | 設定の取得中 |
| saving(保存中) | `settings-state-saving-mobile.png` / `settings-state-saving-pc.png` | 変更を保存しているとき |
| saved(保存済み) | `settings-state-saved-mobile.png` / `settings-state-saved-pc.png` | 保存が終わった直後 |
| savefailed(保存失敗) | `settings-state-savefailed-mobile.png` / `settings-state-savefailed-pc.png` | 保存に失敗したとき |
| empty(空) | `settings-state-empty-mobile.png` / `settings-state-empty-pc.png` | 登録端末・ログイン履歴がまだないとき |

## ログイン(`/login`、完成形 4 枚 + 状態 3 種 ×2 = 6 枚)

完成形: `rest/shots/login-mobile-light.png`、`login-mobile-dark.png`、
`login-pc-light.png`、`login-pc-dark.png`

| 状態 | 画像(mobile / pc) | いつ出るか |
|---|---|---|
| loading(読み込み中) | `login-state-loading-mobile.png` / `login-state-loading-pc.png` | セッション判定中 |
| failed(取得失敗) | `login-state-failed-mobile.png` / `login-state-failed-pc.png` | 判定に失敗したとき |
| loggedin(ログイン済み) | `login-state-loggedin-mobile.png` / `login-state-loggedin-pc.png` | ログイン済みで開いたとき |

## 未ログインのホーム(`/`未ログイン時、完成形 4 枚 + 状態 2 種 ×2 = 4 枚)

完成形: `rest/shots/loginprompt-mobile-light.png`、`loginprompt-mobile-dark.png`、
`loginprompt-pc-light.png`、`loginprompt-pc-dark.png`

| 状態 | 画像(mobile / pc) | いつ出るか |
|---|---|---|
| loading(読み込み中) | `loginprompt-state-loading-mobile.png` / `loginprompt-state-loading-pc.png` | セッション判定中 |
| failed(取得失敗) | `loginprompt-state-failed-mobile.png` / `loginprompt-state-failed-pc.png` | 判定に失敗したとき |

## 404(`/404`・未知パス、完成形 4 枚、状態なし)

404 自体が状態のため、別の状態画面は作らない。認証・権限の振り分け
(未ログインはログインへ、管理者以外は 404)は完成形に含む。

完成形: `rest/shots/notfound-mobile-light.png`、`notfound-mobile-dark.png`、
`rest/shots/notfound-pc-light.png`、`notfound-pc-dark.png`

## ログイン一覧(管理者)(`/admin/logins`、完成形 4 枚 + 状態 3 種 ×2 = 6 枚)

完成形: `rest/shots/admin-logins-mobile-light.png`、`admin-logins-mobile-dark.png`、
`rest/shots/admin-logins-pc-light.png`、`admin-logins-pc-dark.png`

| 状態 | 画像(mobile / pc) | いつ出るか |
|---|---|---|
| empty(データなし) | `admin-logins-state-empty-mobile.png` / `admin-logins-state-empty-pc.png` | 条件に一致する記録がないとき |
| loading(読み込み中) | `admin-logins-state-loading-mobile.png` / `admin-logins-state-loading-pc.png` | 記録の取得中 |
| failed(取得失敗) | `admin-logins-state-failed-mobile.png` / `admin-logins-state-failed-pc.png` | 取得に失敗したとき |

## ダイアログ(完成形なし + 状態 10 種 ×2 = 20 枚)

ダイアログは開いている状態の画面全体で 1 つずつ作る。後ろに元の画面が見える。
スマホは下部シート、PC は中央ダイアログで出す。

| 状態 | 画像(mobile / pc) | 後ろの画面 / いつ出るか |
|---|---|---|
| holding(保有株詳細) | `dialogs-state-holding-mobile.png` / `dialogs-state-holding-pc.png` | 資産管理 / 国内株式の行から詳細を開いたとき |
| receipt-detail(取引明細の行詳細) | `dialogs-state-receipt-detail-mobile.png` / `dialogs-state-receipt-detail-pc.png` | 取引明細 / 配当の行から内訳を開いたとき |
| dividend-detail(ホーム配当詳細) | `dialogs-state-dividend-detail-mobile.png` / `dialogs-state-dividend-detail-pc.png` | ホーム / 配当の行から詳細を開いたとき |
| filter(条件追加パネル) | `dialogs-state-filter-mobile.png` / `dialogs-state-filter-pc.png` | 取引明細 / 期間・口座・商品で絞るとき |
| csv-menu(CSV行メニュー) | `dialogs-state-csv-menu-mobile.png` / `dialogs-state-csv-menu-pc.png` | CSV管理 / 行の操作を開いたとき |
| delete-confirm(削除確認) | `dialogs-state-delete-confirm-mobile.png` / `dialogs-state-delete-confirm-pc.png` | CSV管理 / 保存済みデータの削除を確認するとき |
| account-menu(アカウントメニュー) | `dialogs-state-account-menu-mobile.png` / `dialogs-state-account-menu-pc.png` | ホーム / 右上のアカウントを開いたとき |
| account-delete(アカウント削除確認) | `dialogs-state-account-delete-mobile.png` / `dialogs-state-account-delete-pc.png` | 設定 / アカウントの削除を確認するとき |
| admin-search(管理者検索パネル) | `dialogs-state-admin-search-mobile.png` / `dialogs-state-admin-search-pc.png` | ログイン一覧 / メール・場所・条件で絞るとき |
| logout-all(全端末ログアウト確認) | `dialogs-state-logout-all-mobile.png` / `dialogs-state-logout-all-pc.png` | 設定 / 心当たりのないログインの通知から操作するとき |
