# スマホの画面の組み立て方

対象: スマホ幅(390×844、`lg` 未満)のログイン後の全画面。基準画面は資産管理 `/assetbalance`。
調査基準: `main` の 2026-11-02 時点のソースと、架空データのモックで撮った画面。

画面ごとに操作感・表示方法をそろえるため、観点ごとに「決まり」と「守らせる共通部品」を1か所にまとめる。
画面の要件と各画面の配置は [ui-design.md](../ui-design.md)、用語と書式は `ui/labels.rs`・`ui/format.rs` を正本とし、ここでは画面の組み立て方だけを決める。

| 用語 | 意味 |
| --- | --- |
| 上バー | ページ名・件数・低頻度リンク・アカウントを並べる紺の帯(`PageFrame` の見出し) |
| 帯 | 集計を出す紺の面(`SummaryBand`)。主の数字1つと補助1行、開くと内訳 |
| 注記行 | 帯の直下の小さい文字の行(`SkeletonNote`)。時点・対象外・絞り込み中と「計算基準を見る」 |
| 検索入口 | 「検索・並べ替え」のボタンと、その下の条件タグ・件数の段 |
| 行 | 一覧の1件(`ListRow`)。1段目=名前と主の金額、2段目=補助情報と補助の金額 |
| シート | 画面下から出る面(`DetailDialog`)。行の詳細に使う |

## 画面の型

ログイン後の画面は、上から次の順に組み立てる。使わない段は飛ばしてよいが、順番は変えない。

```text
上バー(ページ名 [件数] ……… [低頻度リンク] [アカウント])
[戻る]                      ← 詳細画面だけ
タブ                        ← 種別を切り替える画面だけ
帯                          ← 数値を集計する画面だけ
注記行                      ← 出す文があるときだけ
検索入口                    ← 一覧を絞り込める画面だけ
一覧(行・グループ行)/ 詳細 / フォーム
──── ページの末尾 ────
フッター
下部ナビ(ホーム|資産管理|取引明細|銘柄検索)
```

## 観点ごとの決まり

