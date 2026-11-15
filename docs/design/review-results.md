# 相互レビュー結果(Phase 6 指摘一覧と高の対応状況)

対象: `docs/design/work/phase6-codex/review-of-opencode.md`(Codex→OpenCode。全33枚目視)、`docs/design/work/phase6-opencode/review-of-codex.md`(OpenCode→Codex。全35枚目視)。Phase 7 は本番コードを変えず、見本(HTML・CSS・tokens.css、必要なら phase4-*)の側で高の9件を直した。各高の対応状況は「Phase 7 で修正済み(根拠つき)」で記録し、残りは中・低の残件とする。

## 高の指摘(9件)と対応状況

### 高-1(Codex→OpenCode-1) 共通トークンのある色・余白が画面CSSに直接指定

- 内容: `site.css:5-13` の `#fff`、`components.css:1` の padding、`settings-opencode.html:48-69` の margin、`finance.html:10-50,60` の余白・文字サイズの直書き。
- 対応: **Phase 7 で修正済み(根拠: `docs/design/work/phase5-opencode/html/site.css:5,7,9,12,35,44,121,127` の `#fff`→`var(--on-brand)`、`docs/design/work/phase5-opencode/html/components.css:1` の `padding:12px`→`var(--space-3)`・`8px`→`var(--space-2)`・`color:white`→`var(--on-brand)`、`docs/design/work/phase5-opencode/data/templates/settings-opencode.html` の `margin-bottom:16px`→`var(--space-4)`(6件)・`margin-top:4px/12px`→`var(--space-1)/var(--space-3)`・`24px`→`var(--space-6)`、`docs/design/work/phase4-opencode/finance.html:10-50` の余白・文字サイズ直書きの変数化(例外は `min-height:44px`(タップ領域)・`1px/2px`罫線・`3px`強調罫・`960px`面幅・`120px`図形・`640px`切替点・`30px`円環内側・`bottom:56px`・ポップオーバー座標 `top:52px/left:440px/width:320px`)、`docs/design/work/phase4-opencode/tokens.css:43-49` への `--text-*`・`--space-*`・`--radius-*`追加、画像 `docs/design/work/phase4-opencode/shots/finance-pc-light.png`・`finance-sp-light.png`)**。既存トークンで表せない値は例外理由を本項に記し、残りは M-7(実装 PR-1・PR-9)に残す。

### 高-2(Codex→OpenCode-2) Phase 4 OpenCode 文書・見本と Phase 5 共有トークンの対応未整理

- 内容: `cmp` で Codex/Phase5 の `tokens.css` はバイト同一。一方 OpenCode `layout.md:133`・`tokens.css:1-7,59` は現行抜粋で名前(`--color-page` 対 `--page`)・ダーク正数色(`#53d28a` 対 `#7ce8ab`)が異なる。
- 対応: **Phase 7 で修正済み(根拠: `docs/design/work/phase4-opencode/tokens.css:1-10` 先頭コメントの対応表(`--color-page`→`--page` 等、ダーク正数色は抜粋 `#53d28a` に対し共有 `#7ce8ab` を正と明記)、`docs/design/work/phase4-opencode/layout.md` 未確認節の同旨更新)**。`design-system.md` §1 の「Codex 名・値を正」と合わせて、Phase 4 見本の表示は変えず文書で整理した。

### 高-3(Codex→OpenCode-3) 金額表示が金融表示ルールに反して等幅書体を指定

