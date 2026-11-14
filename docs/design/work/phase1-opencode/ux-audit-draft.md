# Phase 1 (OpenCode): UX 監査ドラフト

## 0. 調査方法と限界

- 対象: `frontend/src` の全ルート・feature・`ui/` 部品。ルーティングは `frontend/src/app.rs:84-106`。
- 設計文書: `docs/design/mobile-ui-rules.md`, `docs/ui-design.md`, `frontend/README.md` を突き合わせた。
- 画像: `shots/` に 168 枚 (PC 1280×800 / モバイル 390×844、ライト/ダーク) を保存したが、本モデルは画像入力非対応のため目視確認は未実施。構造・状態の存在はコードと E2E (axe) で確認した。画像依存の指摘は「未確認」として扱う。
- アクセシビリティの数値 (コントラスト・タップ領域) は `frontend/e2e/accessibility/a11y.spec.ts` の結果を根拠にした。

重要度: 高 (利用阻害・データ誤認の可能性) / 中 (操作負荷・一貫性) / 低 (改善余地)。

---

## 1. 画面をまたぐ課題 (横断)

| # | 課題 | 重要度 | 根拠 | 改善方針 |
|---|---|---|---|---|
| C-1 | 設計文書の不一致: `docs/ui-design.md` はボタンを「主/副/無効の3種類」とするが、`frontend/README.md` と `mobile-ui-rules.md` は 20 種以上の `ButtonVariant` を定義している | 中 | `ui-design.md:87-89`, `README.md:143` | `ui-design.md` を現行部品表に合わせて更新するか、mobile-rules を優先と明記する |
| C-2 | 設計文書の不一致: `ui-design.md` は「ボタンの言い換え・見ればわかる案内は書かない」とするが、CSV 画面は状態説明文を多く出す (ui-design の例外に未記載) | 低 | `ui-design.md:14`, `docs/ui-design.md:136-138` | CSV の状態文言を mobile-rules §CSV に集約し、ui-design から参照する |
| C-3 | 検索 UI の配置が画面ごとに異なる (資産管理=本文内入口、取引明細=モバイルは下部操作帯、銘柄検索=下部操作帯)。mobile-rules で例外列挙されているが、ユーザー視点では学習コスト | 中 | `mobile-ui-rules.md:45,98`, `ui-design.md:79-80` | 現状維持 (例外は根拠あり)。Phase 2 で操作導線の統一を検討 |
| C-4 | PC とモバイルで情報アクセス経路が異なる (取引明細: PC は全列表示、モバイルは `DetailDialog`)。データ自体は同じだが操作回数が違う | 中 | `receipts/view/cards.rs:479`, `ui-design.md:154` | PC でも行クリックで詳細ダイアログを開けるようにするか、モバイルでも主要列を増やす (密度とのトレードオフ) |
| C-5 | 設定画面は PC のみ左 TOC (`hidden lg:block`)。モバイルは全セクションを縦に並べるため長い | 中 | `settings.rs:170` | モバイル用の折りたたみ TOC またはセクションジャンプを追加 |
| C-6 | 構成比への入口が PC では上バー (`page-secondary-link-sm max-lg:hidden`) と件数行の両方に存在する可能性 | 中 | `asset_balance/view.rs:341` | PC でも入口を件数行の 1 か所に統一する (mobile-rules §構成比への入口) |
| C-7 | ダークテーマのコントラストは axe で検証されているが、画像の目視未確認 | 低 | `a11y.spec.ts` ダークセクション | Phase 2 で視覚レビュー |
| C-8 | `admin/logins` は下部ナビに項目がなく、直接 URL のみ。一般ユーザーには不要だが、管理者導線が不明 | 低 | `app.rs:71-73` (shows_bottom_nav は true)、`site.rs` BottomNav | 管理者メニューに追加するか、設定画面からリンクする |
| C-9 | ホームのチャート色は 5 系列固定 + その他同色。口座/商品が 6 種以上で色が衝突 | 低 | `home/view.rs` CHART_COLORS / account_chart_index | 6 系列目以降も個別色を割り当てるか、凡例のテキストで区別を強化 |

