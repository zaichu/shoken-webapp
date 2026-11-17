# 見た目の定義の棚卸し(Issue #1987 1段目)

対象: `frontend/style/input.css`(2,261行)と `frontend/src/**/*.rs` の class 指定。
確かめ方: CSS のクラス定義(`.[a-z]...`)256件を抽出し、Rust 側の `class="..."`・`class:name=`・文字列リテラル(変数・`format!` 経由)と突き合わせた。`data-testid` のみの一致は「未使用」と数えた。
推測と確かめたことは分けて書く。コードは変えていない。

## 1. 同じ役割の見た目が複数の場所にある箇所

### 1a. カード・面(背景・枠・角丸の組み合わせ)

| 役割 | 定義場所 | 備考 |
|---|---|---|
| 汎用パネル | input.css:450 `.panel-card` | `CardVariant::Panel`(card.rs:25)経由 |
| 表の面 | input.css:454 `.table-card` | table.rs:191 で文字列指定(検査の許可リスト入り) |
| 折りたたみ | input.css:461 `.collapsible-card` | `CardVariant::Collapsible`(card.rs:26)経由 |
| 空状態 | input.css:466 `.empty-state` | empty_state.rs:43 の既定値。検査(PRIMITIVE_CLASS)で features 直書き禁止済み |
| 表の枠(旧道) | input.css:1470 `.table-frame` | admin_logins.rs:468、login_history.rs:192、table.rs:207。`border: 0; box-shadow: none` を各自で消す流儀 |
| 資産の面 | input.css:1997 `.asset-panel` | fund_detail_page.rs:143。`border: 0; box-shadow: none` |
| 資産の面(余白付き) | input.css:2064 `.asset-panel-padded` | main_content.rs:181。`.asset-panel` と別定義 |
| 保有タイル | input.css:2007 `.holding-tile` | holding_tiles.rs:21 |
| 保有サマリー | input.css:2032 `.holding-summary-card` | holding_tiles.rs:38 |
| 一覧の面(明細) | input.css:1153 `.receipt-list-surface` | table.rs:189-191 で `compact_mobile` 時に選択 |
| 一覧の面(資産) | input.css:1715 `.asset-list-surface` | 同じ「スマホは面なし・PCは角丸+枠」を別名で定義 |
| ホーム行の面 | input.css:1549 `.home-card-grid > *` | スマホだけ `rounded-xl border` を子に付ける特殊形 |

確かめたこと: `.table-card`(input.css:454)と`.table-frame`(input.css:1470)はどちらも「表の面」で `bg-surface`+枠消しを持ち、table.rs:191/207/213-215 で入れ子(`table-card` > `p-0` > `table-frame`)に使われる。`bg-surface` と `border: 0` が二重。
推測: `.table-frame` は旧道で、新規は `.table-card` に寄せている途中。要確認。
確かめたこと(追加): `CardVariant::Item/Group/Sunken/GroupLabel`(card.rs:27-30)は角丸・枠・面の組み合わせを Rust の文字列で直書きしており、CSS の `.panel-card` 系と同じ役割を Rust 側でも持つ。`check-card-like` 検査は features/ だけを見るため ui/ 内のこの重複は検出対象外。

### 1b. 行(スマホの2行リスト)

| 役割 | 定義場所 |
|---|---|
| 共通2行の行本体 | input.css:1733 `.list-row` 以下(`-main`1737/`-name`1741/`-primary`1745/`-sub`1749/`-meta`1753/`-secondary`1757) |
| 行の区切り・押せる面 | input.css:1761 `.list-row-item`、1765 `.list-row-action`/`.list-row-action-static` |
| グループ見出し | input.css:1783 `.list-group-header`(list_row.rs:122 の `ListGroupHeader` が正本)。表側は input.css:1251 `.receipts-table-group`+1257 `-group-label` が同じトークン(金線・`bg-row-group`)を別定義 |
| 明細の行リスト | input.css:1276 `.receipt-compact-list` |
| 資産の行リスト | input.css:1721 `.asset-compact-list`。中身は `m-0 p-0 list-none` のみで `.receipt-compact-list` と同値(別名) |

### 1c. 金額の色

| 役割 | 定義場所 |
|---|---|
| 金額の色分け本体 | amount.rs:49 `amount_classes`(kind/negative/positive で `text-positive` 等を選択)+ input.css:216 `[data-negative="true"]` |
| 帯の数字の色 | input.css:1493 `.summary-value`(紺=`text-brand-text`)。損益色は中の `Amount` が付ける二段構え |
| 札(バッジ) | input.css:506 `.badge`+510-522 4種、1293 `.badge-account` 系(口座区分)、526 `.badge-code`。`Badge` 部品経由だが `badge-code-flush`(530)など呼び分けが CSS 側に分散 |
| 表のコード列 | input.css:539 `.table-code`(薄色・桁揃え)。`.badge-code` と役割が近い(コード表示)が別定義 |