- 内容: `finance-data.md:18` は `font-mono` 禁止・本文書体+`tabular-nums` とするが、両 `tokens.css:9,34` の `--font-mono`・`.numeric` と OpenCode `site.css` が金額に等幅書体を使う。
- 対応: **Phase 7 で修正済み(根拠: 3 tokens.css(`docs/design/work/phase4-codex/tokens.css:9,36`・`docs/design/work/phase5-codex/html/tokens.css`・`docs/design/work/phase5-opencode/html/tokens.css`。`cmp` で3ファイル同一確認)から `--font-mono` 定義と `.numeric`・`.summary strong`・`.band-item .v`・`.tbl .num/.code`・`.mrows .amt`・`.kv dd`・`.num` の等幅指定を除去し本文書体+`tabular-nums` に統一。`grep -rn 'font-mono|monospace|Consolas' docs/design/work/phase{4,5}-*/html/ phase4-opencode/finance.html phase4-codex/html/` は0件。画像 `home-pc.png`・`receipts-dividend-pc-light.png` で金額表示を目視)**。`finance-data.md:18` の禁止規定は変更なし。

### 高-4(Codex→OpenCode-4) 危険操作の確認ダイアログにキーボード操作の設計がなく削除確定のフォーカスも不明

- 内容: CSV 6件+アカウント1件の計7件(`csv-opencode.html:92-133`・`account-opencode.html:11-12,51-56`)に `role="dialog"` 等はあるが初期位置・トラップ・Esc・復帰・確定挙動が不明。`receipts-opencode.html:277-291` の `rcpt-d1` は行詳細であり対象外。
- 対応: **Phase 7 で修正済み(根拠: `docs/design/work/phase5-opencode/data/templates/csv-opencode.html` の削除確認6件(`csv-delete`・`csv-delete-div`・`csv-delete-fund`・`csv-delete-hold-stock`・`csv-delete-hold-fund`・`csv-delete-hold`)と `docs/design/work/phase5-opencode/data/templates/account-opencode.html` の `account-delete` に `aria-describedby` とキーボード操作注記(`.kbd-note`。初期フォーカスは「キャンセル」、Tab は確認内循環、Esc はキャンセルと同じく閉じて元のボタンへ復帰、確定は削除ボタンのみ)を追加。`docs/design/work/phase5-opencode/html/site.css:50` に `.kbd-note` 定義。画像 `docs/design/work/phase5-opencode/shots/csv-delete-pc-dark.png`・`account-delete-pc-dark.png` で注記表示を目視)**。`receipts-opencode.html` の行詳細(`rcpt-d1` 等)は危険操作でないため対象外のまま。

### H-1(OpenCode→Codex) PC上ナビの「取引明細」が資産管理を指している(誤遷移)

- 内容: Phase 5 Codex 全9 HTML の PC上ナビ・モバイル下部ナビで「取引明細」のリンク先が `assets.html`。
- 対応: **Phase 7 で修正済み(根拠: `docs/design/work/phase5-codex/data/templates/` 全9件の PC 上部ナビとモバイル下部ナビの「取引明細」リンク先を `../phase5-opencode/html/receipts-opencode.html` に差し替え。`grep -c 'href="assets.html">取引明細'` は全9件で0件。全 HTML を `npm run build`(381 asserts 通過)で再生成し、画像 `docs/design/work/phase5-codex/shots/` 全27枚+`docs/design/work/phase5-opencode/shots/` 全30枚を撮り直し)**。

### H-2(OpenCode→Codex) 投信の評価損益¥0を欠測記号「—」で表示(ゼロと欠測の混同)

- 内容: `assets.html` で投信行の評価損益が「—」(`positive` 付き)。`data-check.md:9` は +¥0(0.00%)と検算し、同一画面の `fund-detail.html` では ¥0 と正しく表示して矛盾する。`MISSING`(「—」)は欠測の表示。
- 対応: **Phase 7 で修正済み(根拠: `docs/design/work/phase5-codex/data/templates/assets.html` の PC 表セルとモバイル行を `<td class="numeric">{{FUND_GAIN}}</td>`・`<small>{{FUND_GAIN}}</small>` に変更し符号・色なしの ¥0 で表示(`positive` 除去)。`FUND_GAIN` は fixtures 由来で build 検算(381 asserts 通過)により ¥0。`fund-detail.html` の ¥0 と一致。画像 `docs/design/work/phase5-codex/shots/assets-pc.png`・`assets-mobile.png`・`assets-dark.png` で目視)**。投信行コード欄の「—」は銘柄コードなしの正規表示(`MISSING` の欠測表示と混同しない旨を `screen-design-status.md` に注記)。

