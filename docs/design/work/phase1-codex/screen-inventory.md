# 全画面・状態棚卸し（コード起点）

調査対象は `frontend/src/app.rs` のルート分岐と `frontend/src/features/`、`frontend/src/ui/`。行番号は本調査時点のもの。撮影は `shots/INDEX.md` の実績を参照。

## ルート一覧

| URL / パターン | ルート画面・目的 | 到達方法・表示データ・操作 | 状態・レスポンシブ | 根拠 |
|---|---|---|---|---|
| `/` | ホーム。資産概要と配当状況 | 下部/上部ナビ。保有株・投信、集計、今年の配当、商品構成、損益上位、最近の配当。カード内詳細、資産/明細/CSVへ移動 | 未ログインはログイン案内。読込中はカード骨格、カード単位の失敗・空・値ありを独立表示。集計不可・長い銘柄名・巨大額・欠測を表示。PCはヘッダーと列カード、スマホは下部ナビと圧縮カード | `frontend/src/app.rs:84-107,405-408`; `frontend/src/features/home/view.rs:76-123,1026-1040` |
| `/search` (+ `?code=`) | 銘柄検索 | 下部ナビ、銘柄コード/名前から入力して検索。結果は銘柄基本情報、外部情報リンク、保有していれば保有概要 | 初期案内、入力中、読込中、該当なし、失敗、結果あり、保有データ取得失敗。コードはURLと同期。スマホは検索欄を下部操作帯、PCは本文側 | `frontend/src/app.rs:87,407`; `frontend/src/features/stock_search/view.rs:67-99,133-156,377-394,396-470` |
| `/receipts` (+ `?kind=`) | 取引明細（配当金/国内株式/投資信託） | 下部ナビ/ホーム。タブ切替、検索、期間/口座/商品条件、並べ替え、行詳細、CSV管理 | 認証/タブ読込、一覧、空、検索0件、取得失敗、ページ上限/部分取得、フィルター条件あり。スマホはカード表示・検索欄が下部操作帯・スワイプタブ、PCは列テーブル・ヘッダーナビ | `frontend/src/app.rs:88-90,408`; `frontend/src/features/receipts.rs:22-30`; `frontend/src/features/receipts/view.rs:25-43,45-89`; `frontend/src/features/receipts/view/tabs.rs:80-118` |
| `/receipts/conditions` | 旧条件URL。独立ページなしで取引明細を表示 | 旧ブックマークから到達。通常の明細画面へ | `/receipts` と同じ。URL query に条件状態を復元する専用画面ではない | `frontend/src/app.rs:89-90` |
| `/csv` | CSV管理 | ナビ/各画面リンク。資産CSV2種・明細CSV3種の取得、ファイル取込、削除 | 初期統計読込、取得中、同期中、成功/失敗/未取得、取込/削除確認、種類ごとの件数・状態。スマホは名称と状態を2段、PCは表 | `frontend/src/app.rs:91-96,409`; `frontend/src/features/csv_page.rs:118-145`; `frontend/src/ui/csv_page.rs:530,688-702` |
| `/receipts/csv`, `/receipts/csv/delete`, `/assetbalance/csv`, `/assetbalance/csv/delete` | 旧CSV URL。CSV統合画面へ | 過去リンク/ブックマーク | 旧削除 URL も確認なしで統合一覧を開く。削除操作そのものは実行しない | `frontend/src/app.rs:91-96` |
| `/assetbalance` | 資産管理（すべて/国内株式/投資信託） | ナビ、ホーム。タブ、名称検索、ソート、詳細、構成比、CSV管理 | 認証/データ読込/失敗、空、検索0件、件数上限、株価・基準価額の欠測/更新待ち、損益符号、長い名称・大きな数値。PCはテーブル、スマホは行カード/スワイプタブ | `frontend/src/app.rs:97,410`; `frontend/src/features/asset_balance/view/kind_tabs.rs:11-23`; `view/main_content.rs:65-110`; `view/asset_table.rs:23-59` |
| `/assetbalance/composition` | 構成比 | 資産管理の構成比リンク。構成基準（評価額/年間配当）切替、構成円図・凡例・順位一覧、戻る | 読込/エラー/空、基準額0で算出不可、投信配当は欠測、長い名前。PC/スマホとも同一情報、幅で密度調整 | `frontend/src/app.rs:98,411-413`; `frontend/src/features/asset_balance/view/composition_page.rs:27-54,72-84,189-290`; `frontend/src/features/asset_balance/portfolio.rs:149-188` |
| `/assetbalance/holdings/:id` (`detail_id` が許容するID) | 旧保有株詳細URL | 資産行から/旧リンク。IDで保存済株を探し、見つかれば `/search?code=...` に履歴置換 | 読込骨格、取得失敗/再試行、IDなし。該当なしは空状態。このページで詳細表示はしない。未ログインでは保護され `/login` へ | `frontend/src/app.rs:76-78,101,414-417`; `frontend/src/urls.rs:1-14`; `frontend/src/features/asset_balance/view/holding_detail_page.rs:12-15,32-68,91-120` |
| `/assetbalance/funds/:id` (`FUND_HOLDINGS_PREFIX`) | 投信詳細 | 資産管理の投信行。価格・基準日、数量/口数、取得額・評価額・損益・口座等 | 読込/失敗/再試行/見つからない/価格欠測・更新待ち。PC/スマホはタイル群を幅調整 | `frontend/src/app.rs:80-82,101-102,418-422`; `frontend/src/urls.rs:1-14`; `frontend/src/features/asset_balance/view/fund_detail_page.rs:19-20,73-115` |
| `/settings` (+ `#account|display|data|security|devices|login-history|danger`) | 設定 | アカウントメニュー。目次リンクで7セクションへスクロール | アカウント、表示（テーマ/資産ソート/明細初期タブ/金額単位）、データ（出力/初期化）、セキュリティ（通知等）、登録端末、ログイン履歴、危険操作。保存/失敗/確認/読込状態。PCは左目次、スマホは縦並び・目次非表示 | `frontend/src/app.rs:99,423`; `frontend/src/features/settings.rs:142-208,213-222,257-317,349-379,427-468,487-505,616-651,685-765`; `frontend/src/features/settings/login_history.rs:121-180,289-410` |
| `/admin/logins` | 管理者ログイン履歴 | 管理者メニュー。メール/場所検索、条件パネル、期間・ソート | 未ログインはログイン遷移、一般ユーザーはNotFound、管理者は読込/失敗/0件/データあり。長いメール/ブラウザ名、欠測場所。PC表・スマホカード | `frontend/src/app.rs:100,361-363,424`; `frontend/src/features/admin_logins.rs:202-215,275,349-361,400-430,458-477` |
| `/login` | ログイン案内 | 未ログイン時の保護ルートリダイレクト、アカウント操作 | セッション判定中/ログイン済み遷移/ログアウト案内。PC/スマホ中央寄せ | `frontend/src/app.rs:103,370-380,425`; `frontend/src/features/login.rs:8-24` |
| `/404`・未知パス | NotFound | URL直打ち、無効リンク | 404ページ。ホーム/主要機能リンク。下部ナビなし。未知パスは `/404` に置換 | `frontend/src/app.rs:104-106,354-358,426`; `frontend/src/features/not_found.rs:13-43` |