### 1d. タブ

| 役割 | 定義場所 |
|---|---|
| 種別タブ本体 | input.css:601 `.page-tab-list`、612 `.page-tab`、618 選択状態 |
| タブの入口(Button 経由) | button.rs:103 `PageTab => "page-tab"`。同じ見た目を Rust 側の対応表でも持つ |
| 少数択一 | input.css:471 `.segmented`、477 `.segment-option`。button.rs:100 `Segmented`/`ConditionSegment` も同じ文字列を返す二重管理 |
| 条件パネルの5択 | input.css:828 `.condition-segment`(中は `.segment-option` を流用) |
| 年の選択肢 | input.css:851 `.year-option`。`.condition-option`(842)と役割が近い(丸い選択肢)が別定義 |
| 構成比の区分タブ | input.css:1798 `.comp-tab-row`(スマホで全幅化)。タブ本体は `.page-tab-list` を流用(composition_page.rs:204-207) |

### 1e. 検索・条件

| 役割 | 定義場所 |
|---|---|
| 検索面 | input.css:747 `.search-field`(虫眼鏡+入力+実行+タグを1面に)。スマホ入口は input.css:992 `.search-sort-entry`+1013 `.mobile-search-box` が別定義 |
| 件数・解除 | input.css:762 `.receipts-search-count`、766 `.receipts-search-clear`(button.rs:98 `SearchClear` 経由) |
| 条件の追加行 | input.css:771 `.add-condition-row`、780 `.add-condition-button`(button.rs:95 `AddCondition` 経由) |
| 選択中の条件札 | input.css:741 `.active-condition-chip`(button.rs:94 `ConditionChip` 経由) |

### 1f. 見出し

| 役割 | 定義場所 |
|---|---|
| ページ名 | input.css:1357 `.page-frame-title` |
| ダイアログ名 | input.css:799 `.dialog-title`、819 `.condition-panel-title`、910 `.search-dialog-section-title` |
| 設定の節見出し | settings.rs:69/113/224/313/538 `class="text-heading font-bold mb-4"` の直書き×5、stock_search/view.rs:583 `mb-3`、592 `mb-2`。部品化されていない |

### 1g. 余白トークン(PC/スマホ二重)

input.css:143-157 `--space-*`(tabs/summary/note/search/condition/table/page/card 等)。スマホだけ別値が要る箇所は `-mobile` 側(`--space-tabs-mobile`、`--space-search-mobile`)を持つが、適用側が `max-lg:mt-0` 等のユーティリティ上書きで逃げている箇所がある(§3 参照)。

## 2. input.css にあるが .rs から使われていない独自クラス

確かめたこと(`class="..."`・`class:name=`・文字列リテラル・`format!` を横断し、`data-testid` のみの一致を除外):

| クラス | 定義 | 状況 |
|---|---|---|
| `.asset-compact-row` | input.css:1725(`border-b border-border`) | class 指定なし。`data-testid="asset-compact-row"`(holdings_list.rs:116、all_list.rs:419、fund_table.rs:230)が同名のため文字列一致するが、行の見た目は `list-row-item`(input.css:1761)で付けている。#1984 で指摘済みのまま残存 |
| `.asset-search-count` | input.css:1466(`mt-1.5 px-1 text-right ...`) | class 指定なし。`testid="asset-search-count"`(search_bar.rs:235)のみ。件数表示は `SearchCount` 部品が `receipts-search-count`(search_controls.rs:185)で出すため不要 |

要確認(動的に組み立てる名前): `tier.class()`→`receipt-tier-md`(2119)/`receipt-tier-wide`(2123)/`receipt-tier-wider`(2127)+`receipt-span-*`(2143-2161)は table.rs:172/237/268/300-301 で `format!` 合成。`align.class()`→`col-text`(1179)/`col-num`(1183)。`size.class()`→`summary-value-primary`(1497)/`secondary`(1501)(summary.rs:86-89)。`variant.class()`→Card/Button の対応表(card.rs、button.rs:62-115)。`comp-slice-{0-5}`(1833-1855)・`comp-bar-fill`(1896)・`comp-legend-dot`(1876)は `fill` 変数と合成(composition_page.rs:330/361/500)。いずれも静的部分が文字列リテラルにあり、使用中と判定。`has-action-bar`(1347)は `class:has-action-bar=` 指令(page_frame.rs:69)で使用中。

## 3. 画面側(部品の外)で余白・文字の大きさを直書きしている箇所

