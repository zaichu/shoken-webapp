# Phase 6 相互レビュー (OpenCode → Codex)

対象: Phase 4 (`docs/design/work/phase4-codex/`) と Phase 5 (`docs/design/work/phase5-codex/`) の Codex 成果物。
本番コード (`frontend/`・`backend/`・`shared/`・`.github/`) は変更していない。
画像は下記「確認した画像」の 35 枚を実際に開いて目視した。

## 重要度「高」の指摘

### H-1. PC 上ナビの「取引明細」が資産管理を指している（誤遷移）

- 何が: 全 9 HTML の PC 上ナビで「取引明細」のリンク先が `assets.html` になっている。
- どの画面/ファイルで: `docs/design/work/phase5-codex/html/` の全 9 件（`grep -o 'href="[^"]*">取引明細'` で 9 件すべて `href="assets.html"`）。モバイル下部ナビも同じ。
- なぜ問題か: 取引明細は OpenCode 担当とはいえ、ナビの遷移先が誤っていると画面間レビュー・実装時のリンク対応表の信頼が崩れる。利用者が取引明細へ進めない。
- どう直すか: 正しい遷移先（OpenCode 側 `receipts-opencode.html` 相当）に差し替えるか、未確定なら `href="#"`＋「仮」注記にして `screens-status.md` に残件として書く。

### H-2. 投信の評価損益 ¥0 を欠測記号「—」で表示している（ゼロと欠測の混同）

- 何が: 資産一覧で投信行の評価損益が「—」（`class="numeric positive"`）になっている。
- どの画面/ファイルで: `docs/design/work/phase5-codex/html/assets.html`（PC 表・モバイル行とも）。`shots/assets-pc.png`・`shots/assets-dark.png` で目視確認。
- なぜ問題か: `data-check.md:9` は当該投信の評価損益を +¥0（0.00%）と検算している。ゼロは確定値であり欠測ではない。「—」は `labels.rs:170 MISSING` の欠測表示と読まれる。同じ画面の投信詳細（`fund-detail.html`）では評価損益・トータルリターンを ¥0 と正しく表示しており、画面間で矛盾する。さらに `positive` クラス付きで色にも意味を持たせている。
- どう直すか: ¥0 を符号なし・色なし（`positive` を外す）で表示する。欠測の行の扱いと区別する注記を `screens-status.md` に足す。

### H-3. 集計帯の損失色が増加色と同一＋トークン直書き

- 何が: `.summary .gain,.summary .loss{color:#8fe2bc}`（`docs/design/work/phase5-codex/html/components.css` 先頭 1 行に連結）。
- どの画面/ファイルで: 上記 CSS。`shots/home-pc.png` では利益表示なので顕在化していないが、損失が出た瞬間に誤表示になる。
- なぜ問題か: (1) `.loss` が増加色と同じでは損益の符号識別が色でできず、金融表示として誤認を招く。(2) `#8fe2bc` は `tokens.css` に存在しない直書きで、「画面の HTML が tokens の変数だけを使う」遵守（grep 確認）に反する。`foundations.md:38` のコントラスト表にもこの組み合わせの記載がない。
- どう直すか: 損失用の帯上トークン（例 `--on-brand-negative`）を `tokens.css` に追加し、light/dark 両方でコントラスト 4.5:1 を検算して `foundations.md` の表に追記する。`gain`/`loss` の色分けを画像で再確認する。

### H-4. ダイアログにセマンティクスが一切ない

- 何が: `role="dialog"`・`aria-modal`・`aria-labelledby`・フォーカス管理の指定がゼロ（`grep -c 'role=\|aria-' dialogs.html` = 0。`states-codex.html` の `aria-busy` 1 件のみが全 HTML の唯一の aria）。
- どの画面/ファイルで: `docs/design/work/phase5-codex/html/dialogs.html`。`shots/dialogs-pc.png`・`shots/dialogs-mobile.png` で目視。
- なぜ問題か: `components.md:18` はダイアログの対応部品を `DialogShell, DetailDialog` とし、フォーカスリング可視をうたっているが、設計 HTML に対応する指定がなく、実装者がそのまま写すとキーボード・スクリーンリーダー利用者が閉じる操作にたどり着けない。同ファイルに「閉じる」が 2 個並ぶ構成で、どちらが正かも不明。
- どう直すか: `section.dialog` に `role="dialog" aria-modal="true" aria-labelledby` を付け、閉じるボタンを 1 個に一元化し、Esc・フォーカストラップ・復帰先を実装注記として `dialogs.html` 先頭コメントまたは `screens-status.md` に書く。