### H-3(OpenCode→Codex) 集計帯の損失色が増加色と同一+トークン直書き

- 内容: `components.css` 先頭1行 `.summary .gain,.summary .loss{color:#8fe2bc}`。`loss` が増加色と同一で符号識別が色でできない。`#8fe2bc` は `tokens.css` にない直書き。
- 対応: **Phase 7 で修正済み(根拠: 3 tokens.css に `--on-brand-positive`(light `#8fe2bc`/dark `#7ce8ab`)と `--on-brand-negative`(light `#fdecea`/dark `#f5a49b`)を追加し、`docs/design/work/phase5-codex/html/components.css` の `.summary .gain/.loss` と `docs/design/work/phase5-opencode/html/site.css:49` の `.band .pos/.neg` を参照に変更。`--positive`(light `#0b7a4b`)は帯背景(`#0f2a4a`)上で 2.69:1 しかなく使えないことを検算で確認したため帯上では使わない。`python3 docs/design/work/phase4-codex/scripts/contrast.py` で light/dark 28組合せすべて 4.5:1 以上(帯上4組合せ: 9.50/12.66/9.64/7.35:1)を確認し `docs/design/work/phase4-codex/foundations.md` の表を更新。画像 `home-pc.png`・`home-dark.png` で利益色を目視)**。

### H-4(OpenCode→Codex) ダイアログにセマンティクスが一切ない

- 内容: `dialogs.html` に `role`・`aria-*`・フォーカス管理の指定がゼロ(`grep` で0件)。「閉じる」が2個並ぶ。
- 対応: **Phase 7 で修正済み(根拠: `docs/design/work/phase5-codex/data/templates/dialogs.html` の2ダイアログ(`#dividend`・`#holding`)の `section.dialog` に `role="dialog"`・`aria-modal="true"`・`aria-labelledby`(`#dividend` は `aria-describedby` 付き)を付与し、配当詳細の重複「閉じる」(フッター側)を除去して1個に一元化。Esc・フォーカストラップ・復帰先を `.dialog-note`(`docs/design/work/phase5-codex/html/components.css` に定義)として両ダイアログに記載。画像 `docs/design/work/phase5-codex/shots/dialogs-pc.png`・`dialogs-mobile.png`・`dialogs-dark.png`・`dialog-holding-pc.png`・`dialog-holding-mobile.png` で目視)**。

### H-5(OpenCode→Codex) スマホの集計帯と月軸ラベルが破綻

- 内容: `home-mobile.png` で集計帯「評価損益 + / ¥1,552,488」が符号と金額の間で改行分断。月別棒の10〜12月ラベルが潰れて判読不能。原因は `components.css` の flex 行化と `.month small{font-size:11px}`(最小12px割れ)。
- 対応: **Phase 7 で修正済み(根拠: `docs/design/work/phase5-codex/html/components.css` の `.summary strong`・`.tile strong`・`.numeric` に `white-space:nowrap` を追加し符号+金額の不可分化、`.month small` を `11px`→`var(--text-xs)`(12px)にし奇数月ラベル非表示(`.month:nth-child(odd) small{display:none}`)の間引きで偶月(2/4/6/8/10/12月)のみ表示、間引き規則を `docs/design/work/phase4-codex/foundations.md` の余白・角丸・影節に記載。画像 `docs/design/work/phase5-codex/shots/home-mobile.png` で集計帯一体表示と月軸判読を目視)**。月別棒の単月集中(M-3)はスケール方針未決として残件に残す。

## 中の指摘(概要。詳細は各レビューが正本)