---

## 2. 画面別

### 2.1 ホーム `/`

| 観点 | 指摘 | 重要度 | 根拠 |
|---|---|---|---|
| 情報の見つけやすさ | ログイン済みは評価額・評価損益・今年の税引後配当・構成比チャートを表示。未ログインは `LoginPrompt` のみ | — (仕様通り) | `home/view.rs` L120-128, `ui-design.md:147` |
| 情報密度と可読性 | `full_bleed` でチャートを広く表示。モバイルでは密度が高い可能性 | 低 | `home/view.rs` PageFrame full_bleed |
| データ時点 | 直近 CSV 取り込み日を `format_latest_import_date` で表示 (データがある場合のみ) | — | `home/view.rs` import_label |
| レスポンシブ | チャートは SVG/クラスで描画。モバイルでの操作は未確認 | 未確認 | — |
| 実装 | store の再利用 (`use_home_overview`) で再訪時に値を保持 | — | `home/store.rs` |

問題なし (根拠付き): 未ログイン時の案内は `LoginPrompt` 共通部品 (`empty_state` 系) を使用 (`home/view.rs` L127)。

### 2.2 銘柄検索 `/search`

| 観点 | 指摘 | 重要度 | 根拠 |
|---|---|---|---|
| 操作の分かりやすさ | 検索欄が下部操作帯 (mobile) / PC は本文上。例外として許容 | — | `mobile-ui-rules.md:98` |
| 空状態 | `EmptyState` (Search アイコン) + 説明 | — | `stock_search/view.rs:380-392` |
| 情報密度 | 保有情報は `HoldingTiles` で 2 列グリッド | 低 | `ui/holding_tiles.rs` |
| データ精度 | 1 株配当・利回りは `format_unit_price` / `unit_price_decimals` で桁揃え | — | `ui-design.md:37-39` |
| 実装 | `?code=` で保有銘柄詳細を兼ねるため、ブックマーク可能 | — | `holding_detail_page.rs:12-13` |

課題: 検索結果の保有セクションが資産データに依存し、未ログインでは非表示になるため、共有 URL を未ログインで開くと情報が欠ける。重要度: 中。根拠: `stock_search/view.rs` HoldingSection。改善: 未ログイン時は「ログインすると保有情報が見られます」を表示。

### 2.3 資産管理 `/assetbalance`

| 観点 | 指摘 | 重要度 | 根拠 |
|---|---|---|---|
| 情報の見つけやすさ | タブ (すべて/国内株式/投資信託) + 帯 + 注記 + 検索入口の順序は mobile-rules に準拠 | — | `mobile-ui-rules.md:20-33`, `asset_balance/view.rs` |
| 情報密度と可読性 | 行は `ListRow` 共通。配当・利回りで並べ替え時にその値を行に追加表示 | 低 | `ui-design.md:95` |
| 画面遷移 | 行クリックで国内株式は銘柄検索へ、投資信託は詳細ページへ。挙動が種別で異なる | 中 | `portfolio.rs:236,243`, `holding_detail_page.rs` |
| レスポンシブ | PC は表、モバイルはカード行。a11y で両幅検証 | — | `a11y.spec.ts` 資産管理 |
| アクセシビリティ | タップ領域 44px 以上 (モバイル) を検証 | — | `a11y.spec.ts` assertTargetSize |
| 実装 | `text_only_search` でタブ切替時に条件を引き継ぐ | — | `asset_balance/view.rs` Effect |

課題: 国内株式の行を押すと銘柄検索 (`/search?code=`) に遷移し、保有詳細ではなく銘柄情報が主になる。ユーザーが「保有の詳細」を期待すると混乱。重要度: 中。根拠: `holding_detail_page.rs:12-13`, `holdings.rs:152-154`。改善: 遷移先で保有セクションを最上部に表示するか、詳細ページを残す。

### 2.4 構成比 `/assetbalance/composition`