`/settings/<unknown>` と `/admin/<unknown>` はルート解決上 NotFound だが、認証前は保護判定で `/login` に置換される。既知外パスはログイン済みなら `/404`。URLにはクエリ・ハッシュが付いても pathname で判定する。`frontend/src/app.rs:109-123,354-380`。

## ページ内画面・操作状態

| 画面/状態 | 目的・データ・操作 | PC/スマホ差 | 根拠 |
|---|---|---|---|
| CSV種類別の同期/取込フォーム | 5種類のデータを再取得、CSVファイル選択、結果の状態表示。件数急減時は確認して続行/取消 | PCは表内メニュー、スマホは行メニューを操作 | `frontend/src/features/csv_sync.rs:127-148,230-267,425-464`; `frontend/src/features/csv_sync/import.rs:110-133` |
| 設定フォーム | 表示テーマ/ソート/初期タブ/金額単位、データ出力/初期化、通知、端末失効、アカウント削除 | PC左目次、スマホ縦積み | `frontend/src/features/settings.rs:142-208,257-317,349-379,427-468,487-505,616-651,685-765` |
| ルート遷移/権限状態 | 保護パスの未ログイン遷移、ログイン済み `/login` からホーム、未知パスの `/404` 置換、管理者以外に管理者画面を隠す | 同じ判定。ナビはログイン/404で非表示 | `frontend/src/app.rs:84-123,354-380,396-426` |
| 資産タブ「すべて」 | 株・投信統合一覧、集計、検索、ソート。行から保有詳細へ遷移 | PC統合テーブル、スマホ統合カード | `frontend/src/features/asset_balance/view/all_list.rs:145-163,399-482` |
| 資産タブ「国内株式」 | 株の一覧、配当/利回りソート、構成比入口 | PC表、スマホ行カード | `frontend/src/features/asset_balance/view/asset_table.rs:23-59`; `view/holdings_list.rs:84-175` |
| 資産タブ「投資信託」 | 投信名、口座、口数、基準価額、損益 | PC表、スマホカード | `frontend/src/features/asset_balance/view/fund_table.rs` |
| 構成比の画面 | 評価額/年間配当基準の切替、円図・凡例・順位表示 | 可変幅 | `frontend/src/features/asset_balance/view/composition_page.rs:27-54,72-84,189-290`; `frontend/src/features/asset_balance/portfolio.rs:149-188` |
| 保有株の詳細ダイアログ | 国内株式一覧から詳細画面へ遷移。ダイアログは旧保有データ表示や一時的な明細表示に使う | PC中央ダイアログ、スマホシート | `frontend/src/ui/detail_dialog.rs:149-210`; `frontend/src/features/asset_balance/view/holdings_list.rs:174-184` |
| ホーム配当詳細ダイアログ | 配当金/税額/税引後、口座・日付 | PC中央、スマホシート | `frontend/src/features/home/view.rs:926-1014` |
| 投信詳細画面 | 投信保有状況・価格更新状況 | タイルの折返し | `frontend/src/features/asset_balance/view/fund_detail_page.rs:19-95` |
| 明細タブ3種 | 配当/株式/投信ごとの項目、集計、検索結果。行詳細、年/月グループ。配当は日付/口座/税引前・税額・税引後、株式/投信は売買・数量・単価・実現損益等 | PC列テーブル、スマホ行カード。スマホ検索は操作帯 | `frontend/src/features/receipts/view.rs:51-89`; `frontend/src/features/receipts/view/main_content.rs`; `frontend/src/features/receipts/view/table.rs`; `frontend/src/features/receipts/view/cards.rs` |
| 条件追加パネル | 期間（全期間/年/年月/日/範囲）、口座、商品を選ぶ。期間は下書き確定、他は即時反映 | PCポップオーバー、スマホ下部シート | `frontend/src/ui/search_controls.rs:210-352`; `frontend/src/features/receipts/filter.rs` |
| CSV行メニュー | 種別ごとの再取得/ファイル取込/削除 | PCメニュー、スマホも行に紐付く | `frontend/src/ui/csv_page.rs:106-117,299-350,403-463,530,688-710` |
| 削除確認 | 対象種別・件数・取消不能の確認 | DialogShell/ConfirmDeleteModal | `frontend/src/ui/confirm_modal.rs:9-59`; `frontend/src/ui/csv_page.rs:530-540` |
| サイト共通アカウントメニュー | 設定、CSV出力、ログイン履歴/管理者、ログアウト、アカウント削除 | PCヘッダー中心、スマホアバター/メニュー | `frontend/src/ui/site.rs:138-179,233-300,467-522` |
| アカウント削除確認 | パスワード/確認入力後に削除を確定 | モーダル | `frontend/src/ui/site.rs:185-228` |
| 管理者の検索パネル/ダイアログ | 履歴をメール・場所・条件で絞る | PC条件ポップオーバー、スマホ検索ダイアログ | `frontend/src/features/admin_logins.rs:349-408` |
| 銘柄検索フォーム/結果 | コード入力、検索実行、基本情報、保有中のタイル、外部リンク | 未保有は下部操作帯、保有中は戻るリンクも表示 | `frontend/src/features/stock_search/view.rs:89-180,377-394,396-470` |
| 設定各フォーム | 端末テーマ、ソート、初期タブ、出力、通知、端末失効、ログアウト/削除 | PC左目次・横並びフォーム、スマホ目次なし・縦積み | `frontend/src/features/settings.rs:147-155,163-208,257-317`; `settings/login_history.rs:121-180,289-410` |