| 観点 | 決まり | 守らせる部品 |
| --- | --- | --- |
| 上バー | ページ名は `PageFrame` の `title` だけで出す。一覧の全件数は `title_count` に1つだけ。右端は低頻度リンク(枠つきの小ボタン)→アカウントの順 | `ui/page_frame.rs` `PageFrame` |
| 戻る | 詳細画面の戻りは `PageFrame` の `back_href`/`back_label` だけで出す。パンくず・独自の戻るリンクは作らない | `PageFrame`(`page-back-link`) |
| 下部ナビ | ログイン後の全画面で出す。出さない画面(ログイン・404)を列挙する方式にする | `ui/site.rs` `BottomNav`、`app.rs` の表示判定 |
| タブ | 種別の切り替えは全幅の下線タブ。タブの行は `SkeletonTabs` に入れ、タブ1つは `.page-tab`。下線タブの下でさらに切り替えるもの(基準など)は `Segmented` | `ui/page_skeleton.rs` `SkeletonTabs`、`ui/underline_tabs.rs`、`ui/segmented.rs` |
| 帯 | 全幅・角丸なしでページと一緒に流れる(張り付かない)。主ラベルの下に時点か期間を1つ出し、補助1行は開閉で中身を変えない。開閉は帯全体を押す | `ui/summary.rs` `SummaryBand` を `SkeletonSummary` に入れる |
| 注記行 | 帯の直下に1つ。左に事実の文、右端に「計算基準を見る」。出す文も計算基準も無いときは行ごと出さない | `SkeletonNote` |
| 検索・並べ替えの入口 | 一覧を絞れる画面は、スマホでは本文内の「検索・並べ替え」ボタン1つにまとめる。押すと検索ダイアログ。並べ替えの見出しクリックや「＋条件」の横並びはスマホに出さない。例外は銘柄検索(検索が目的のため下部操作帯の入力) | `SkeletonSearch` + 共通の入口部品、`ui/search_dialog.rs` |
| 件数 | 全件数は上バー、絞り込み後の件数は検索入口の下の左端に「N件 / M件」。一覧の見出しに件数を重ねて出さない | `PageFrame.title_count`、`ui/search_controls.rs` `SearchCount` |
| 構成比への入口 | 件数行の右の1か所だけ。上バーのボタンは出さない(スマホで上バーが2段になるため) | 件数行の右のリンク |
| タブの切り替え | 種別の下線タブはスマホ幅で横スワイプでも切り替える(横が縦の2倍以上かつ約60px以上。端20px・ダイアログや開閉ボタンの操作中は無視) | `ui/underline_tabs.rs` `SwipeArea` |
| 一覧の見出し行 | 置かない。区切りが必要な一覧(取引明細)だけグループ行(月ごと)を置く。日ごとのグループは作らない | グループ行の共通部品 |
| 行 | 全一覧で `ListRow`。行の器は左右16px・上下12px・下に区切り線。口座は `AccountBadge`、日付は2段目の先頭に `M/D`。右端に矢印などの印は付けない。スマホでは行をカードで包まない | `ui/list_row.rs` `ListRow`・`ListRowButton`・`ListRowLink`・`ListRowStatic`・`ListGroupHeader`、`ui/badge.rs` `AccountBadge`、行の器 |
| 行を押したとき | 保有している物(銘柄・ファンド)は詳細画面へ移る。1回の出来事(入金・売買)はダイアログで開く。その場で展開はしない。全部見えている行(ログイン履歴など)は押せなくてよい | `<a>` + 行の器 / `ui/detail_dialog.rs` `DetailDialog` |
| リンクとボタン | 3種類だけ: 上バーの枠つき小ボタン、本文内の移動リンク(下線なし・強調色の文字に「→」)、戻る。下線付きリンクは外部サイトへのリンクだけ | `page-secondary-link-sm`、`more-link`(`ui/more_link.rs` `MoreLink`)、`page-back-link`、`ui/security_link.rs` |
| 数字 | 金額・率は `Amount` と `format.rs` を通す。お金の出入りと損益(配当の受取額・売買の損益・評価損益・前日比)と税の3つ(税引前・税額・税引後)は符号(+/−)と色の両方(緑/赤)で出す。税引前は損益または受取額としてプラスは+¥の緑・マイナスは−¥の赤、税額は出ていくお金として−¥の赤(還付のマイナスの税額は+¥の緑、0は¥0の通常色)、税引後は今のまま。残高・価格・元になる額(評価額・現在値・取得総額)は通常の文字色で出し、▲▼は使わない。DeltaかPlainかは `AmountKind` で表し、画面ごとに色を書かない。帯の内訳(PCの横並び・スマホの縦リスト)も同じ決まりで出す。日付は `format_iso_date`("YYYY/MM/DD")か `format_short_date`("M/D")。一覧の月の見出しは YYYY年M月 | `ui/amount.rs` `Amount`、`ui/format.rs` |
| 余白 | 文字の左端は画面端から16px。帯・タブ・行は端まで敷き、中の文字だけ16px。`page-gutter` の中にさらに `page-gutter` や余白つきのカードを重ねない | `PageFrame`(`page-gutter`)/`content-gutter` |
| スクロール | スマホはページ全体(`main.app-scroll`)の1本だけ。本文の内側にスクロール領域を作らない | `PageFrame` |
| 空・読み込み中・失敗 | 空は `EmptyState`、読み込み中は `ListSkeleton`、失敗は `ListLoadError`(再読み込みつき)。裏の再取得の失敗だけ `Alert` | `ui/empty_state.rs`、`ui/state.rs` |

## トークン

新しい画面はここから選ぶ。色・寸法の直書きは `npm --prefix frontend run check:css` で落とす。正本は `frontend/style/input.css` の `@theme`。

