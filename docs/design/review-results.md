# 相互レビュー結果(Phase 6 指摘一覧と高の対応状況)

対象: `docs/design/work/phase6-codex/review-of-opencode.md`(Codex→OpenCode。全33枚目視)、`docs/design/work/phase6-opencode/review-of-codex.md`(OpenCode→Codex。全35枚目視)。Phase 7 は本番コードを変えず、HTML 自体の修正は実装フェーズ(PR-1〜9、`implementation-plan.md`)へ繰り越す。ここでは高の指摘を全部載せ、対応状況を「直す(実装PR)/本Phase7で文書に反映/見送り」と理由付きで記録する。未了は未了と書く。

## 高の指摘(9件)と対応状況

### 高-1(Codex→OpenCode-1) 共通トークンのある色・余白が画面CSSに直接指定

- 内容: `site.css:5-13` の `#fff`、`components.css:1` の padding、`settings-opencode.html:48-69` の margin、`finance.html:10-50,60` の余白・文字サイズの直書き。
- 対応: **直す(PR-1・PR-7)**。既存トークンがある値は変数参照に置換し、表せない値は例外理由かデザインシステム側の追加判断を記す。Phase 7 時点では未修正。

### 高-2(Codex→OpenCode-2) Phase 4 OpenCode 文書・見本と Phase 5 共有トークンの対応未整理

- 内容: `cmp` で Codex/Phase5 の `tokens.css` はバイト同一。一方 OpenCode `layout.md:133`・`tokens.css:1-7,59` は現行抜粋で名前(`--color-page` 対 `--page`)・ダーク正数色(`#53d28a` 対 `#7ce8ab`)が異なる。
- 対応: **本Phase7で文書に反映**。`design-system.md` §1 に「Codex 名・値を正」と明記し対応表への参照を残した。HTML・トークン自体の統一は PR-1 で行う(未修正)。

### 高-3(Codex→OpenCode-3) 金額表示が金融表示ルールに反して等幅書体を指定

- 内容: `finance-data.md:18` は `font-mono` 禁止・本文書体+`tabular-nums` とするが、両 `tokens.css:9,34` の `--font-mono`・`.numeric` と OpenCode `site.css` が金額に等幅書体を使う。
- 対応: **直す(PR-1)**。共有 `.numeric` と `site.css` の双方を本文書体+`tabular-nums` に統一し、金額に等幅指定が残らないことを検索で確かめる。Phase 7 時点では未修正。

### 高-4(Codex→OpenCode-4) 危険操作の確認ダイアログにキーボード操作の設計がなく削除確定のフォーカスも不明

- 内容: CSV 6件+アカウント1件の計7件(`csv-opencode.html:92-133`・`account-opencode.html:11-12,51-56`)に `role="dialog"` 等はあるが初期位置・トラップ・Esc・復帰・確定挙動が不明。`receipts-opencode.html:277-291` の `rcpt-d1` は行詳細であり対象外。
- 対応: **直す(PR-8)**。初期位置・Tab範囲・復帰先・Esc/取消・削除ボタンのみ確定を設計に明記し、キーボード確認画像または操作記録を添える。Phase 7 時点では未修正。

### H-1(OpenCode→Codex) PC上ナビの「取引明細」が資産管理を指している(誤遷移)

- 内容: Phase 5 Codex 全9 HTML の PC上ナビ・モバイル下部ナビで「取引明細」のリンク先が `assets.html`。
- 対応: **直す(PR-6)**。正遷移先(OpenCode 側 `receipts-opencode.html` 相当)に差し替えるか、未確定なら `href="#"`+「仮」注記にし `screens-status.md` に残件として書く。Phase 7 時点では未修正。

### H-2(OpenCode→Codex) 投信の評価損益¥0を欠測記号「—」で表示(ゼロと欠測の混同)

- 内容: `assets.html` で投信行の評価損益が「—」(`positive` 付き)。`data-check.md:9` は +¥0(0.00%)と検算し、同一画面の `fund-detail.html` では ¥0 と正しく表示して矛盾する。`MISSING`(「—」)は欠測の表示。
- 対応: **直す(PR-4)**。¥0 を符号なし・色なし(`positive` を外す)で表示し、欠測行との区別注記を足す。Phase 7 時点では未修正。

### H-3(OpenCode→Codex) 集計帯の損失色が増加色と同一+トークン直書き

- 内容: `components.css` 先頭1行 `.summary .gain,.summary .loss{color:#8fe2bc}`。`loss` が増加色と同一で符号識別が色でできない。`#8fe2bc` は `tokens.css` にない直書き。
- 対応: **直す(PR-1)**。損失用の帯上トークン(例 `--on-brand-negative`)を `tokens.css` に追加し light/dark とも 4.5:1 を検算して `foundations.md` の表に追記。`gain`/`loss` の色分けを画像で再確認する。Phase 7 時点では未修正。

### H-4(OpenCode→Codex) ダイアログにセマンティクスが一切ない

- 内容: `dialogs.html` に `role`・`aria-*`・フォーカス管理の指定がゼロ(`grep` で0件)。「閉じる」が2個並ぶ。
- 対応: **直す(PR-8)**。`role="dialog"`・`aria-modal`・`aria-labelledby` を付け、閉じるボタンを1個に一元化し、Esc・フォーカストラップ・復帰先を実装注記として書く。Phase 7 時点では未修正。

### H-5(OpenCode→Codex) スマホの集計帯と月軸ラベルが破綻

- 内容: `home-mobile.png` で集計帯「評価損益 + / ¥1,552,488」が符号と金額の間で改行分断。月別棒の10〜12月ラベルが潰れて判読不能。原因は `components.css` の flex 行化と `.month small{font-size:11px}`(最小12px割れ)。
- 対応: **直す(PR-3)**。金額ブロックに `white-space:nowrap`(符号+金額の不可分化)、月軸は11pxをやめ `--text-xs`(12px)+間引き規則を決めて `foundations.md` に書く。Phase 7 時点では未修正。

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
- 良い点(両レビュー): 時点の年付き統一(Codex担当)、379+308 assert の検算、tokens バイト同一、金額の右揃え・符号併記・欠測「—」統一、構成比凡例の対応、投信タイルのゼロ表示、フォーカス土台・44px 領域、担当切り分けの明示。

## 判定の根拠

- 数字の正本: `docs/design/work/phase5-codex/data/fixtures.json`・ `docs/design/work/phase5-codex/data-check.md`・`docs/design/work/phase5-opencode/data-check.md`。OpenCode の配当年合計・12か月・18行・株式3行(+¥99,017)・投信2行(+¥147,282)・最近の配当2行は両 check で一致確認済み。
- 用語の正本: `frontend/src/ui/labels.rs`。work 内の表記ゆれは labels を正とする。
- 画像の正本: 各レビューの「確認した画像」節(33枚・35枚)。Phase 7 は新規撮影なし。
