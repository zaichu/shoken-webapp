# フロントエンド UI レビュー

## 対象

Leptos の主要ページ（ホーム・銘柄検索・資産管理・取引明細）と CSV の取込・削除を確認する。
レビュー観点は `docs/ui-design.md` の「UI の要件(必須)」に従う。
用語・書式の揺れは `docs/ui-design.md` の「用語と書式(全画面の共通ルール)」で確認する
(定義は `frontend/src/ui/labels.rs` と `frontend/src/ui/format.rs`)。

## 手順

1. `frontend/` で `npm ci` を実行する。
2. `env LEPTOS_E2E_PORT=8091 npx playwright test --config playwright.leptos.config.ts` を実行する(UI レビューは画面を見て確かめるため、E2E を CI に任せる通常の作業の例外として手元で回す)。
3. Playwright のスクリーンショットと `test-results/` の結果を確認する。
4. PC 幅とスマホ幅で、読みやすさ、操作性、画面間の一貫性、CSV 操作後の表示、アクセシビリティを確認する。
5. 指摘は `docs/tasks/` の task file に優先度と再現手順を記録する。

実 backend と DB を使う確認では DB → backend → Leptos の順に起動する。`./scripts/start-local.sh` がこの順序を守る。

## CI と同じ環境での再現

手元では通るのに CI でだけ落ちる(フォント差など)が疑われるときは、`frontend/` で `bash scripts/e2e-ci-like.sh [spec] [--grep ...]` を使う。Ubuntu 24.04・CI と同じフォントのコンテナで回し、`--show-fonts` でホストとコンテナの `fc-match` を比べられる。

## 画面を変える PR の確認

- 同じ役割の画面を並べて比べる(国内株式×投資信託、ホーム×取引明細など)。用語・注記の型・部品・小数桁・見出しの太さと位置
- 端のデータで確かめる: 長い名前、2桁の月、複数口座、小数のある値、0件・欠測
- 幅は 1024・1280・1440・1920・390px、ダークとライトの両方
- 列見出しの縁が値の縁とそろう(右そろえの列は右端が一致)。表が枠からはみ出さない
- E2E の「枠に収まる」などを弱めて通さない。テストデータに端のデータを足す
- スクショは架空データのみ