| 種別 | トークン | 用途 |
| --- | --- | --- |
| 面 | `page`・`surface`・`surface-sunken`・`surface-raised`・`segment`・`row-group` | ページ地・面・沈み面・浮き面・選択肢の地・グループ行の地 |
| 紺・金 | `brand`・`brand-deep`・`brand-lift`・`brand-border`・`brand-line`・`brand-muted`・`brand-text`・`gold` | 上バー・帯・選択中・金の差し色。紺面上の文字は `text-inverse`、紺面上の枠・印は `brand-text` |
| 帯・注意 | `band`・`band-border`・`warn-ink`・`nisa-soft` | 注意の帯・NISA系の淡い面 |
| 損益(明) | `profit-bright`・`loss-bright` | 紺の帯の中の正負(`.summary-band .text-positive`・`[data-negative]` の上書きで使う) |
| 損益 | `negative`・`negative-vivid`・`negative-strong`・`negative-soft`・`negative-border`、`positive`・`positive-strong`・`positive-vivid`・`positive-soft`・`positive-softer`・`positive-border` | `AmountKind::Delta` の正負。`data-negative="true"` と `text-positive` で出す |
| 強調・情報 | `accent`・`accent-bright`・`accent-ring`・`accent-strong`・`accent-vivid`・`accent-text`・`accent-text-soft`・`accent-soft`・`accent-softer`・`accent-border`・`accent-border-strong`、`info`・`info-strong`・`info-deep`・`info-soft`・`info-border`・`info-focus` | リンク・移動リンク(`more-link` は `accent-text`)・銘柄コード |
| 文字・罫線 | `ink`・`ink-hover`、`text`・`text-deep`・`text-strong`・`text-soft`・`text-muted`・`text-subtle`・`text-quiet`・`text-faint`・`text-inverse`・`text-inverse-muted`、`border`・`border-faint`・`border-subtle`・`border-strong`・`border-xstrong`・`border-bold`・`border-link` | 本文・補助・罫線。`text-faint` は薄い補助に限定 |
| 札 | `badge-specific`・`badge-general`・`badge-nisa`・`badge-old-nisa`(各 `bg`・`text`・`border` の3点) | `AccountBadge` の口座区分 |
| 図 | `chart-1`〜`chart-5`・`chart-previous`、`fill`・`fill-strong` | 系列色・前年比較・棒の下地 |
| 余白 | `space-tabs`・`space-summary`・`space-note`・`space-search`・`space-condition`・`space-table` | 骨格の段間(`page-skeleton-*` が使う)。値は `0.75rem`・`0.75rem`・`0.25rem`・`0.75rem`・`0.5rem`・`0.75rem` |
| 文字の大きさ | `text-eyebrow`(0.6875rem)・`text-cell`(0.75rem)、`text-summary-primary`(1.875rem)・`text-summary-secondary`(1.5rem)と compact 版(1.625rem・0.9375rem) | 図の目盛り・表の小文字・帯の主従 |
| 角丸・影・幅 | `radius-note`(1.35rem)、`shadow-elevation-1`・`shadow-elevation-2`・`shadow-card`・`shadow-card-hover`・`shadow-header`・`shadow-sheet-up`・`shadow-edge-lit`・`shadow-edge-accent`・`shadow-edge-brand`、`container-page`(90rem)・`container-wide`(1680px) | 角丸は他に `rounded-xl`(面)・`rounded-lg`(入力・選択)・`rounded-full`(タブ・札)。影と本文幅 |

## 共通部品と共通クラス

新しい画面はここから選ぶ。実装の props は [`frontend/README.md`](../../frontend/README.md)の部品表が正本で、ここでは選び方だけを決める。

