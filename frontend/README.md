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

`playwright.leptos.config.ts` は `e2e/migrated/` の主要画面テストと `e2e/` の Leptos テストを実行します。E2E は `LEPTOS_E2E_PORT` でポートを指定できます。複数の worktree で並行実行するときは別々のポートを使ってください。

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
- 機能どうしは `features/<機能>` の公開部分だけを使い、view の中身には触れない
- テストは各モジュールの `<モジュール>/tests.rs` に置く(`#[cfg(test)] mod tests;`)。1つのモジュールに複数ある場合は `tests.rs` から `tests/<名前>.rs` を宣言する
- テストの fixture は `frontend/tests/fixtures/` に置き、`concat!(env!("CARGO_MANIFEST_DIR"), ...)` で読む

## アクセシビリティ

`e2e/migrated/a11y.spec.ts` が関門になる。axe は WCAG 2.0/2.1/2.2 の A/AA で moderate 以上ゼロを必須にする(minor は記録のみ)。対象は主要画面と読み込み中・取得失敗・空・データあり・CSV プレビュー・確認モーダル・フィルター展開で、PC 1280px とスマホ 390px の両方を見る。スキャンの前にその状態になったことを assert する(空表示や失敗表示のつもりで別の状態を検査しない)。

- タップ領域: PC 幅は 24px 以上、スマホの操作要素は 44px 以上(`max-sm:min-h-[44px]`)
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

## デザインの決まり

- 金額は `¥16,574`、マイナスは `-¥16,574`。プラスに符号なし、通常フォントに `tabular-nums`(`font-mono`不可)。欠損は `—`
- 色はマイナスの損益のみ `text-red-700`(暗背景は `red-300`)。税額・配当・利回りは色なし
- 書式は `shared::format` に集約。`features/asset_balance/format.rs` は f64 を Decimal に直して渡すだけで、持つのは円単位の丸めと、Decimal に収まらない金額・率の表示のみ
- 税引後の見出しは、配当が集計 `配当金(税引)`・月の見出し `税引後`・列 `受取額`、国内株式が `実現損益(税引)`・`税引後`、投資信託が `実現損益(税引)`・`税引損益`

## デプロイ

GitHub Actions の `deploy-frontend.yml` が Vercel CLI でビルド・配信します。`VERCEL_TOKEN`、`VERCEL_ORG_ID`、`VERCEL_PROJECT_ID` が必要です。本番デプロイは main の `workflow_dispatch` と `LEPTOS_PRODUCTION_ENABLED=true` で有効になります。
