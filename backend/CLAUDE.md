# CLAUDE.md

このファイルは backend 作業時に適用するサブディレクトリルールです。

## 読み込み順

1. `../CLAUDE.md`（プロジェクト共通）
2. `./CLAUDE.md`（このファイル）
3. `./.claude/rules/00-backend.md`
4. `./.claude/rules/01-testing.md`
5. `./.claude/rules/02-security.md`

## コマンド

- `make run` - ローカル起動
- `make check` - コンパイルチェック
- `make test` - テスト実行
- `make build` - ビルド

## 補足

- Axum + SQLx の型安全性を優先する
- 認証・Cookie・CORS は `./.claude/rules/02-security.md` を厳守する