### H-5. スマホの集計帯と月軸ラベルが破綻している

- 何が: (1) 集計帯の「評価損益 + / ¥1,552,488」が符号と金額の間で改行分断。(2) 月別棒グラフの 10〜12 月ラベルが縦に潰れて判読不能。
- どの画面/ファイルで: `docs/design/work/phase5-codex/shots/home-mobile.png`。原因は `components.css` の `.summary>div:not(:first-child)`（flex 行化）と `.month small{font-size:11px}`。
- なぜ問題か: 金額の符号分断は読み間違いの元で、PC（`home-pc.png` では一体表示）とスマホで表示が変わる。11px は `foundations.md:42` の最小文字 12px を下回る。
- どう直すか: 金額ブロックに `white-space:nowrap`（または符号＋金額を不可分要素化）し、月軸は 11px をやめて `--text-xs`（12px）＋ 3 か月間引き等の間引き規則を決めて `foundations.md` に書く。

## 重要度「中」の指摘

### M-1. チャート系列色がライト/ダークで変わる（Phase 2 指摘の未解決）

- 何が: `--chart-1/--chart-2` が light（紺/青）と dark（青/橙）で別色。`shots/home-pc.png` と `shots/home-dark.png`、`shots/composition-pc.png` と `shots/composition-dark.png` で目視確認。
- なぜ問題か: Phase 2（codex §5、opencode X-5 系）で「テーマを越えて同じ資産種別へ同じ色を」と指摘されたが、`tokens.css:17,27` の定義が原因で未解決。`components.md:24` の「系列に固定割当て」とも矛盾する。
- どう直すか: 系列色をテーマ不変にするか、変える意図があるなら `foundations.md` に対応表と理由を書く。

### M-2. labels.rs 正本との用語の食い違いが 6 件ある

- 対象と根拠（いずれも `frontend/src/ui/labels.rs` が正）:
  1. `home.html` 集計帯「年間配当」＋小書き「税引前・2026年」→ 正は `ANNUAL_EXPECTED_DIVIDEND`「年間予想配当(税引前)」。「予想」の欠落は実績との混同を招く。
  2. `stock-search.html`・`holding-route.html`「年間配当(1株)」→ 正は `PER_SHARE_DIVIDEND`「1株配当」。
  3. 同「配当利回り」→ 表示 3.00% は `data-check.md:12` の通り ¥360 ÷ ¥12,000（`fixtures.json` の `annualDividendPerShare` ÷ `currentPrice`）の株価基準であり取得額基準ではないため、`labels.rs` に保有詳細用の「配当利回り(株価基準)」の定義を追加するか、画面側に算出根拠を注記する。
  4. `fund-detail.html` 保有状況「価格基準日」→ 正は `BASE_PRICE_AS_OF`「基準価額の基準日」。同一画面の注記行は「基準価額の基準日」で表記ゆれしている（`shots/fund-detail-pc.png`）。
  5. `states-codex.html` 空カード「保有資産がありません」→ 正は `ASSET_EMPTY_TITLE`「資産管理データがありません」。
  6. `states-codex.html` エラーカード「一覧を読み込めませんでした」→ 正は `API_DATA_LOAD_FAILED` 系の文言。
- どう直すか: 上記を labels 定数に置換し、labels にない文言は先に labels へ追加する（`components.md:3` の運用通り）。置換表を `screens-status.md` に残す。

### M-3. 月別棒グラフが 6 月一点集中で他月が判読不能（Phase 2 HO-1 の未解決）

- どの画面で: `shots/home-pc.png`・`shots/home-mobile.png`、`home.html` の `.chart`（6 月 100% に対し他月 2〜5%）。
- なぜ問題か: Phase 2 opencode H-1/HO-1「振り切れて比較として機能していない」のまま。6 月 ¥3,500,000 の単月集中が fixture 由来とはいえ、設計がスケール方針を示していない。
- どう直すか: 上限クリップ＋超過注記、対数軸の不採用理由、月別表への誘導のいずれかを決めて `home.html` に反映し、Phase 3 決定に沿う。

