# frontend

Leptos (CSR) のフロントエンドです。単独の Cargo パッケージで、Vercel に配信します。

## 前提

- Rust（リポジトリルートの `rust-toolchain.toml` を正本とする）と `wasm32-unknown-unknown` ターゲット
- Trunk 0.21.4
- Node.js 22 と npm

## セットアップ

```bash
npm ci
```

Playwright、axe、Tailwind の依存はこのディレクトリの `package.json` と `package-lock.json` で管理します。

## 開発と検証

```bash
trunk serve --port 8081
trunk build --release
cargo fmt --check
cargo clippy --all-targets --target wasm32-unknown-unknown -- -D warnings
cargo clippy --all-targets -- -D warnings
cargo test
npx playwright test --config playwright.leptos.config.ts
npx playwright test --config playwright.vercel.config.ts
```

`playwright.leptos.config.ts` は `e2e/<機能>/` のブラウザテストを、`playwright.vercel.config.ts` は `e2e/deploy/` の配信設定テストを実行します。E2E は `LEPTOS_E2E_PORT` でポートを指定できます。複数の worktree で並行実行するときは別々のポートを使ってください。既定では `trunk serve` でソースから配信しますが、`LEPTOS_E2E_DIST_DIR=<dir>` を指定するとビルド済みの dist(CI がアーティファクトで受け渡す trunk build 成果物)を `serve-dist.mjs` で配信します。配信前に `prepare-vercel-dist.mjs` を空の API origin で実行するため、モックに当たらない API 呼び出しは同一オリジンに留まり外部へ出ません。

### CI と同じ環境で E2E を回す

手元では通るのに CI でだけ落ちる(フォント差など)を再現するため、Ubuntu 24.04 のコンテナで Playwright を回せます。

```bash
bash scripts/e2e-ci-like.sh e2e/receipts/desktop-layout.spec.ts
bash scripts/e2e-ci-like.sh e2e/receipts/desktop-layout.spec.ts --grep "768px"
bash scripts/e2e-ci-like.sh --show-fonts e2e/receipts/desktop-layout.spec.ts
```

ホストで `npm ci` してから `trunk build --release --dist dist-e2e` して、コンテナに `LEPTOS_E2E_DIST_DIR=dist-e2e` で渡します(CI と同じ。コンテナに Rust は入れません)。コンテナはイメージ内の `node_modules` を使い、ホストの `node_modules` には書き込みません。フォント・依存パッケージは `.github/actions/setup-playwright/action.yml` の apt 行から読み取り、Node は `setup-node` の版、`@playwright/test` は `package-lock.json` の版を使います。イメージは内容のハッシュでタグ付けして使い回します(`--rebuild` で作り直し)。引数はそのまま Playwright に渡します。ポートは `LEPTOS_E2E_PORT` がなければ 8140 を使います。

## ソースの構成

機能ごとにまとめる。1つの責務で完結する処理は `foo.rs` だけに置く。責務を分ける子モジュールがあるときだけ `foo.rs` + `foo/` にし、親は型・公開窓口・子の組み立て、子は `foo/bar.rs` に置く。`mod.rs` は使わない。単体テストだけの `foo/tests.rs` も下記の配置規則に従う。

一覧と CSV を持つストアは `store.rs` に一覧取得・セッション世代・キャッシュを置き、CSV の状態操作は `store/csv.rs` に置く。CSV 行の型・レスポンス変換は別責務なので機能直下の `csv.rs` に置く。

```
src/
  main.rs, app.rs    起動とルーティング
  api/               通信クライアント(client)と DTO(dto)
  session/           ログイン状態・無操作ログアウト・タブ間の同期
  ui/                画面部品。site はナビゲーション、state は読み込み・失敗表示
  features/
    receipts/        取引明細。model・store(+store/csv)・filter・csv・view(画面)
    asset_balance/   資産管理。model・store(+store/csv)・format・csv・view(画面)など
    stock_search.rs, stock_search/view.rs  銘柄検索の状態と画面
    home, login, not_found, dividend_per_share
  support/           list_search・pagination・csv_flow など機能をまたぐ処理
  testing/           テスト用の補助(cfg(test) のみ)
```

- 依存の向きは view → store → model。model は Leptos に依存しない
- 機能どうしは `features/<機能>.rs`(facade)が re-export する公開部分だけを使う。サブモジュールは非公開にして、可視性で守る

### 共通処理の正本

