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

## アクセシビリティ

`e2e/migrated/a11y.spec.ts` が関門になる。axe は WCAG 2A/2AA/2.2AA で moderate 以上ゼロを必須にする(minor は記録のみ)。対象は主要画面と読み込み中・取得失敗・空・CSV プレビュー・確認モーダル・フィルター展開で、PC 1280px とスマホ 390px の両方を見る。

- タップ領域: 24px 以上。スマホの操作要素は 44px 以上(`max-sm:min-h-[44px]`)
- フォーカスは見えること。`aria-*` の真偽値は `"true"`/`"false"` 文字列で出す(bool のまま置かない)
- 読み込み中・失敗・CSV 結果は `role="status"`/`role="alert"` と `aria-live` で伝え、操作中は `aria-busy` を付ける
- `prefers-reduced-motion` ではアニメーションを止める(`style/input.css` のメディアクエリ)
- マイナスは `-` の符号と赤の両方で表す(色だけにしない)。文字色は AA コントラストを満たすものだけ使う
- 直せない違反は除外リストに Issue 番号付きで載せる(今は #1066 の配当 emerald、#1070 のスマホ 44px 未満、#1071 のホーム)

## CSS

`style/input.css` を Trunk の pre_build フックで `style/output.css` に生成し、`index.html` から読み込みます。生成物は Git 管理外です。

## デプロイ

GitHub Actions の `deploy-frontend.yml` が Vercel CLI でビルド・配信します。`VERCEL_TOKEN`、`VERCEL_ORG_ID`、`VERCEL_PROJECT_ID` が必要です。本番デプロイは main の `workflow_dispatch` と `LEPTOS_PRODUCTION_ENABLED=true` で有効になります。
