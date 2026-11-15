# UX 課題一覧(統合・重要度順)

Phase 2 の2資料(`docs/design/work/phase2-codex/ux-audit-codex.md`、`docs/design/work/phase2-opencode/ux-audit-opencode.md`)を統合し重複を除いた。矛盾は Phase 6 と `frontend/src/ui/labels.rs` を正として解いた。根拠画像は実在確認済みのみを挙げる。H-2・X-1(誤字指摘)は原寸確認で取り下げ済みのため「取り下げ」に記録する。Phase 5 での解消状況は `screen-design-status.md`・`review-results.md` を参照。

## 高

### 1. ホームの配当プログレスバーが振り切れて比較として機能しない

- 画面: ホーム `/`
- 根拠: `docs/design/work/phase1-opencode/shots/home-default-pc-light.png`(スマホ `docs/design/work/phase1-opencode/shots/home-default-mobile-light.png` も同じ)。「受取 +¥3,163,438 / 予想」に対しバーが全幅満杯で 4桁%表示になる。
- 課題: 実績が予想を大幅に上回ると達成率バーの意味を失い、月別内訳との関係も読めない。
- 改善方針: 実績>予想の表示を分ける(上限クリップ+「予想の○倍」文言、または目標達成型に切替)。4桁%は出さない。対象 `frontend/src/features/home/view.rs`。

### 2. 金融データの時点が短い日付だけで示される

- 画面: ホーム・資産管理(横断)
- 根拠: `docs/design/work/phase1-codex/shots/home-pc-light.png`(「データ: 2026/1/1 取り込み」+「価格基準日 10/20」)、`docs/design/work/phase1-codex/shots/asset-all-1440x900-pc-light.png`(集計の「価格基準日 10/20」)。
- 課題: 取込日(年あり)と価格基準日(年なし)が隣り合い、いつの価格か年まで特定できない。
- 改善方針: 取込日と価格基準日を別ラベルで年を含む日付に統一(`PRICE_AS_OF`・`home_import_label`)。注記行に集約。Phase 5 Codex 担当では年付きに統一済み。

### 3. 月別棒グラフが単月一点集中で他月が判読不能

- 画面: ホーム `/`(月別配当)
- 根拠: `docs/design/work/phase1-opencode/shots/home-default-pc-light.png`、`docs/design/work/phase1-opencode/shots/home-default-pc-dark.png`(6月 ¥3,500,000 に対し他月 2〜5%)。Phase 5 でも `docs/design/work/phase5-codex/shots/home-pc.png` で同形。
- 課題: fixtures 由来の単月集中とはいえ、スケール方針(クリップ・注記・表誘導)がないと設計が比較として機能しない。
- 改善方針: 上限クリップ+超過注記、対数軸の不採用理由、月別表への誘導のいずれかを決める。

## 中

### 4. ホームのモバイル上部が他の集計画面と構造が違う

- 画面: ホーム `/`
- 根拠: `docs/design/work/phase1-codex/shots/home-mobile-light.png`、`docs/design/work/phase1-codex/shots/home-mobile-dark.png` 対 `docs/design/work/phase1-codex/shots/receipts-dividend-mobile-dark.png`。
- 改善方針: ホームも共通の集計帯規則に寄せ、評価額・評価損益・配当サマリーの優先順と開閉対象を明確化。

### 5. 長い銘柄・ファンド名が PC で省略され識別が難しい

- 画面: 資産管理 `/assetbalance`
- 根拠: `docs/design/work/phase1-opencode/shots/asset-all-default-pc-light.png`(2件が「…」)、`docs/design/work/phase1-codex/shots/asset-all-1440x900-pc-light.png`。
- 改善方針: 名称列に十分な幅+2行まで表示。全文は行詳細・銘柄詳細で確認。hover のみに頼らない(`docs/design/mobile-ui-rules.md` 禁止)。

### 6. 帯の項目数・順序が資産タブで変わり定点観測しにくい

- 画面: 資産管理 `/assetbalance`
- 根拠: `docs/design/work/phase1-opencode/shots/asset-all-default-pc-light.png`、`docs/design/work/phase1-opencode/shots/asset-search-ui-default-pc-light.png`。
- 改善方針: 項目順序を固定しタブ固有項目は末尾追加。

### 7. 明細の情報密度が高く一覧と詳細の役割が曖昧 / モバイルで税額に手数がかかる

- 画面: 取引明細 `/receipts`
- 根拠: `docs/design/work/phase1-codex/shots/receipts-stock-scrolled-1440x900-pc-light.png`(PC 10列超)、`docs/design/work/phase1-opencode/shots/receipts-dividend-default-mobile-light.png`+`docs/design/work/phase1-opencode/shots/receipts-detail-dialog-default-mobile-light.png`(モバイルは税額が詳細内)。
- 改善方針: 一覧は主要値(税引後)を強調・補助値は列グルーピング整列。モバイル行 2段目に税額目安を追加するか密度と交換で判断。詳細の項目順は PC 列順に揃える。

### 8. 構成比凡例が省略で識別不能・チャートと一覧の対応が弱い

- 画面: 構成比 `/assetbalance/composition`
- 根拠: `docs/design/work/phase1-opencode/shots/asset-composition-default-pc-light.png`、`docs/design/work/phase1-opencode/shots/asset-composition-default-mobile-light.png`。
- 改善方針: 凡例は短縮名+コードを残し、割合・順位一覧と同一順序・同一値表記。「ほか◯件」の導線を明確化。