| 用途 | 使う部品・クラス | 選び方 |
| --- | --- | --- |
| 一覧の行 | `ui/list_row.rs` `ListRow`・`ListRowButton`・`ListRowLink`・`ListRowStatic`。器は `list-row`・`list-row-main`・`list-row-name`・`list-row-primary`・`list-row-sub`・`list-row-meta`・`list-row-secondary`、行は `list-row-item` + `list-row-action`(押せる)・`list-row-action-static`(押せない) | 全一覧で使う。行全体がボタンなら `ButtonVariant::RowButton`(`ListRowButton`)、遷移なら `ListRowLink` |
| グループ見出し | `ListGroupHeader` + `list-group-header`・`list-group-title`・`list-group-summary`。表の行は `receipts-table-group` + `receipts-table-group-label` | 同じトークン(`bg-row-group`・金の左線 `--color-gold`)。表と行で別名を作らない |
| 帯 | `ui/summary.rs` `SummaryBand`・`SummaryItem`(`SummarySize::Primary`・`Secondary`、`SummaryTone::Neutral`・`Gain`・`Loss`)。枠は `summary-band`・`summary-band-grid`・`summary-band-group`・`summary-band-items`、項目は `summary-item`・`summary-label`・`summary-label-note`・`summary-value`・`summary-value-primary`・`summary-value-secondary`・`summary-note`・`summary-status`、スマホ開閉は `summary-band-main-toggle`・`summary-band-top-row`・`summary-band-hero-row`・`summary-band-chevron-circle`・`summary-band-toggle-chevron`・`summary-band-breakdown`・`summary-band-breakdown-row`・`summary-band-breakdown-value`、PC 残存は `summary-band-pc`・`summary-band-item-tag1` | 主1つ+補助1行。色は `SummaryTone` だけで選び、帯独自の色名は作らない |
| ダイアログ枠とタイトル | 枠は `dialog-panel`(角丸・枠・影・`max-height:88vh`)、題は `dialog-title`。中央は `DialogShell` + `search-dialog-root`・`search-dialog-backdrop`・`search-dialog-panel`・`search-dialog-head`・`search-dialog-body`・`search-dialog-footer`、条件は `condition-panel`・`condition-backdrop`・`condition-panel-title`・`condition-panel-actions`、詳細は `ui/detail_dialog.rs` `DetailDialog` + `detail-dialog-*`、削除確認は `ConfirmDeleteModal` | 枠と題は共通。配置・幅・内側スクロールだけ個別に残す |
| ボタンの種別 | `ui/button.rs` `Button`・`LinkButton`・`TabButton`(`ButtonVariant::PageTab`)・`Switch`。大きさは `ButtonSize::Xs`・`Sm`・`Md`。種別は `Primary`・`Secondary`・`Header`・`DangerSolid`(確認の確定)・`Ghost`(解除など静かな操作)・`Retry`(再読み込み)・`MenuItem`・`MenuItemDanger`(削除)・`CopyName`・`AssetSortColumn`・`ConditionChip`・`AddCondition`・`ConditionOption`・`YearOption`・`SearchClear`・`Segmented`・`PageTab`・`ConditionSegment`・`RowButton` | `features/` で `<button>` を直書きしない。行全体は `RowButton`、種別切替のタブは `PageTab`(`TabButton`) |
| 切り替え | `ui/segmented.rs` `Segmented` + `segmented`・`segment-option`、条件は `condition-segment`・`condition-option`・`condition-option-list`、年・月は `year-option`、種別タブは `page-tab-list`・`page-tab`・`badge-count`、下線タブの横 swipe は `tab-swipe-area`・`tab-swipe-track`・`tab-swipe-slide`・`tab-swipe-ink` | `Segmented` と条件の切り替えは同じ `segment-option`。`flex-1` は容器側だけに置く |
| リンク | `ui/more_link.rs` `MoreLink` + `more-link`、上バーの枠つき小は `page-secondary-link-sm`、戻りは `page-back-link`、銘柄は `ui/security_link.rs` `SecurityCodeLink`・`CopyableInstrumentName` + `security-code-link`・`copyable-name`・`copy-icon` | 本文内の移動は `MoreLink` に寄せる。下線付きは外部リンクだけ |
| バッジの種別 | `ui/badge.rs` `Badge`(`BadgeVariant::Positive`・`Info`・`Accent`・`Danger`・`AccountSpecific`・`AccountGeneral`・`AccountNisa`・`AccountOldNisa`・`Code`・`CodeFlush`・`Count`)・`AccountBadge`・`CodeBadge`。クラスは `badge` + `badge-positive`・`badge-info`・`badge-accent`・`badge-danger`・`badge-account`・`badge-account-specific`・`badge-account-nisa`・`badge-account-old-nisa`・`badge-code`・`badge-code-flush`・`badge-count`。表の薄字コードは `table-code` | 小さな札は `Badge` の種別で表す。角丸・余白の差は variant に集める |
| 金額の種別 | `ui/amount.rs` `Amount`(`AmountKind::Plain`・`Delta`、`block`)。`Delta` は `text-positive`・`data-negative="true"`、紺帯では明色に寄せる | 出入り・損益・税の3つ(税引前・税額・税引後)だけ `Delta`、残高・価格・元額は `Plain`。画面ごとに色を書かない |