### 共通状態の分岐

- アプリ起動: セッション読込スピナー、保護画面は未認証なら `/login`、ログインページに認証済みなら `/`。管理者画面は権限なしをNotFoundに見せる。`frontend/src/app.rs:361-380,396-426`。
- データ一覧: 読込スケルトン、正常一覧、成功0件、取得失敗/再試行、前回データを保った再取得エラー。明細は `Loading/Ready/Failed`、資産/詳細はキャッシュ世代つき成功/失敗スロットを分岐する。`frontend/src/features/receipts.rs:131-136`; `frontend/src/features/asset_balance/view/composition_page.rs:128-169`; `frontend/src/ui/state.rs:82-118`。
- ホームはセクションごとに骨格/空/値ありを持ち、カード単位に情報が欠けたり失敗しても他カードを表示する。`frontend/src/features/home/view.rs:1026-1040,1081-1115`。
- 検索/条件: 未入力の案内、入力中、処理中、結果、該当なし、エラー、検索後の絞込件数、条件チップ有無/解除、期間下書き/確定、並べ替え、対象が検索結果0件。銘柄検索フォームと結果: `frontend/src/features/stock_search/view.rs:67-180,377-470`; 条件UI: `frontend/src/ui/search_controls.rs:210-405`。
- 資産の数値/価格: 正・負・ゼロ、欠測、価格更新待ち、基準価額/配当推定なし、集計基準額0、行上限到達。`frontend/src/features/asset_balance/view/composition_page.rs:128-169`; `frontend/src/features/asset_balance/view/main_content.rs:402-`。
- CSV処理: 待機、取得中、取得完了、取込中、急減確認中、取込完了、スキップ、失敗。`frontend/src/features/csv_sync.rs:127-148`。
- 設定/アカウント操作: 読込中、保存中、保存失敗、保存済み、通知設定、ログアウト/全データ初期化/削除の確認と失敗。各フォーム: `frontend/src/features/settings.rs`; アカウントメニュー/削除: `frontend/src/ui/site.rs:138-228`。
- 長い名称・メール/ブラウザ名、複数口座、大きな数値、欠測は特別な文字列分岐だけでなく折返し/省略/桁合わせの表示条件を持つ。`frontend/src/ui/format.rs`; `frontend/src/features/asset_balance/format.rs`; E2E架空fixture。
- テーマはシステム追従/ライト/ダーク。画面幅は `lg` を境にヘッダー/下部ナビ、テーブル/カード、ポップオーバー/シートを切替。