確かめたこと: `class=` のある行は features/ 521行・ui/ 281行。うち余白ユーティリティ(`m*`/`p*`/`gap`/`space-*`)を含む行は features/ 262行・ui/ 83行。文字サイズの生指定(`text-xs` 等)は検査(check-css-tokens.sh)の基準で features/ 0件に抑えられており、画面側は `text-heading`/`text-label`/`text-cell`/`text-body` トークンを使っている。余白は未規制。

代表例(ファイル:行):

- settings.rs:69/113/224/313/538 — 節見出し `text-heading font-bold mb-4`×5(538 は `text-negative` 付き)。同じ見た目の繰り返し
- stock_search/view.rs:583 — `h2 ... mb-3 text-heading`、592 — `h3 ... mb-2 text-label`
- asset_balance/view/view.rs:374/391/408 — `no-print mb-3 px-4 lg:px-0`×3(同じ行の繰り返し)
- asset_balance/view/main_content.rs:181 — `asset-panel-padded mx-4 mt-3 lg:mx-0`(部品の面に画面側の外余白を足す)
- asset_balance/view/fund_detail_page.rs:60、holding_detail_page.rs:81、composition_page.rs:121、stock_search/view.rs:296 — `no-print mb-3`(戻りリンク行の繰り返し)
- home/view/composition_card.rs:175/215 — `home-stacked-bar mt-3`/`mt-1.5`、176/216 — `home-composition-rows mt-1.5`(同じ部品に画面違いの外余白)
- home/view/dividend_card.rs — class=38行中32行が余白付き(ファイル別で最多)
- admin_logins.rs:340 — `pc-search-area mb-4`(位置決めフックに外余白を足す)
- search_entry.rs:106 — `ShowMoreButton` 内 `mt-4 flex justify-center`(ui/ 内だが画面寄りの余白)

ファイル別(class= 行数/うち余白): settings.rs 60/35、dividend_card.rs 38/32、stock_search/view.rs 35/21、receipts/view/cards.rs 28/19、composition_page.rs 28/22、login_history.rs 27/13、asset_table.rs 23/2、all_list.rs 22/2、admin_logins.rs 21/5、search_bar.rs 20/6、composition_card.rs 20/11、dividend_info.rs 19/13。

## 4. 同じ要素に独自クラスとユーティリティで同じ性質を二重に付けている箇所

確かめたこと(両方に同じ性質の宣言があるものだけ):

| 要素 | 独自クラス側 | ユーティリティ側 | ファイル:行 |
|---|---|---|---|
| 種別タブ | `.page-tab-list` は `position: relative`(input.css:603) | `relative` | receipts/view/tabs.rs:77、kind_tabs.rs:29 |
| タブ行の骨格 | `.page-skeleton-tabs` は `margin-top/bottom` をトークンで(input.css:1364-1367) | `max-lg:mt-0 max-lg:mb-0` | ui/page_skeleton.rs:22 |
| 表の面(入れ子) | `.table-card`(input.css:454: `bg-surface`、枠なし、角丸0.75rem) | 内側に `.table-frame`(input.css:1470: `bg-surface`、枠なし)+`overflow-clip`(`table-card` も `overflow-clip` 持ち) | receipts/view/table.rs:191/207/213-215 |
| 行ボタン | `.list-row-action`(input.css:1765: `block w-full px-4 py-3`) | `ButtonVariant::RowButton` が `border-0 bg-transparent cursor-pointer text-left` を追加(button.rs:109-110)。枠・背景の上書き | button.rs:109-110 |
| CSS内での上書き1 | `.more-link` は `ml-auto`(input.css:1579) | —(CSS内で `.home-footlink .more-link{margin-left:0}` input.css:1563-1565 が打ち消し) | input.css:1563-1565/1579 |
| CSS内での上書き2 | `.summary-value` は `leading-none`(input.css:1493) | —(CSS内で `.summary-band-main-toggle .summary-value-secondary{leading-normal}` input.css:1121-1123 が打ち消し) | input.css:1121-1123/1493 |

#1984 のタブ余白の二重指定(スマホで `.asset-mobile-tabs-links` に余白を重ねると32px)は、現状は解消済みを確認: input.css:1405-1406 に禁止コメントがあり、main_content.rs:296/426/617 の `asset-mobile-tabs-links` に余白ユーティリティは付いていない。

## 5. 決まりの案(統合担当が決める)

### 案A: 部品の見た目は部品の中だけで決め、画面側はクラスを足さない