| 観点 | 指摘 | 重要度 | 根拠 |
|---|---|---|---|
| 情報の見つけやすさ | 取得額が大きい順の一覧 + 単色棒。合計 0 は「算出不可」 | — | `ui-design.md:102-105` |
| 視覚的一貫性 | 上バーの構成比リンクは PC のみ。モバイルは件数行の入口 | 中 | `asset_balance/view.rs:341` (C-6) |
| 実装 | 保存済み保有全体を対象 | — | `ui-design.md:102` |

### 2.5 投資信託詳細 `/assetbalance/funds/:id`

| 観点 | 指摘 | 重要度 | 根拠 |
|---|---|---|---|
| 情報密度 | `HoldingTiles` で評価額・評価損益等を 2 列幅タイル | — | `mobile-ui-rules.md:92` |
| データ精度 | 基準日 `price_as_of`、価格ソース `price_source`、`nav_is_latest` を表示 | — | `FUND_ROW` (screens.mjs) |
| 画面遷移 | 「銘柄情報を見る」で `/search?code=` へ | — | `ui-design.md:109` |

課題: 予想 NAV 日付 (`expected_nav_date`) が null の場合の表示 (「更新待ち」) が `PENDING_UPDATE` で統一されているか未確認。重要度: 低。根拠: `ui-design.md:31`。

### 2.6 取引明細 `/receipts`

| 観点 | 指摘 | 重要度 | 根拠 |
|---|---|---|---|
| 情報の見つけやすさ | タブ + 帯 + 注記 + 検索 + 条件・件数行の順序は mobile-rules に準拠 | — | `mobile-ui-rules.md:20-33`, `ui-design.md:115` |
| 操作の分かりやすさ | 期間は 5 種 (すべて/年/年月/日/範囲)。口座・商品は即反映 | — | `ui-design.md:118` |
| 情報密度と可読性 | 月グループヘッダーに件数と小計。行は日付・銘柄・口座・主金額 | — | `ui-design.md:119,156` |
| 画面遷移 | PC は全列表示、モバイルは `DetailDialog` で残りを表示 (C-4) | 中 | `receipts/view/cards.rs:479` |
| レスポンシブ | 検索欄: PC は本文上、モバイルは下部操作帯 | — | `ui-design.md:115` |
| アクセシビリティ | 条件パネルは Esc/外側クリックで閉じ、フォーカスをトリガーに戻す | — | `ui-design.md:128` |
| 実装 | 集計は検索結果と同じ対象 | — | `ui-design.md:120` |

課題: 条件タグの「×」個別解除と「すべてクリア」が条件行の右端に集約。モバイルではタグが折り返して高さが変わる可能性 (画像未確認)。重要度: 低。根拠: `ui-design.md:116`。

### 2.7 CSV 管理 `/csv`

| 観点 | 指摘 | 重要度 | 根拠 |
|---|---|---|---|
| 情報の見つけやすさ | 5 種類を 1 画面の表で管理。行の「⋯」に操作集約 | — | `ui-design.md:132-134` |
| 操作の分かりやすさ | 取得中もボタン位置固定、文言のみ変更 | — | `ui-design.md:139` |
| 情報密度 | スマホは 2 段表示。理由は折り返し全文表示 | — | `ui-design.md:137-138` |
| データ精度 | 状態は事実のみ (「最新です」とは書かない) | — | `ui-design.md:136` |
| アクセシビリティ | 削除確認モーダルは表の外 (`Portal`) | — | `ui-design.md:142` |
| 実装 | 行は `KindFlag` で開閉状態を保持 | — | `ui-design.md:141` |

課題: 削除確認・詳細モーダルの画像未撮影 (C-? )。重要度: 低。根拠: `shots/INDEX.md` 未撮影欄。改善: 撮影スクリプトに操作を追加。

### 2.8 設定 `/settings`