| 用途 | 正本 | 呼び出し側に残すもの |
| --- | --- | --- |
| 一覧の全件取得 | `support/pagination.rs` の `fetch_all_pages`。ページ結合・終了・切り詰め判定は `collect_list_pages`、上限は `LIST_PER_PAGE` / `LIST_MAX_PAGES` | `ListEndpoint` の行・集計型と PATH。ホームの集計のみ取得は同じ PATH を使い、`per_page=1`・年指定を維持する |
| CSV の状態・通信 | `support/csv_flow.rs` の `CsvTabState` と preview/upload/delete 関数 | `store/csv.rs` の世代確認・ファイル保持・一覧再取得。エラー文言は `ApiError::message()`、銘柄検索の案内文は `user_message()` |
| 金額・数値の書式 | Decimal は `shared::format`、表示要素は `ui/amount.rs` の `Amount` | `asset_balance/format.rs` の f64 変換・巨大値・非有限値の扱い。配当集計の率は既存の `to_fixed` を使い、Decimal の丸めと混ぜない |
| URL と画面遷移 | `app.rs` の `CurrentPath`・`current_location`・`pathname_of`・`navigate` | `ui/site.rs` は現在パスを読んでナビを表示する。OAuth の外部遷移は `SessionStore::login` / `redirect_to`、通常のリンクは `<a>` と App のクリック処理を使う |

### 画面部品の名前

- `Page`: ルートに対応する画面入口。`<機能名>Page` とし、feature の公開窓口から使う(`ReceiptsPage`・`AssetBalancePage`・`StockSearchPage`)。
- `View`: 引数を受けて表示する部分に役割名が必要なときだけ使う。画面の組み立ては `view.rs`、状態取得は store に置く。Card・Table・Tile など具体名がある部品に View を重ねない。
- `Panel`: サイドパネル・ドロワー・タブパネルのように独立した表示領域(`ReceiptsPanel`・`AssetBalancePanel`・`TabPanel`)。
- `Section`: 画面やパネル内のひとまとまりの内容(`CsvSection`・`DividendSummarySection`)。接尾辞だけの薄い部品や、Page / View / Panel / Section の転送層を作らない。

### テストの配置

- 単体テストは `<モジュール>/tests.rs` に置く(`#[cfg(test)] mod tests;`)。分割するときは `tests/suite.rs` を入口にして親から `#[path = "<モジュール>/tests/suite.rs"] mod tests;` で読み、子は `#[path = "<名前>.rs"] mod <名前>;` で宣言する。同じ階層に `tests.rs` と `tests/` を並べない
- Rust の共有テスト補助・データ生成は `src/testing/<機能>.rs` に置く。データファイルは `frontend/tests/fixtures/` に置き、`concat!(env!("CARGO_MANIFEST_DIR"), ...)` で読む
- E2E は `e2e/<機能>/<内容>.spec.ts` に置く。補助は `e2e/support/`、データは `e2e/fixtures/`。Issue 番号で命名せず、`migrated/` や `e2e-vercel/` は使わない
- 撮影専用 spec は Git 管理外の `.local-e2e/` に置き、通常の Playwright 設定や CI の対象にしない
- `npm run check:test-layout` で配置を検査する。検査スクリプトのテストは `npm run test:test-layout`。どちらも CI で実行する

## アクセシビリティ

`e2e/accessibility/a11y.spec.ts` が関門になる。axe は WCAG 2.0/2.1/2.2 の A/AA で moderate 以上ゼロを必須にする(minor は記録のみ)。対象は主要画面と読み込み中・取得失敗・空・データあり・CSV プレビュー・確認モーダル・フィルター展開で、PC 1280px とスマホ 390px の両方を見る。スキャンの前にその状態になったことを assert する(空表示や失敗表示のつもりで別の状態を検査しない)。

- タップ領域: PC 幅は 24px 以上、スマホの操作要素は 44px 以上(`max-sm:min-h-11`)
- 主要な操作は Tab だけで到達でき、Enter/Space で操作できること(`.focus()` での到達は検証にならない)
- フォーカスは見えること。ファイル選択は入力(`sr-only`)を表示ラベル内に入れ、ラベルの `focus-within` で枠を出す
- `aria-*` の真偽値は `"true"`/`"false"` 文字列で出す(bool のまま置かない)
- タブの `aria-controls` は読み込み中も実在する要素を指す(非選択タブのパネルは `hidden` で出す)
- 読み込み中・失敗・CSV 結果は `role="status"`/`role="alert"` と `aria-live` で伝え、操作中は `aria-busy` を付ける
- `prefers-reduced-motion` ではアニメーションを止める(`style/input.css` のメディアクエリ)
- マイナスは `-` の符号と赤の両方で表す(色だけにしない)。文字色は AA コントラストを満たすものだけ使う
- 直せない違反は除外リストに Issue 番号付きで載せる(今はなし)

## CSS