- `Card`・`ListRow`・`SummaryBand`・`PageTab`・`SearchField`・`SearchCountAndClear`・`ShowMoreButton`・`EmptyState`・`Badge`・`Amount` の見た目(余白・文字サイズ・色・枠・角丸)は `frontend/src/ui/` の部品または対応する CSS クラスのどちらか1か所に置く。画面側(features/)は部品の props(`extra_class` 等)で外余白だけ渡せるが、部品の内側の性質を変えるクラスは付けない。
- 直す量の見積もり: features/ の余白付き class= 約262行。うち部品呼び出し箇所への集約が必要なのは§3代表例を中心に約30-50か所、ファイル数は12-15。`settings.rs` の節見出し×5は `SectionHeading` 部品1件にまとめられる。
- 機械で止める検査: `check-ui-primitives.sh` に「features/ で余白ユーティリティ(`m*`/`p*`/`gap`/`space-*`)の直書きを禁止(部品 props 経由のみ許可)」を追加。既存262行は許可リスト(ファイル:行)に凍結し、新規だけ落とす(check-css-tokens.sh の `css-type-spacing-baseline.txt` 方式と同様に基準超え方式)。

### 案B: 余白・文字の大きさはトークンだけ

- 余白は `--space-*`(input.css:143-157)、文字は `--text-*`(160-172)のトークン経由に限定。`mt-4`/`px-4`/`gap-2` 等の数値ユーティリティは features/ では使わず、意味のある名前の CSS クラスか部品に置き換える。`max-lg:mt-0` のような帯別の打ち消し(§4 page_skeleton 例)は `-mobile` トークンに吸収する。
- 直す量の見積もり: features/ の余白ユーティリティ行262行が対象。ただし `--space-*` への置換は1対1ではなく意味付けの整理が要るため、ファイル数は20-25、段階的に進める(まず§3代表例の重複パターンから)。
- 機械で止める検査: `check-css-tokens.sh` の `SPACE_PROP` 相当を Rust 側にも拡張し、features/ での `(m|p)(t|b|l|r|x|y)?-`・`gap-`・`space-[xy]-` を基準超え方式で凍結。新規は `scripts/css-type-spacing-baseline.txt` と同じ形式の基準ファイル(例 `css-spacing-baseline.txt`: 種別・ファイル・件数)で管理。

### 案C: 同じ役割は1つの定義に寄せ、別名・旧道を消す

- 対象: `.table-frame`→`.table-card` に一本化(table.rs:207、admin_logins.rs:468、login_history.rs:192)、`.asset-compact-list` と `.receipt-compact-list` の統合、`settings-toc-link` の基本形を CSS に移す(toc.rs:190 の `flex min-h-11 ... px-3 py-2` を `.settings-toc-link` 本体へ)、未使用2件(§2)の削除、CSS内上書き2件(§4)の解消(`.more-link` に `margin-inline-start:auto` の例外フラグ、`summary-value` の `leading` を用途別クラスに分離)。
- 直す量の見積もり: 6-8ファイル(CSS+table.rs+admin_logins.rs+login_history.rs+toc.rs+home 関連)。各直しは画面ごとの小さな PR に分け、変更前後で画像が変わらないことを確かめる(Issue #1987 §4の手順)。
- 機械で止める検査: 未使用クラスの検査を新設 — CSS 定義クラス一覧 × Rust 側トークン(`class="..."`+`class:name=`+文字列リテラル+`format!` 静的部分、`data-testid` 除外)の差分を取るスクリプト(今回の棚卸し手順を `scripts/check-css-unused.sh` として固定)。`format!` 合成・`class:name=` 指令は静的部分で拾う。`receipt-tier-*`/`receipt-span-*` のように Rust 側テスト(tests.rs:551-570)が CSS 対応を保証しているものは検査対象外にする。

## 6. やらなかったこと

- コードの変更(§5の実施、未使用クラスの削除、二重指定の解消)はやらない。決まりを統合担当が決めてから別 PR で直す
- 色の直書きの棚卸しはやらない(`check-css-colors.sh`+`css-color-baseline.txt` が既に管理しているため)
- `text-*` 生サイズの棚卸しはやらない(`check-css-tokens.sh` の基準で features/ 0件に抑えられており、追加の重複は見つからなかったため)
- E2E fixture や撮影スクリプトの本格改修はやらない(§7 の小さな直しのみ)
- 推測のまま残したもの: §1a の `.table-frame` が旧道か(要確認)、§2 の動的組立の完全性(目視で `format!` 静的部分を確認済みだが、変数経由のクラス名の網羅は未検証)

## 7. ついでの直し

- `frontend/scripts/ui-shots.sh` の `server_owns_port`: 子プロセスがない(かつ自身も待ち受けていない)場合、素の `return` が直前の `while` の終了状態(0)を返し、待ち受け未確認のまま成功を返していた(#1986 の指摘)。`return 1` に直し、理由のコメントを付けた
- 動作確認: `frontend/` で `npm run check:css` を実行し通過。`ui-shots.sh` を1回動かし撮影できることを確認(画面は変えていないため比較撮影なし)

Refs #1987