| 観点 | 指摘 | 重要度 | 根拠 |
|---|---|---|---|
| 情報の見つけやすさ | PC は左 TOC、モバイルは TOC なし (C-5) | 中 | `settings.rs:170` |
| 操作の分かりやすさ | セクションは 7 つ。モバイルでは長い縦スクロール | 中 | `settings.rs:189-203` |
| アクセシビリティ | 登録端末・ログイン履歴は PC テーブル / モバイルリストに分離 | — | `login_history.rs:195-230` |
| データ精度 | ログイン履歴は国・地域・ブラウザ・OS を表示。長い値の折り返しは未確認 | 未確認 | `loginHistory()` モック |
| 実装 | フラグメント直行 (`/settings#data`) をサポート | — | `app.rs` フラグメント処理 |

### 2.9 管理者ログイン一覧 `/admin/logins`

| 観点 | 指摘 | 重要度 | 根拠 |
|---|---|---|---|
| 情報の見つけやすさ | テーブル (PC) / リスト (モバイル)。検索はメール・場所 | — | `admin_logins.rs` |
| 操作の分かりやすさ | ソート列 4 つ。モバイルはシートで検索・並べ替え | — | `admin_logins.rs` AdminDialogFields |
| レスポンシブ | 64 件のページング (もっと見る) | 低 | `can_load_more` |
| アクセシビリティ | axe 対象外? a11y spec は admin 画面を含まない | 中 | `a11y.spec.ts` (admin なし) | 

課題: 管理者画面が axe スキャンの対象に含まれていないため、アクセシビリティ回帰を検知できない。重要度: 中。根拠: `frontend/e2e/accessibility/a11y.spec.ts` (admin ルートなし)。改善: admin 画面を a11y spec に追加。

### 2.10 ログイン `/login`

| 観点 | 指摘 | 重要度 | 根拠 |
|---|---|---|---|
| 情報の見つけやすさ | `LoginPrompt` のみ。Google ログインボタン | — | `login.rs:25-30`, `ui/login_prompt.rs` |
| アクセシビリティ | axe 対象 (ログイン) | — | `a11y.spec.ts` ログイン |
| 実装 | ログイン済みなら `/` へリダイレクト | — | `app.rs:378` |

### 2.11 404 `/404` および不明パス

| 観点 | 指摘 | 重要度 | 根拠 |
|---|---|---|---|
| 情報の見つけやすさ | EmptyState + トップへ戻る + 関連リンク 3 つ | — | `not_found.rs:14-35` |
| レスポンシブ | BottomNav なし (app.rs shows_bottom_nav false) | — | `app.rs:71-73` |

---

## 3. 決まりと実画面のずれ (mobile-ui-rules / ui-design)

| # | 決まり | 実装/画面 | ずれ | 根拠 |
|---|---|---|---|---|
| D-1 | ボタンは 3 種類 (ui-design) | 20+ variants (README) | 文書間の矛盾 (C-1) | `ui-design.md:87`, `README.md:143` |
| D-2 | 構成比入口は件数行の右 1 か所 | PC では上バーにもリンク | PC で重複 (C-6) | `asset_balance/view.rs:341` |
| D-3 | スマホの主要操作は親指が届く下部 | 資産管理の検索入口は本文内 (上部) | 例外列挙あり。許容 | `mobile-ui-rules.md:45` |
| D-4 | アイコンだけのボタン禁止 | `SearchClear` 等は文字ラベル? | 未確認 (画像) | `mobile-ui-rules.md:15` |
| D-5 | 用語は `labels.rs` 正本 | コメント・テスト以外に旧語なし | 一致 | `labels/tests.rs` |
| D-6 | マイナスは符号と赤の両方 | `data-negative` + AmountKind Delta | 一致 (a11y 検証) | `a11y.spec.ts` マイナステスト |

---

## 4. 未確認事項

- 画像の視覚的確認 (レイアウト崩れ、折り返し、余白) は未実施。
- コントラストの実測値 (axe は moderate 以上ゼロを確認済み、minor は記録のみ)。
- 320px 幅での横スクロールは a11y spec で検証済み。
- 実機ブラウザ・フォント差は未確認。
- admin 画面の axe スキャンは未実施 (C? D?).
- CSV モーダル・詳細モーダルの撮影は未実施 (`shots/INDEX.md`)。