### M-4. 状態ボードの空・エラーカードが自前の EmptyState 型を満たさない

- どの画面で: `docs/design/work/phase5-codex/html/states-codex.html`（`shots/states-codex-pc.png`）。アイコンなし、次の操作なし（空カードに CSV 導線がない）。
- なぜ問題か: `components.md:20` の空状態＝「アイコン・見出し・説明・次の操作」に反し、Phase 2 codex §8 の「空は次の行動を共通位置に」とも逆行する。
- どう直すか: `EmptyState` 準拠（アイコン＋操作ボタン付き）に描き直す。

### M-5. 構成比の基準タブがダミー（評価額選択中も年間配当セクションを同時表示）

- どの画面で: `docs/design/work/phase5-codex/html/composition.html`（tabs は `href="#"` と `#dividend` アンカーのみ）。`shots/composition-pc.png` 下部に両基準のセクションが縦積み。
- なぜ問題か: タブ切替の設計なのか 1 画面に両方出す設計なのか実装者が判断できない。`screens-status.md:18` の「年間配当の内訳を表示」とタブの関係が未定義。
- どう直すか: 切替式なら片方だけ表示＋切替注記に、積む式ならタブをやめて見出し構成にする。どちらかにして `composition.html` を修正。

### M-6. 並べ替え UI が表に重なり、閉じる・クリア・適用後表示がない

- どの画面で: `shots/assets-pc.png`（ポップオーバーが列見出しに重なる）、`shots/assets-mobile.png`（シートに「適用する」のみ）。`assets.html` に backdrop なし、`SHEET_CLEAR`（クリア）・`DIALOG_CLOSE`（閉じる）未使用。
- なぜ問題か: Phase 2 RE-2 と同根のまま。適用後の条件チップ・件数（`SearchCount`）もなく、絞り込み中か分からない。
- どう直すか: backdrop＋閉じる＋クリアを追加し、適用後は条件チップと件数を表示する（`components.md:13` の型通り）。静的例示でも適用後状態を 1 枚足す。

### M-7. トークン外の直書きサイズ・余白が残っている

- 内容: `.month small{font-size:11px}`、`font-size:18px/20px/24px`（`--text-lg/xl/2xl` 未使用）、`style="margin-top:12px/16px/8px"・gap:8px`、`border-radius:16px`（`--radius-lg`=12px と不一致）等。`components.css` と各 HTML の grep で確認。
- なぜ問題か: 「tokens の変数だけを使う」遵守に反し、テーマ・密度の一括変更が効かない。11px は可読下限割れ（H-5 の一因）。
- どう直すか: 上記を `--text-*`・`--space-*`・`--radius-*` に置換する。`?dark` 切替で変わらない固定色 `#8fe2bc`（H-3）も同時に除去する。

### M-8. 銘柄検索の外部リンクが 1 件のダミーのみ

- どの画面で: `stock-search.html`・`holding-route.html` の「企業情報を見る ↗」（`href="https://example.invalid"`）。`shots/stock-search-pc.png` で目視。
- なぜ問題か: 実装は楽天・SBI・株探・日経等 8 リンク群（`labels.rs:954-968`）で、配置・優先順・モバイル時の扱いの設計判断がない。
- どう直すか: 8 リンクの並べ方（メニュー化か一覧か）を決めて描くか、未決なら残件として `screens-status.md` に明記する。

### M-9. 実装対応表と工数見積りがない

- 内容: 設計の `.table` と既存 `.asset-table`・`.receipt-table`（`frontend/style/input.css`）の対応、`DialogShell`・`DetailDialog` の props 対応、既存 `ButtonVariant` との対応がない。`screens-status.md`・`data-check.md` に工数記述なし。
- なぜ問題か: 観点「実装できる構成か／工数見積りが妥当か」に対し検証材料がなく、そのままでは実装者が作り直す。
- どう直すか: 部品ごとの既存対応表（流用／拡張／新設）と、画面ごとの工数メモ（S/M/L で可）を `screens-status.md` に追記する。

