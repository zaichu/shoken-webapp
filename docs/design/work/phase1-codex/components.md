# 共通UI部品・デザイントークン

## `frontend/src/ui/` の共通コンポーネント

| 名前 | 役割 | 利用画面 | props の要点・根拠 |
|---|---|---|---|
| `PageFrame` / `PcHeader` | ページの見出し、戻りリンク、操作リンク、本文、操作帯 | 全ページ | title/count/back/links/action_bar/full_bleed。`frontend/src/ui/page_frame.rs:5-32,38-119` |
| `Button`, `TabButton`, `LinkButton`, `Switch` | 主/副/危険操作、ページタブ、遷移、設定切替 | 全画面の操作 | variant/size/disabled/class/href、タブは role・キーボード操作を担う。`frontend/src/ui/button.rs:134-255` |
| `Card` | グループ化する面 | ホーム、設定、CSV、明細 | Panel/Collapsible/Item/Group/Sunken、class/testid。`frontend/src/ui/card.rs:3-33,49-` |
| `EmptyState`, `LoginPrompt` | 空・案内・認証誘導 | 検索、一覧、404、ログアウト画面 | title/description/icon/as_h1/children、案内用の任意class。`frontend/src/ui/empty_state.rs:35-60`; `frontend/src/ui/login_prompt.rs:11-` |
| `Badge`, `AccountBadge`, `CodeBadge` | 口座区分、銘柄コード、状態 | 資産・明細・検索 | 意味別variant。`frontend/src/ui/badge.rs:50-113` |
| `ListRow`, `ListRowButton`, `ListRowLink`, `ListRowStatic`, `ListGroupHeader` | スマホ一覧行とグループ見出し | ホーム、資産、明細 | 主値/補助情報、button/link/static を選ぶ。`frontend/src/ui/list_row.rs:28-120` |
| `SummaryBand`, `SummaryItem` | 集計帯と集計値 | ホーム、資産、明細 | Primary/Secondary、Neutral/Gain/Loss、ラベル/注記。`frontend/src/ui/summary.rs:61-78,127-` |
| `Amount` | 金額・率・数量の符号、色、桁揃え | 数値を出す各画面 | kind/text/negative/positive/block。`frontend/src/ui/amount.rs:13-49` |
| `Alert`, `Spinner`, `Loading`, `ListLoadError`, `ListSkeleton` | 通信/画面読込、失敗、再試行 | 全データ画面 | variant、size、Cards/Table、message/retry。`frontend/src/ui/state.rs:7-22,37-52,82-118` |
| `DialogShell` | 共通ダイアログ枠 | 検索・条件・確認 | title/on_close/testid/children。Escape、外側クリック、フォーカス管理。`frontend/src/ui/dialog.rs:114-174` |
| `DetailDialog` | PCの詳細ダイアログ、スマホの詳細シート | 資産・配当明細 | name/code/account と左右の詳細セル。`frontend/src/ui/detail_dialog.rs:149-210` |
| `ConfirmDeleteModal` | 取消不能操作の確認 | CSV削除、アカウント削除 | title/description/confirm label/loading/count。`frontend/src/ui/confirm_modal.rs:9-59` |
| `SearchBox`, `SearchTextInput`, `SearchTag`, `ConditionPanel`, `AddConditionRow` | 検索入力、条件チップ、絞込UI | 資産、明細、管理者検索 | value/placeholder/input/clear、条件ごとの開閉・選択肢。`frontend/src/ui/search_controls.rs:37-100,210-` |
| `SearchDialog` | スマホ用検索/並べ替えシート | 資産、明細、管理者 | title/on_close/testid/children。`frontend/src/ui/search_dialog.rs:7-49` |
| `SwipeArea`, `SwipeSlide` | 下線タブの横スワイプ | 資産・明細 | active index、slides、touch gesture。`frontend/src/ui/underline_tabs.rs:586-` |
| `Select`, `Segmented` | 設定・構成基準の選択 | 設定、構成比、検索条件 | value/aria_label/options/on_change。`frontend/src/ui/select.rs:6-`; `frontend/src/ui/segmented.rs:6-42` |
| `HoldingTiles`, `HoldingTile`, `HoldingSummary*`, `HoldingDetails`, `HoldingDetailItem` | 保有詳細の数値群 | 投信詳細、銘柄検索 | label/value/status/children。`frontend/src/ui/holding_tiles.rs:6-60` |
| `PageSkeleton` 系 | タブ・集計・注記・検索・一覧の順序を統一 | 資産、明細 | セクションを合成する子要素、FilteredCount。`frontend/src/ui/page_skeleton.rs:7-73` |
| `MoreLink`, `SecurityCodeLink`, `CopyableInstrumentName` | 関連画面リンク、銘柄コード、名前コピー | 資産、検索、明細 | 遷移先・コード・表示名。`frontend/src/ui/more_link.rs:4-`; `frontend/src/ui/security_link.rs:43-105,170-` |
| `BottomNav`, `AccountMenu`, `AccountDeleteDialog`, `SiteFooter` | 共通ナビ、アカウント操作、フッター | 全ページ（一部除く） | path/user/admin、メニュー開閉、削除処理。`frontend/src/ui/site.rs:49-129,138-228,233-300,533-540` |
| `CsvManageFetchCard`, `CsvKindTable` | 一括取得カード、5種類のCSV操作一覧 | CSV管理 | 種類別状態・処理中・行操作。`frontend/src/ui/csv_page.rs:31-69,689-710` |
| `StatRow`, `DisclosureToggle`, `Avatar` | 設定・詳細の小部品 | 設定、サイトメニュー | ラベル/値、開閉状態、ユーザー情報。`frontend/src/ui/stat_row.rs:6-`; `frontend/src/ui/disclosure.rs:9-`; `frontend/src/ui/avatar.rs:32-` |

