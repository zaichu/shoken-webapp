# CLAUDE.md

このファイルは backend 作業時に適用するサブディレクトリルールです。
このファイルを読むときは、必ず先にリポジトリルートの `../CLAUDE.md` を読み、
そこから参照されるプロジェクト全体ルール（`../.claude/rules/*.md`）も適用してください。
このファイル単独で判断してはいけません。

## 読み込み順

1. `../CLAUDE.md`（プロジェクト共通）
2. `./CLAUDE.md`（このファイル）
3. `./.claude/rules/00-backend.md`

## コマンド

- `make run` - ローカル起動（wrangler dev）
- `make check` - コンパイルチェック
- `make test` - テスト実行
- `make worker-build` - Workers 向け wasm ビルド

## 補足

- Axum + tokio-postgres。実行環境は Cloudflare Workers のみ（ホスト側常駐サーバーは存在しない）
- 認証・Cookie・CORS を含む backend 固有ルールは `./.claude/rules/00-backend.md` を厳守する
