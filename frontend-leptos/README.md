# frontend-leptos

React フロントエンドの Leptos (CSR) 移行版。Issue #882 の開発・検証基盤。
本番経路は `frontend/` のまま。ここは単独 Cargo パッケージで backend の workspace には入れない。

## 前提

- Rust 1.96.0 + `wasm32-unknown-unknown` ターゲット (`rustup target add wasm32-unknown-unknown`)
- trunk 0.21.4
- Node v22.14.0 / npm 10.9.2

## セットアップ

```bash
cd ../frontend && npm ci
cd ../frontend-leptos && npm ci
```

`@playwright/test` は `file:../frontend/node_modules/@playwright/test` で frontend 側の実体を参照する。別インスタンスになると Playwright が二重読み込みエラーを出すため。

## コマンド

```bash
trunk serve --port 8081
trunk build --release
npx playwright test --config playwright.leptos.config.ts
npx playwright test --config playwright.receipts.config.ts
cargo fmt --check
cargo clippy --target wasm32-unknown-unknown -- -D warnings
```

`playwright.leptos.config.ts` は `frontend/e2e` の既存 spec (search-flow / receipt-flow) を無改修で実行する。`playwright.receipts.config.ts` は `e2e/receipts-cache.spec.ts` (4件) を実行する。

## CSS

Tailwind browser CDN は使わない。`style/input.css` (`@theme` は `frontend/src/styles/tailwind.css` から流用) を Trunk の pre_build フックで `@tailwindcss/cli` (lock 済み) により `style/output.css` へ生成し、`index.html` の `<link data-trunk rel="css">` で成果物に含める。生成物は git 管理外。

## 実装範囲

6ページの表示系のみ。操作系 (CSV 取込・削除など) はない。

| ページ | 状態 |
|---|---|
| 検索 (`/search`) | 単一銘柄の取得・表示、`code` クエリ対応。E2E 2件が通過 |
| 取引明細 (`/receipts`) | 3タブの一覧表示とタブ別キャッシュ。E2E 3+4件が通過 |
| ホーム (`/`) | 静的表示のみ |
| 資産管理 (`/assetbalance`) | 一覧・評価・構成比・KPI を表示。検索・CSV 取込なし。配当はバッチ取得後、pending が残る間は15秒間隔で再ポーリング（通信エラー時も上限付きでリトライ） |
| ログイン (`/login`) | Google 認証への入口遷移のみ |
| 404 | 静的表示のみ |

本番 React と同等ではない点:

- 取引明細に PoC 用のユーザー切替バー (`AuthSimulator`) が残っている
- 金額の型が暫定 (取引明細は `String`、資産管理は `f64`) で実 API の JSON number との互換は未検証 (#883 で扱う)
- 明細の取得は `per_page=1000&page=1` 固定でページネーション UI なし
- 画面遷移は通常の `<a>` によるフルリロード
- ログイン画面の Google ロゴは外部 URL (`gstatic.com`) 参照のまま