## CSS トークン・スタイルの配置

- スタイルの正本は `frontend/style/input.css`。Tailwind v4 の `@theme` が色・影・角丸・間隔・文字サイズのユーティリティを定義し、`:root[data-theme="dark"]` がダーク値を設定する。`@layer components` はページ骨格と共通部品のまとまりを定義する。`frontend/style/input.css:1-152,221-325`。
- テーマ設定と適用は `frontend/src/settings.rs:170-190,336-425`。UIの組み立て方は `docs/design/mobile-ui-rules.md`、画面構成・表示要件・用語/書式は `docs/ui-design.md`。
- 色は意味別トークンを使う。`page/surface/surface-sunken/surface-raised` は面、`brand/gold` はブランド、`positive/negative` は損益、`accent/info` は操作・案内、`badge-*` は口座、`chart-*` は図表。原色パレットや白黒直書きは禁止。`docs/design/mobile-ui-rules.md:60-84`。
- 文字は `text-eyebrow`, `text-cell`, `text-summary-*` と共有ラベルを使う。金額は `tabular-nums`、円記号・桁区切り・符号は `frontend/src/ui/format.rs:1-24`、欠測は `—`。
- 画面間隔は `--space-tabs`, `--space-summary`, `--space-note`, `--space-search`, `--space-condition`, `--space-table` の6トークン。値は順に `.75rem, .75rem, .25rem, .75rem, .5rem, .75rem`。`frontend/style/input.css` と `docs/design/mobile-ui-rules.md` のトークン表。
- スマホ/PC切替は主に `lg` (64rem)。ヘッダー/下部ナビ、テーブル/カード、検索操作帯、ポップオーバー/シートを切り替える。タップ領域は44px以上、`prefers-reduced-motion` で動きを抑える。`docs/design/mobile-ui-rules.md`。

## CSS 値の許可リスト

`frontend/scripts/check-css-tokens.sh:20-38` の許可値は次のとおり。任意値は理由コメントの付いた限定例だけで、他の任意値や生パレット・旧トークン・色の直書きは検査対象 (`:48-125`、UI部品と色の検査を後続実行)。

| 対象 | 許可パターン | 許可理由 |
|---|---|---|
| Rust class | `transition-[...]` | 遷移プロパティ列挙 |
| Rust class | `min-w-[8ch]` | 表の金額列幅 |
| Rust class | `min-h-[50vh]` | 404/モーダルの画面基準配置 |
| Rust class | `shadow-[0_1px_2px_rgba(15,23,42,.08)]` | タブ選択時の微弱な影 |
| Rust class | `text-[11px]`, `text-[13px]`, `p-[3px]` | タブ件数、検索チップ、タブ内余白 |
| CSS | 上記に加えて `grid-cols-[...]` | レール付き2列などのレイアウト指定 |
| CSS | `text-[22px]`, `text-[26px]`, `text-[15px]` | 資産管理の集計文字サイズ |
| CSSの白黒 | `print:(\[[^]]+\]:)?border-black` のみ | 印刷時の罫線 |