- Codex→OpenCode 中-5: Phase 1 の複数状態(CSV取得中・取込中・失敗、設定保存中/失敗、端末失効確認等)が OpenCode 担当画面に未反映 → PR-7 で追加。
- Codex→OpenCode 中-6: 管理者「+場所」が非操作要素(`span`) → PR-7 でボタン化+aria対応。
- Codex→OpenCode 低-7: Phase 4 金融例(¥3,345,000・¥300,000)と Phase 5 fixtures(¥28,593,148・¥3,163,438)の数値セット差 → 本Phase7で `design-system.md` §3 に「別データ」と明記し解消。
- OpenCode→Codex M-1: 系列色が light/dark で変わる → PR-5 でテーマ不変か対応表に整理。
- OpenCode→Codex M-2: `labels.rs` 正本との用語食い違い6件(「年間配当」→「年間予想配当(税引前)」、「年間配当(1株)」→「1株配当」、配当利回りの基準注記、「価格基準日」→「基準価額の基準日」、空・エラー文言) → PR-7 で labels 先行で置換。
- OpenCode→Codex M-3: 月別棒の単月集中(Phase 2 HO-1 未解決) → PR-3 で方針決定。
- OpenCode→Codex M-4: 状態ボードが `EmptyState` 型(アイコン・次の操作)不適合 → PR-2 で描き直し。
- OpenCode→Codex M-5: 構成比の基準タブがダミー(両基準を縦積み) → PR-5 で切替/積みの二択。
- OpenCode→Codex M-6・M-10: 並べ替え UI の重なり・閉じる等なし、適用後チップ・件数なし → PR-4 で追加。
- OpenCode→Codex M-7: トークン外の直書きサイズ・余白(`11px`・`18/20/24px`・`margin-top`・`gap:8px`・`radius:16px`) → PR-1・PR-9 で置換。
- OpenCode→Codex M-8: 銘柄検索の外部リンクがダミー1件(`example.invalid`) → PR-8 で8リンク(`labels.rs:954-968`)の並べ方を決めるか残件明記。
- OpenCode→Codex M-9: 実装対応表・工数見積なし → 本Phase7で `implementation-plan.md` に初版を記録。
- OpenCode→Codex M-11: ダイアログ重複・情報不足(閉じる2個・口座/基準日なし・符号分断) → PR-8・PR-4 で修正。
- OpenCode→Codex M-12: ホーム「最近の配当」が税引後のみ → PR-3 で省略方針か税額目安追加を決定。
- OpenCode→Codex M-13: 状態ボードに fixtures 外の銘柄名 → PR-9 で fixtures 内名称に統一。
- 両レビューの低(L-1〜L-7 等: 折り返し・nowrap・コード欄「—」注記・凡例色・遷移注記・言い回し・直書き各1件)は PR-9 の微修正に回す。
- 良い点(両レビュー): 時点の年付き統一(Codex担当)、381+308 assert の検算、tokens バイト同一、金額の右揃え・符号併記・欠測「—」統一、構成比凡例の対応、投信タイルのゼロ表示、フォーカス土台・44px 領域、担当切り分けの明示。

## 判定の根拠

- 数字の正本: `docs/design/work/phase5-codex/data/fixtures.json`・ `docs/design/work/phase5-codex/data-check.md`・`docs/design/work/phase5-opencode/data-check.md`。OpenCode の配当年合計・12か月・18行・株式3行(+¥99,017)・投信2行(+¥147,282)・最近の配当2行は両 check で一致確認済み。Phase 7 は fixtures 無変更で両 build を再実行し assert 通過(381+308 件)を確認した。
- 用語の正本: `frontend/src/ui/labels.rs`。work 内の表記ゆれは labels を正とする。
- 画像の正本: Phase 7 で撮り直した `docs/design/work/phase5-codex/shots/` 全28枚(PC/スマホ/ダーク+ダイアログ変種)・`docs/design/work/phase5-opencode/shots/` 全30枚・`docs/design/work/phase4-opencode/shots/` 全3枚。撮影条件は各 work の `screens-status.md` に記録。
