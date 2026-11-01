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
| シート | 画面下から出る面(`BottomSheet`)。行の詳細に使う |

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
| 一覧の見出し行 | 置かない。区切りが必要な一覧(取引明細)だけグループ行(月ごと)を置く。日ごとのグループは作らない | グループ行の共通部品 |
| 行 | 全一覧で `ListRow`。行の器は左右16px・上下8px・下に区切り線。口座は `AccountBadge`、日付は2段目の先頭に `M/D`。右端に矢印などの印は付けない。スマホでは行をカードで包まない | `ui/list_row.rs` `ListRow`、`ui/badge.rs` `AccountBadge`、行の器 |
| 行を押したとき | 保有している物(銘柄・ファンド)は詳細画面へ移る。1回の出来事(入金・売買)はシートで開く。その場で展開はしない。全部見えている行(ログイン履歴など)は押せなくてよい | `<a>` + 行の器 / `ui/row_detail_sheet.rs` `RowDetailSheet` |
| リンクとボタン | 3種類だけ: 上バーの枠つき小ボタン、本文内の移動リンク(下線なし・強調色の文字に「→」)、戻る。下線付きリンクは外部サイトへのリンクだけ | `page-secondary-link-sm`、`home-card-link` を共通名にしたもの、`page-back-link`、`ui/security_link.rs` |
| 数字 | 金額・率は `Amount` と `format.rs` を通す。増減(損益・前日比・前月比)は符号と色の両方、増減でない金額は色を付けない。日付は `format_iso_date`("YYYY/MM/DD")か `format_short_date`("M/D") | `ui/amount.rs` `Amount`、`ui/format.rs` |
| 余白 | 文字の左端は画面端から16px。帯・タブ・行は端まで敷き、中の文字だけ16px。`page-gutter` の中にさらに `page-gutter` や余白つきのカードを重ねない | `PageFrame`(`page-gutter`)/`content-gutter` |
| スクロール | スマホはページ全体(`main.app-scroll`)の1本だけ。本文の内側にスクロール領域を作らない | `PageFrame` |
| 空・読み込み中・失敗 | 空は `EmptyState`、読み込み中は `ListSkeleton`、失敗は `ListLoadError`(再読み込みつき)。裏の再取得の失敗だけ `Alert` | `ui/empty_state.rs`、`ui/state.rs` |

## 例外として認めるもの

| 画面 | 例外 | 理由 |
| --- | --- | --- |
| 銘柄検索 | 検索欄を下部操作帯に置く | 検索そのものが画面の目的で、親指の届く位置に入力を置く |
| CSV管理 | 一覧が表(種類 \| 保存済み \| 状態 \| ⋯) | 5種類固定で、行ごとに状態と操作を並べる必要がある |
| 設定 | 帯・検索入口が無く、項目ごとの見出しで区切る | フォームの画面で、一覧の型に当たらない |

## 検査

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