### 9. CSV の更新状態が色と長い文に依存し一読で分からない

- 画面: CSV管理 `/csv`
- 根拠: `docs/design/work/phase1-codex/shots/csv-asset-1024x768-pc-dark.png`、`docs/design/work/phase1-opencode/shots/csv-default-pc-light.png`、`docs/design/work/phase1-opencode/shots/csv-default-mobile-light.png`(「要更新・15日前(10/31 09:12)に更新」)。
- 改善方針: 状態名と最終取得時点を別項目にし年を含む日付に統一。操作メニューと独立表示。事実のみ書く。

### 10. 設定の生 User-Agent 表示が切れて折り返す

- 画面: 設定 `/settings`(ログイン履歴)
- 根拠: `docs/design/work/phase1-opencode/shots/settings-login-history-default-pc-light.png`、`docs/design/work/phase1-opencode/shots/settings-login-history-default-mobile-light.png`、`docs/design/work/phase1-codex/shots/settings-login-history-scrolled-1440x900-pc-light.png`。
- 改善方針: 短いブラウザ/OS名に正規化し全文は詳細に隠す。Phase 5 OpenCode 担当で対応済み。

### 11. 設定モバイルに目次がなく全体像が掴めない / 管理者導線がない(同根)

- 画面: 設定 `/settings`・管理者 `/admin/logins`(横断)
- 根拠: `docs/design/work/phase1-opencode/shots/settings-default-mobile-light.png`、`docs/design/work/phase1-opencode/shots/admin-logins-default-pc-light.png`。
- 改善方針: 長い画面用の簡易目次パターンを決まりに足す(Phase 5 でスマホ簡易目次を追加)。管理者導線はアカウントメニュー経由を明示。

### 12. 管理者の長いメールが行高をばらつかせる / モバイルが長大

- 画面: 管理者 `/admin/logins`
- 根拠: `docs/design/work/phase1-opencode/shots/admin-logins-default-pc-light.png`(3行折り返し)、`docs/design/work/phase1-opencode/shots/admin-logins-default-mobile-light.png`。
- 改善方針: PC は省略+全文手段、管理者は識別優先で全文表示+折り返しを許容(Phase 5 は全文表示を採用し AD-1 を見送り)。モバイルは 2段短縮+詳細シート。

### 13. チャート系列色がライト/ダークで変わる

- 画面: ホーム・構成比(横断)
- 根拠: `docs/design/work/phase1-opencode/shots/home-default-pc-light.png` 対ダーク、`docs/design/work/phase1-codex/shots/home-mobile-light.png` 対ダーク。
- 改善方針: テーマを越えて同じ資産に同じ色を割当て、色以外のラベルも残す。

### 14. 検索 UI の入口・件数・クリア操作が画面でばらつく

- 画面: 資産・明細・管理者(横断)
- 根拠: `docs/design/work/phase1-opencode/shots/asset-search-ui-default-pc-light.png`、`docs/design/work/phase1-opencode/shots/receipts-search-ui-default-pc-light.png`、`docs/design/work/phase1-codex/shots/admin-logins-empty-1920x1080-pc-light.png`。
- 改善方針: 入口名・条件適用後の件数・クリア操作を共通化(`SearchCount`・チップ)。

### 15. 未ログイン共有 URL の保有セクション扱いが未定義

- 画面: 銘柄検索 `/search`
- 根拠: 画像なし(ログイン済み `docs/design/work/phase1-opencode/shots/search-default-pc-light.png` のみ)。コード上の非表示が根拠。
- 改善方針: 「ログインすると保有情報が見られます」案内を出す。

## 低

- 空・エラー後の次の行動が画面で異なる(資産は説明+CSV導線、明細は取込ボタン、エラーは再読込だが枠が異なる)。根拠 `docs/design/work/phase1-opencode/shots/receipts-empty-default-pc-light.png`・`asset-empty-default-mobile-light.png`・`receipts-error-default-pc-dark.png`。→ `EmptyState`・`ListLoadError` に寄せ説明は1〜2行に揃える。
- 条件パネルが表に重なる。根拠 `docs/design/work/phase1-opencode/shots/receipts-search-ui-default-pc-light.png`。→ backdrop を濃くし非活性が分かるようにする。
- グループ小計と帯集計の関係が一目で分からない。根拠 `docs/design/work/phase1-opencode/shots/receipts-dividend-default-pc-light.png`。→ 小計に「○件の合計」を添える(Phase 5 で対応)。
- 空状態の説明文の長さがばらつく(検索空3行)。根拠 `docs/design/work/phase1-opencode/shots/search-empty-default-pc-light.png`。→ 2行に縮める。
- 構成比入口が2か所に見える。根拠 `docs/design/work/phase1-opencode/shots/asset-all-default-pc-light.png`。→ 件数行の右1か所に寄せるか例外追記。
- スマホ行高のばらつき・ログイン/404の軽微点。根拠 `docs/design/work/phase1-opencode/shots/asset-all-default-mobile-light.png`・`login-default-pc-light.png`・`notfound-default-pc-light.png`。→ 現状維持可。

## 取り下げ

- H-2/X-1(再試行ボタンの誤字疑い): `docs/design/work/phase1-opencode/shots/asset-error-default-pc-light.png` 原寸で「再読み込み」と確認し取り下げ。
