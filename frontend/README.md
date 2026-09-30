# frontend

Leptos (CSR) のフロントエンドです。単独の Cargo パッケージで、Vercel に配信します。

## 前提

- Rust 1.96.0 と `wasm32-unknown-unknown` ターゲット
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

`playwright.leptos.config.ts` は `e2e/migrated/` の主要画面テストと `e2e/` の Leptos テストを実行します。E2E は `LEPTOS_E2E_PORT` でポートを指定できます。複数の worktree で並行実行するときは別々のポートを使ってください。既定では `trunk serve` でソースから配信しますが、`LEPTOS_E2E_DIST_DIR=<dir>` を指定するとビルド済みの dist(CI がアーティファクトで受け渡す trunk build 成果物)を `serve-dist.mjs` で配信します。配信前に `prepare-vercel-dist.mjs` を空の API origin で実行するため、モックに当たらない API 呼び出しは同一オリジンに留まり外部へ出ません。

## ソースの構成

機能ごとにまとめる。親モジュールは `foo.rs`、子は `foo/bar.rs` に置き、`mod.rs` は使わない。

```
src/
  main.rs, app.rs    起動とルーティング
  api/               通信クライアント(client)と DTO(dto)
  session/           ログイン状態・無操作ログアウト・タブ間の同期
  ui/                複数の機能で使う画面部品
  features/
    receipts/        取引明細。model(計算・整形)・store・filter・csv・view(画面)
    asset_balance/   資産管理。model・store・csv_store・format・view(画面)など
    stock_search/    銘柄検索。store と view(画面)
    home, login, not_found, dividend_per_share
  support/           list_search・pagination・csv_flow など機能をまたぐ処理
  testing/           テスト用の補助(cfg(test) のみ)
```

- 依存の向きは view → store → model。model は Leptos に依存しない
- 機能どうしは `features/<機能>.rs`(facade)が re-export する公開部分だけを使う。サブモジュールは非公開にして、可視性で守る
- テストは各モジュールの `<モジュール>/tests.rs` に置く(`#[cfg(test)] mod tests;`)。1つのモジュールに複数ある場合は `tests.rs` から `tests/<名前>.rs` を宣言する
- テストの fixture は `frontend/tests/fixtures/` に置き、`concat!(env!("CARGO_MANIFEST_DIR"), ...)` で読む

## アクセシビリティ

`e2e/migrated/a11y.spec.ts` が関門になる。axe は WCAG 2.0/2.1/2.2 の A/AA で moderate 以上ゼロを必須にする(minor は記録のみ)。対象は主要画面と読み込み中・取得失敗・空・データあり・CSV プレビュー・確認モーダル・フィルター展開で、PC 1280px とスマホ 390px の両方を見る。スキャンの前にその状態になったことを assert する(空表示や失敗表示のつもりで別の状態を検査しない)。

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
| `Button` / `IconButton`(`ui/button.rs`) | すべての `<button>` | 大きさを持つ variant は `Primary(ButtonSize)`・`Secondary(ButtonSize)`・`SecondarySoft(ButtonSize)`・`DangerSolid(ButtonSize)`・`Header(ButtonSize)`。それ以外は DangerGhost・Ghost・Quiet・Prompt・Login・SearchSubmit・Retry・MenuItem・MenuItemDanger・CopyName(size は持たない)。IconButton は Close |
| `LinkButton`(同) | ボタンの見た目の遷移リンク(`<a>`) | ButtonVariant を共有 |
| `Card` / `SectionHeader`(`ui/card.rs`) | カード状の面と節見出し | Panel・Table・Rail・Collapsible・Feature(href で `<a>`)・Summary・Item・Group・Holding・Stat・Sunken・Strip・Hint・Dashed・Step・Tile・GroupLabel など |
| `DisclosureToggle` / `ChevronIcon` / `DisclosureHint`(`ui/disclosure.rs`) | 開閉トリガーと回る山形 | Toolbar・ToolbarMenu・Rail・GroupCard・HeaderFlat・AssetCard・SearchCard。`hint=true` で末尾に「開く/閉じる」 |
| `EmptyState`(`ui/empty_state.rs`) | データが空の画面 | `icon`・`as_h1`・children(次の行動)を持つ |
| `Badge` / `CodeBadge`(`ui/badge.rs`) | 押せない小さなバッジ | Accent・Info・Positive・Neutral・Muted・Warn・File など |
| `Chip` / `Select` / `FieldTrigger` / `OptionButton`(`ui/choice.rs`) | 押せる選択部品とフォーム | Chip は Filter・Segment(aria-pressed)・Pill |
| `TabButton`(`ui/tabs.rs`) | `role="tab"` のタブ | 見た目と roving tabindex は部品が持つ。矢印キー移動は呼び出し側の `on_keydown` が担う(例: ReceiptsTabButton) |
| `Amount`(`ui/amount.rs`) | 金額・率の値 | `tabular-nums` と `[data-negative]` をまとめる。`block=true` で `<p>` として出す |

## デザインの決まり

- 金額は `¥16,574`、マイナスは `-¥16,574`。プラスに符号なし、通常フォントに `tabular-nums`(`font-mono`不可)。欠損は `—`
- 色はマイナスの損益と危険な操作のみ赤系。`data-negative="true"` を付ける値と削除系 UI は `text-negative`(`#b91c1c`)、属性を持たず赤字だけにする箇所(明細の金額・集計値など)は `text-negative-vivid`(Tailwind の赤パレット相当)。税額・配当・利回りは色なし
- 書式は `shared::format` に集約。`features/asset_balance/format.rs` は f64 を Decimal に直して渡すだけで、持つのは円単位の丸めと、Decimal に収まらない金額・率の表示のみ
- 税引後の見出しは、配当が集計 `配当金(税引)`・月の見出し `税引後`・列 `受取額`、国内株式が `実現損益(税引)`・`税引後`、投資信託が `実現損益(税引)`・`税引損益`

## デプロイ

GitHub Actions の `deploy-frontend.yml` が Vercel CLI でビルド・配信します。`VERCEL_TOKEN`、`VERCEL_ORG_ID`、`VERCEL_PROJECT_ID` が必要です。本番デプロイは main の `workflow_dispatch` と `LEPTOS_PRODUCTION_ENABLED=true` で有効になります。