`style/input.css` を Trunk の pre_build フックで `style/output.css` に生成し、`index.html` から読み込みます。生成物は Git 管理外です。

### 色・値はトークンとコンポーネントで再利用する

- 色・影・角丸・幅は `style/input.css` の `@theme` に意味で名前を付けたトークンで使う: 面 `surface`・文字 `text`・プラス `positive`/`gain`・マイナス `negative`・アクセント `accent`・リンク `info`、影 `shadow-elevation-1..3`、角丸 `rounded-panel`/`rounded-note`/`rounded-field` など
- 生のパレット(`slate-500`・`amber-50`・`red-700` など色名+番号)と白・黒の直書き(`bg-white`・`text-black` など)、任意値(`shadow-[...]`・`text-[10px]`・`min-h-[44px]` など `[...]` 指定)を `src/` のクラスに直書きしない。同じ見た目が必要ならトークンを追加するか、`ui/` か `input.css` の `@layer components` の部品(`.panel-card`・`.empty-state`・`.receipts-tab-list`・`.security-code-link` など)を使う
- `@apply` は `@layer base` と、コンポーネント化できない最小限にとどめる
- 1つのクラス内で同じプロパティを二度指定しない(打ち消し合う指定は効いている方だけ残す)
- `bash scripts/check-css-tokens.sh`(`npm run check:css`)が違反を検出し、CI(`frontend.yml`)でも実行する。やむを得ない例外はスクリプトの許可リスト(`ALLOWED_SRC`・`ALLOWED_CSS`)に理由付きで列挙する
- 同じ検査で `scripts/check-ui-primitives.sh` も動き、`features/` での `<button>`/`<select>` の直書き、コンポーネントクラスの直書き、カードに相当するユーティリティの組み合わせ(`rounded-*`+`border`+`bg-surface`)、`aria-expanded` を持つ独自の開閉要素を落とす。違反が出たら下の画面部品を使う

### 画面の基本部品(`src/ui/`)

`features/` のマークアップは基本部品で組む。種類は enum の props で選び、違う見た目が必要なら variant を足す(クラスを直書きしない)。レイアウトや余白など見た目に関係しない調整だけ `class` で追加する。

| 部品 | 役割 | 主な variant |
| --- | --- | --- |
| `Button` / `IconButton`(`ui/button.rs`) | すべての `<button>` | 大きさを持つ variant は `Primary(ButtonSize)`・`Secondary(ButtonSize)`・`SecondarySoft(ButtonSize)`・`DangerSolid(ButtonSize)`・`Header(ButtonSize)`。それ以外は DangerGhost・Ghost・Quiet・Prompt・Login・SearchSubmit・Retry・MenuItem・MenuItemDanger・CopyName(size は持たない)。IconButton はモーダルの閉じるボタン(variant なし) |
| `LinkButton`(同) | ボタンの見た目の遷移リンク(`<a>`) | ButtonVariant を共有 |
| `Card`(`ui/card.rs`) | カード状の面 | Panel・Login・Table・Collapsible・Feature(href で `<a>`)・Shell・Soft・Item・Group・Holding・Sunken・Strip・DashedCompact・Tile・GroupLabel |
| `DisclosureToggle` / `ChevronIcon` / `DisclosureHint`(`ui/disclosure.rs`) | 開閉トリガーと回る山形 | Rail・GroupCard・HeaderFlat・AssetCard・SearchCard。`hint=true` で末尾に「開く/閉じる」 |
| `EmptyState`(`ui/empty_state.rs`) | データが空の画面 | `icon`・`as_h1`・children(次の行動)を持つ |
| `Badge` / `CodeBadge`(`ui/badge.rs`) | 押せない小さなバッジ | Accent・AccentFlat・Info・Positive |
| `Chip` / `Select` / `FieldTrigger` / `OptionButton`(`ui/choice.rs`) | 押せる選択部品とフォーム | Chip は Filter・Segment(aria-pressed)・Pill |
| `TabButton`(`ui/tabs.rs`) | `role="tab"` のタブ | 見た目と roving tabindex は部品が持つ。矢印キー移動は呼び出し側の `on_keydown` が担う(例: ReceiptsTabButton) |
| `Amount`(`ui/amount.rs`) | 金額・率の値 | `tabular-nums` と `[data-negative]` をまとめる。`block=true` で `<p>` として出す |
| `Alert` / `Spinner` / `Loading` / `LoadingStrip` / `Skeleton` / `ListLoadError` / `ListSkeleton`(`ui/state.rs`) | 読み込み・失敗の状態表示 | Alert は Warning・Danger、SpinnerSize は Sm・Lg、ListSkeletonVariant は Cards・Table。ListLoadError は再読み込み操作を持つ |
| `CollapsibleSearchCard`(`ui/collapsible_search_card.rs`) | 検索欄の開閉と解除操作 | SearchCardLayout は Card・Toolbar |
| `CsvSection`(`ui/csv_section.rs`) / `CsvActionRail`(`ui/csv_rail.rs`) | CSV の選択・保存・結果・削除欄 | CsvSource が状態と操作を渡し、配線とマークアップを共有する |
| `CsvPreviewBanner` / `CsvPreviewNotice`(`ui/csv_preview.rs`) | 未保存のプレビュー・行エラーの通知 | 保存操作と有効行数に合わせた通知。variant なし |
| `ConfirmDeleteModal`(`ui/confirm_modal.rs`) | 削除確認ダイアログ | フォーカスの閉じ込め、処理中の閉じ操作無効化。item_count は任意 |
| `WorkspaceShell`(`ui/workspace_shell.rs`) | サイドパネルと本文の配置 | collapsible でドロワーと常時表示を切り替える |
| `SecurityCodeLink` / `CopyableInstrumentName`(`ui/security_link.rs`) | 銘柄コードの遷移と銘柄名コピー | SecurityLinkVariant は Code・Details |
| `SiteHeader` / `SiteFooter`(`ui/site.rs`) | サイト共通のナビゲーション・ユーザーメニュー・フッター | URL と SPA 遷移は app.rs が持つ |

