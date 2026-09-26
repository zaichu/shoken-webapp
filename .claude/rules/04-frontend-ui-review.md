# フロントエンド UI レビュー

## 対象

Leptos の主要ページ（ホーム・銘柄検索・資産管理・取引明細）と CSV の取込・削除を確認する。

## 手順

1. `frontend/` で `npm ci` を実行する。
2. `env LEPTOS_E2E_PORT=8091 npx playwright test --config playwright.leptos.config.ts` を実行する。
3. Playwright のスクリーンショットと `test-results/` の結果を確認する。
4. PC 幅とスマホ幅で、読みやすさ、操作性、画面間の一貫性、CSV 操作後の表示、アクセシビリティを確認する。
5. 指摘は `docs/tasks/` の task file に優先度と再現手順を記録する。

実 backend と DB を使う確認では DB → backend → Leptos の順に起動する。`./scripts/start-local.sh` がこの順序を守る。