### M-10. 適用中の条件チップ・件数表示がない

- どの画面で: `assets.html`（検索・並べ替えのみで `filtering_count` 相当なし）。
- なぜ問題か: `components.md:13` の `SearchCount` と乖離し、絞り込み中かの判別がつかない。
- どう直すか: M-6 と合わせて適用後状態を追加する。

### M-11. ダイアログの重複と情報不足

- 内容: 「閉じる」がヘッダー＋フッターに 2 個（`dialogs.html`）。保有株詳細に口座・現在値の基準日・配当情報がなく、実装の `DetailDialog` より薄い（`shots/dialog-holding-pc.png`）。PC で評価損益「+ / ¥200,000」が改行分断される（タイル 4 列の幅不足）。
- どう直すか: 閉じるを一元化し、保有株詳細に口座・基準日を足す。タイルは `white-space:nowrap` または 2 列化する。

### M-12. ホーム「最近の配当」が税引後のみ

- どの画面で: `shots/home-pc.png`（¥10,957・¥23,905 のみ）。税引前・税額はダイアログを開く必要がある。
- なぜ問題か: Phase 2 RE-1（モバイル一覧で税額が見えない）と同根。意図的な省略か設計漏れか不明。
- どう直すか: 省略方針なら Phase 3 決定に記録し、出すなら 2 段目に税額目安を足す。どちらかを `screens-status.md` に書く。

### M-13. 状態ボードに fixtures 外の銘柄名がある

- 内容: `states-codex.html` の「架空新興国ファンド」「架空グローバル・次世代技術革新…」は `data/fixtures.json` に存在しない（grep 確認：fixtures の架空名は 3 件のみ）。
- なぜ問題か: `data-check.md` の「全画面は同一 fixtures から生成」の主張と矛盾し、突き合わせの信頼を損なう。
- どう直すか: fixtures 内名称（または存在しない旨の注記）に統一する。

## 重要度「低」の指摘

- L-1. `fund-detail-mobile.png` で「資産管理へ戻る」が 2 行に折り返す。ボタン幅を確保するか短い文言にする。
- L-2. `stock-search-pc.png`・`stock-search-mobile.png` で「銘柄を検索」ボタンが縦 2 行になる。`white-space:nowrap` で抑える。
- L-3. 資産表の投信行コード欄「—」がコードなしなのか欠測なのか注記なし（`assets.html`）。`MISSING` の運用を注記する。
- L-4. 年間配当セクションの凡例色がドーナツと不整合（`composition.html` の配当側で `chart-2` を 9901 に使用）。系列色の割当規則を決める（M-1 と合わせて）。
- L-5. ホーム「配当の詳細」→ `dialogs.html` への別ページ遷移は実装（モーダル）と異なる。静的例示の旨を注記するか、モーダル想定の注記を足す。
- L-6. 構成比の「2026年見込み」など細部の言い回しが labels にない。M-2 と合わせて labels 先行で統一する。
- L-7. `phase4-codex/html/foundations.html` に `style="...color:white...color:#111827"` の直書きが各 1 件。Phase 4 見本のため実害は小さいが、M-7 と合わせて除去する。

## 良い点

- 時点表示が年付きに統一された（「価格基準日 2026/11/13」「データ: 2026/11/14 取り込み」）。Phase 2 codex §1（年なし基準日の混在）は Codex 担当画面では解決している。
- 数値の検算が強い。`data-check.md` の 379 assert＋月別・行別の突き合わせ表で、ホーム最近の配当（2026/12/24 ¥10,957 等）と明細の一致まで追える。
- `tokens.css` が Phase 4 と Phase 5 でバイト同一で、土台のブレがない。
- 金額の右揃え・桁区切り・符号併記・等幅数字・欠測「—」の基本が全画面で統一され、ダークでも崩れ・はみ出し（H-5 を除く）はない。
- 構成比の凡例は色＋名称＋金額＋割合が同一行で対応し、割合の桁そろえもできている。割合（32.0%/68.0%、13.1%/86.9%）は検算と一致する。
- 投信詳細のタイル群はラベル・年日付・ゼロ表示（¥0）が正しく、PC／スマホ／ダークで破綻がない。
- `:focus-visible` の土台が `tokens.css:33` にあり、下部ナビ 56px・ボタン 44px のタップ領域も確保されている。
- `screens-status.md` が担当範囲と OpenCode 担当分の切り分けを明示し、過主張していない。