## デザインの決まり

- 金額は `¥16,574`、マイナスは `-¥16,574`。プラスに符号なし、通常フォントに `tabular-nums`(`font-mono`不可)。欠損は `—`
- 色はマイナスの損益と危険な操作のみ赤系。`data-negative="true"` を付ける値と削除系 UI は `text-negative`(`#b91c1c`)、属性を持たず赤字だけにする箇所(明細の金額・集計値など)は `text-negative-vivid`(Tailwind の赤パレット相当)。税額・配当・利回りは色なし
- 書式は `shared::format` に集約。`features/asset_balance/format.rs` は f64 を Decimal に直して渡すだけで、持つのは円単位の丸めと、Decimal に収まらない金額・率の表示のみ
- 税引後の表現は `税引後` に統一する。税引前の実現損益は `実現損益`、配当の税引前は `配当金` と表記する

### 用語対応表

ユーザーに見える表示文字列の統一先。コード内の識別子・型名は変えない。

| 概念 | 統一する表記 | 旧表記(置き換え済み) |
| --- | --- | --- |
| 税引後の金額・損益 | `税引後` | `配当金(税引)` `実現損益(税引)` `税引損益` `受取額` `受取金額` |
| 今年の税引後配当 | `今年の税引後配当金` | `今年の配当金(税引)` |
| 税引前の実現損益(取引明細) | `実現損益` | `損益` |
| 税引前の配当(取引明細) | `配当金` | — |
| 節見出し(集計) | `集計情報` | `資産サマリー` |
| 配当利回り(ポートフォリオ) | `配当利回り（年間）` | `配当利回り` |

- `損益` は `評価損益`(ホーム・資産管理の未実現損益)などの複合語でのみ使う。取引明細の実現損益は `実現損益` と表記する
- 配当タブの `税引後` は受取額であり損益ではないため、負数でも赤色にしない

## デプロイ

GitHub Actions の `deploy-frontend.yml` が Vercel CLI で配信します。`VERCEL_TOKEN`、`VERCEL_ORG_ID`、`VERCEL_PROJECT_ID` が必要です。

- main への frontend/shared 変更 push 後と `frontend.yml` の `workflow_dispatch`（main のみ）は、Frontend CI 成功後に `vercel-dist` 成果物を再利用してデプロイします。本番への反映には `LEPTOS_PRODUCTION_ENABLED=true` が必要です。
- `deploy-frontend.yml` 自体の `workflow_dispatch` は main 限定の独立ビルド経路です。Frontend CI の成功待ちや成果物の再利用は行いません。
- PR では `deploy-frontend.yml` が `pull_request` で起動し、独立してビルド・preview 配信し、URL を PR コメントに投稿します（投稿者が OWNER/MEMBER/COLLABORATOR の場合のみ）。Frontend CI の成功を待たず、CI が失敗しても preview が配信される場合があります。

現行のセキュリティルールを守って PR preview に CI 成果物を再利用するには、権限のある後段ワークフローと run・PR・成果物の照合が必要になるため、二重ビルドのまま維持します（[#1185](https://github.com/zaichu/shoken-webapp/issues/1185)）。Frontend CI の PR ジョブへデプロイ用 secrets を追加する変更や、既存の権限・セキュリティルールの変更は行いません。
