# 開発環境・撮影手順 (Phase 1 OpenCode)

## 前提

- Node.js 22, npm
- Rust toolchain (リポジトリルートの `rust-toolchain.toml`)
- Trunk 0.21.4
- Playwright は `frontend/` で `npm ci` 済み

## ビルドと配信

```bash
cd frontend
env RUSTC_WRAPPER=sccache trunk build --release --dist /tmp/opencode/dist-leptos
node scripts/prepare-dist.mjs /tmp/opencode/dist-leptos ''
node scripts/serve-dist.mjs /tmp/opencode/dist-leptos 8170
```

- `prepare-dist.mjs` は API origin を空にし、モック外の API 通信を同一オリジンに留める。
- ポートは 8160-8199 の空きを使う (複数 worktree 並行時は別ポート)。

## 撮影

基本 15 画面 + 追加状態は `docs/design/work/phase1-opencode/tools/` のスクリプトを使用。

```bash
node docs/design/work/phase1-opencode/tools/shoot-baseline.mjs
node docs/design/work/phase1-opencode/tools/shoot-extras.mjs
node docs/design/work/phase1-opencode/tools/shoot-dialogs.mjs
node docs/design/work/phase1-opencode/tools/shoot-admin-empty.mjs
```

- 各スクリプトは自身の場所を基準にリポジトリルートを解決するため、チェックアウトの場所が変わってもそのまま実行できる。

各スクリプトは Playwright Chromium で `http://127.0.0.1:8170` に接続し、`screens.mjs` の架空モックを適用して PNG を `shots/` に保存する。

## モック

- `frontend/scripts/ui-shots/screens.mjs` の `prepareContext` / `installMocks` を再利用。
- セッションは `MOCK_USER` (`test@example.com` / テストユーザー)。
- API はすべてローカルでモック。本番 backend・実データ・`.env` は不使用。

## 確認コマンド

```bash
cd docs/design/work/phase1-opencode/shots
find . -name '*.png' -size +100k   # 超過がないこと
ls | wc -l
```

## 未確認

- 実機ブラウザ・フォント差は未確認。
- CI コンテナ (`scripts/e2e-ci-like.sh`) での再現は未実施。