## 全画面の網羅の確認表

`screen-inventory.md` の項目と突き合わせ。○＝過不足なし、△＝静的例示のみ・一部不足、×＝未デザイン。OpenCode 担当分は対象外と記載する。

| 画面・状態 | 判定 | 備考 |
|---|---|---|
| ホーム（集計帯・配当・構成・損益上位・最近の配当） | ○ | H-5・M-3・M-12 の手直しは要るが項目は揃う |
| 資産管理・すべて（一覧・詳細パネル） | ○ | H-2 の ¥0 修正が必要 |
| 資産管理・国内株式タブ | ○ | 小計・行ともあり |
| 資産管理・投資信託タブ | ○ | 小計・行・詳細リンクあり |
| 構成比・評価額基準 | ○ | M-1・M-5 の整理が必要 |
| 構成比・年間配当基準 | ○ | 同上。区分切替（銘柄別/商品別/口座別）は未デザイン（×、下記） |
| 投資信託詳細 | ○ | 実画面の更新待ち状態はなし（△相当・下記「更新待ち」行） |
| 銘柄検索（結果あり・保有あり） | ○ | M-8 の外部リンク未決 |
| 銘柄検索（初期・該当なし・失敗・未ログイン） | △ | `states-codex.html` の静的カードのみ。検索画面自体の状態なし |
| 旧保有詳細 URL → 銘柄検索への置換 | ○ | `holding-route.html` あり |
| 配当詳細ダイアログ | ○ | H-4 のセマンティクス修正が必要 |
| 保有株詳細ダイアログ | ○ | 同上＋M-11 |
| 資産の検索・並べ替え UI | ○ | 静的例示あり。M-6・M-10 の適用後状態なし |
| 読込中・空・検索 0 件・取得失敗 | △ | 状態ボードのみ。実画面の分岐なし。M-4・M-2-⑤⑥ |
| 一部価格未取得・更新待ち | △ | 同上。名称が fixtures 外（M-13） |
| 長い名称・桁の多い値・複数口座 | ○ | 全幅表示・桁そろえあり |
| 構成比の区分切替（銘柄別/商品別/口座別） | × | `COMPOSITION_GROUP_*` のデザインなし |
| 取引明細 3 タブ・条件パネル・行詳細 | 対象外 | OpenCode 担当（`../phase5-opencode/`） |
| CSV 管理・削除確認・行メニュー | 対象外 | 同上 |
| 設定 7 セクション・ログイン履歴・端末 | 対象外 | 同上 |
| 管理者ログイン一覧・権限なし | 対象外 | 同上 |
| ログイン・404・アカウント削除 | 対象外 | 同上 |

## 確認した画像（35 枚・すべて目視）

Phase 5（27 枚）: `home-pc/mobile/dark`、`assets-pc/mobile/dark`、`composition-pc/mobile/dark`、`fund-detail-pc/mobile/dark`、`stock-search-pc/mobile/dark`、`holding-route-pc/mobile`、`dialogs-pc/mobile/dark`、`dialog-dividend-pc/mobile`、`dialog-holding-pc/mobile`、`states-codex-pc/mobile/dark`（いずれも `docs/design/work/phase5-codex/shots/`）。
Phase 4（8 枚）: `foundations-pc-light/mobile-light/pc-dark/mobile-dark`、`components-pc-light/mobile-light/pc-dark/mobile-dark`（いずれも `docs/design/work/phase4-codex/shots/`）。

## 未確認

- `docs/design/work/phase5-codex/shots/holding-route-dark.png`（ダーク変種のみ未見。PC/モバイルは確認済みでレイアウト差異の恐れは小さい）
- キーボードフォーカスの実際の見え方、コントラスト比の実測値、44px 未満のタップ領域の有無（静止画＋コード目視のみで断定できない）
- 実機ブラウザでの表示、スクリーンリーダーの読み上げ順