## 画面・モーダル・状態の集計

| 集計 | 件数 | 数え方 |
|---|---:|---|
| pathname ルート分岐 | 19 | `route_for_path` の明示固定パス12件 + 旧URL別名5件 + 保有ID群2件 + `/404` と未知パスをまとめた NotFound 分岐1件。クエリ/ハッシュは別画面に数えない |
| 画面・タブ・設定セクション | 30 | ルート画面12種 + 明細3タブ + 資産3タブ + 構成比2基準 + 設定7セクション + 株詳細旧URL遷移1 + 管理者権限なし1 |
| ダイアログ/パネル種別 | 9 | 保有株詳細、統合一覧の行詳細、ホーム配当詳細、検索、条件追加、CSV行メニュー、削除確認、アカウント削除、管理者検索 |
| 状態分類 | 8群 | 認証/権限、読込/通信、一覧データ、検索/条件、資産価格/数値、CSV操作、設定/フォーム、幅/テーマ。具体例は下記「共通状態」と各ルート行に記載 |

状態はコードに明示的な分岐がある表示/操作の種類を合算した。相互に組み合わせる全直積ではない。画面数は URL 数とは分け、タブ/セクションの別表示と権限状態も含める。

## 実装ファイル索引

- ルート判定/保護/アプリシェル: `frontend/src/app.rs:84-119,354-433`
- URL prefix: `frontend/src/urls.rs:1-14`（株 `/assetbalance/holdings/`、投信 `/assetbalance/funds/`）
- 資産トップ組立: `frontend/src/features/asset_balance/view.rs:260-`、タブ内容: `frontend/src/features/asset_balance/view/main_content.rs`、構成比: `frontend/src/features/asset_balance/view/composition_page.rs`、株/投信一覧: `frontend/src/features/asset_balance/view/asset_table.rs`, `frontend/src/features/asset_balance/view/fund_table.rs`, `frontend/src/features/asset_balance/view/all_list.rs`
- 明細: `frontend/src/features/receipts/view.rs:25-124`, `frontend/src/features/receipts/view/tabs.rs`, `frontend/src/features/receipts/view/main_content.rs`, `frontend/src/features/receipts/view/table.rs`, `frontend/src/features/receipts/view/cards.rs`, `frontend/src/features/receipts/filter.rs`
- 検索: `frontend/src/features/stock_search.rs`, `frontend/src/features/stock_search/view.rs`
- CSV: `frontend/src/features/csv_page.rs:118-145`, `frontend/src/ui/csv_page.rs`
- 設定: `frontend/src/features/settings.rs:142-208`; ログイン履歴/端末 `frontend/src/features/settings/login_history.rs`
- 管理者: `frontend/src/features/admin_logins.rs:202-`; ホーム `frontend/src/features/home/view.rs:76-123`
- 共有部品: `frontend/src/ui/`（個別の役割は `components.md`）
