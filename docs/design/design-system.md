# デザインシステム(概要と参照)

詳細は work の該当ファイルが正本。本書は tokens・部品・金融表示・レイアウトの要点と対応表のみを持ち、重複記述しない。数値は `docs/design/work/phase5-codex/data/fixtures.json` と両 `data-check.md` に一致させる(共通値: 評価額 ¥28,593,148=国内株式 ¥9,149,807+投資信託 ¥19,443,341、取得総額 ¥27,040,660、評価損益 +¥1,552,488、配当 税引前 ¥3,969,907−税額 ¥806,469=税引後 ¥3,163,438、株式税引後 +¥99,017、投信税引後 +¥147,282、明細18/3/2行)。文言は `frontend/src/ui/labels.rs` を正とする。

## 1. Tokens

- 正本: `docs/design/work/phase4-codex/tokens.css`(Phase 5 両担当でバイト同一。`cmp` 終了コード0)。Phase 4 OpenCode の `docs/design/work/phase4-opencode/tokens.css` は Codex 作成前の現行抜粋であり、名前(`--color-page` 対 `--page` 等)とダーク正数色(`#53d28a` 対 `#7ce8ab`)が異なるため、新規実装は Codex 側の名前・値を正とする。差分整理は `review-results.md` の高-2 に記録。
- 対応表: `docs/design/work/phase4-codex/foundations.md`(現行 `frontend/style/input.css:5-154,220-300` との対応表あり)。
- 未解決(実装時に決める): 帯上の損失色トークンなし(現行案は増加色と同一・直書き `#8fe2bc`。高-3)、系列色のテーマ不変対応(M-1)、11px 等の直書き残り(M-7)。いずれも `review-results.md` に繰越。
- 見本: `docs/design/work/phase4-codex/html/foundations.html`(shots 8枚: `docs/design/work/phase4-codex/shots/foundations-pc-light.png` 他)。

## 2. 部品

- 正本: `docs/design/work/phase4-codex/components.md`(既存対応付き)。既存部品との対応は `implementation-plan.md` に再掲。
- 主要部品と既存名: PC上ナビ・スマホ下部ナビ(`PageFrame`・`BottomNav`)、アカウントメニュー(`Avatar`・`Menu`)、ボタン(`Button`・`ButtonVariant`)、入力・選択(`Select`・`SearchTextInput`・`SearchBox`)、タブ(`TabButton`・`UnderlineTabs`・`Segmented`)、検索入口(`SearchEntry`・`SearchDialog`・`SearchCount`)、要点カード(`SummaryBand`・`SummaryItem`・`Card`)、一覧(`ListRow*`・`DetailDialog`)、PCテーブル(`.asset-table`・`.receipt-table`)、スマホ行(`ListRow*`・`ListGroupHeader`)、ダイアログ/シート(`DialogShell`・`DetailDialog`・`ConfirmDeleteModal`)、空(`EmptyState`)・読込(`ListSkeleton`)・失敗(`ListLoadError`・`Alert`)。
- ルール: 44px タップ領域、3px フォーカスリング、hover 前提禁止、増減は `AmountKind::Delta` に寄せ色のみにしない。
- 見本: `docs/design/work/phase4-codex/html/components.html`(shots 8枚: `docs/design/work/phase4-codex/shots/components-pc-light.png` 他)。

## 3. 金融データ表示規則

- 正本: `docs/design/work/phase4-opencode/finance-data.md`。見本 `docs/design/work/phase4-opencode/finance.html`(shots 3枚: `docs/design/work/phase4-opencode/shots/finance-pc-light.png` 他)。
- 要点: 3桁区切り・右揃え・`tabular-nums`(本文書体のまま。`font-mono` は使わない。`frontend/README.md:166`)、整数円・列内小数桁揃え・率は2桁、単位は `labels.rs` 関数、省略形(`¥28.6M`)禁止・全文表示、損益は符号+色・残高は符号なし・税額は出ていくお金として `-¥`+赤、0は `¥0`(符号・色なし)・欠測は `—`(`MISSING`)で混ぜない、時点は取込日と価格基準日を別ラベル年付き、合計と内訳の一致(不一致は不具合)、月次は棒・構成は円+凡例(金額+割合)・6系列以降は「ほか◯件」、状態名と時点を分離。
- 注意: Phase 4 OpenCode の数値例(評価額 ¥3,345,000、配当 ¥300,000)は本書内の共通例であり Phase 5 fixtures(¥28,593,148、¥3,163,438)とは別データ。混同しないこと(Phase 6 低-7相当)。Phase 5 の実装値は fixtures を正とする。
- 未解決: `font-mono` 指定が共有 CSS(`tokens.css:9,34`)と OpenCode `site.css` に残り規則と矛盾(高-3相当)。実装時に本文書体+`tabular-nums` に統一する。

## 4. レイアウト規則

- 正本: `docs/design/work/phase4-opencode/layout.md`。型の正本は `docs/design/mobile-ui-rules.md`・`docs/ui-design.md`。
- 要点: 画面の型は上バー→戻る→タブ→集計帯→注記→検索入口→一覧/詳細/フォーム→フッター→下部ナビ(スマホのみ)の順で変えない。切替点は 1024px(`lg`)のみ。PC(1280〜)の資産・明細に右詳細パネル。スマホは1カラム・帯は全幅・表は出さない・タブは下線+スワイプ。画面ごとの配置(ホーム・資産・明細・検索・CSV・設定・管理者)は `layout.md` §3 が正本。
- 未解決: スマホ集計帯の符号分断・月軸11px潰れ(H-5)、並べ替え UI の重なり(M-6)、条件チップ・件数なし(M-10)。実装時に直す。

## 5. 既知の矛盾と正の扱い

- 用語: work 内の表記ゆれは `labels.rs` を正とする(例: 「年間配当」→`ANNUAL_EXPECTED_DIVIDEND`「年間予想配当(税引前)」、「年間配当(1株)」→`PER_SHARE_DIVIDEND`「1株配当」、投信「価格基準日」→`BASE_PRICE_AS_OF`「基準価額の基準日」、空・エラー文言→`ASSET_EMPTY_TITLE`・`API_DATA_LOAD_FAILED` 系)。一覧は `review-results.md` M-2 に記録。
- アクセシビリティ: ダイアログの `role`・`aria`・フォーカス・Esc は Phase 5 時点で Codex 側なし・OpenCode 側は危険操作のみ部分的(高-4・H-4)。実装時に一元化する。