## 例外として認めるもの

| 画面 | 例外 | 理由 |
| --- | --- | --- |
| 銘柄検索 | 検索欄を下部操作帯に置く | 検索そのものが画面の目的で、親指の届く位置に入力を置く |
| CSV管理 | 一覧が表(種類 \| 保存済み \| 状態 \| ⋯) | 5種類固定で、行ごとに状態と操作を並べる必要がある |
| 設定 | 帯・検索入口が無く、項目ごとの見出しで区切る | フォームの画面で、一覧の型に当たらない |

## 検査

- 画面の文言は `ui/labels.rs` から使う。`labels.rs`・テスト・受信CSV値(`csv_input.rs`)の外の日本語の直書きは `labels/tests.rs` の検査で落とす
- 新しい画面は `PageFrame` を使い、`page-gutter` を本文に自分で書かない(`full_bleed` の画面だけ `content-gutter` を使う)
- 撮影(`frontend/scripts/ui-shots.sh`)でスマホ390・ダークの上端・下端・帯を開いた後を撮り、基準の資産管理と並べて比べる
- 見た目の値を固定するテストは書かない。部品を使っているか(例: 一覧の `li` が `ListRow` を含む)だけを確かめる

根拠:

- [`frontend/src/ui/page_frame.rs`](../../frontend/src/ui/page_frame.rs): ページ名・戻る・本文・操作帯の枠。下部ナビは描かない。
- [`frontend/src/app.rs`](../../frontend/src/app.rs): 現在パスと表示判定。下部ナビ・フッター・本文スクロールの持ち主。
- [`frontend/src/ui/site.rs`](../../frontend/src/ui/site.rs): 共通ナビ・下部ナビ・アカウントメニュー・フッター。
- [`frontend/src/ui/page_skeleton.rs`](../../frontend/src/ui/page_skeleton.rs): タブ行・帯・注記行・検索行の順序と余白。
- [`frontend/src/ui/summary.rs`](../../frontend/src/ui/summary.rs): 集計の帯。開閉で内訳を出す。
- [`frontend/src/features/asset_balance/view.rs`](../../frontend/src/features/asset_balance/view.rs): 基準画面の組み立て(上バー・タブ・帯・注記・検索入口・一覧)。
- [`frontend/src/features/receipts/view.rs`](../../frontend/src/features/receipts/view.rs): 同じ骨組みで組み立てる取引明細側。

## 未確認

- 本番ブラウザでの遷移・表示結果は確認していない。図の分岐とリンクはコード上の対応を表す。

## 文書との食い違い

- [ui-design.md](../ui-design.md) の用語・書式と矛盾する所は見つからなかった。ここでは組み立て方だけを決め、用語の追加・変更はしない。
- 既存の Cloudflare 設計メモは移行時の記録であり、現行のスマホ幅の扱いには使わない。
