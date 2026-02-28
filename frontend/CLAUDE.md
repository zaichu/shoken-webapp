# CLAUDE.md

このファイルは frontend 作業時に適用するサブディレクトリルールです。
このファイルを読むときは、必ず先にリポジトリルートの `../CLAUDE.md` を読み、
そこから参照されるプロジェクト全体ルール（`../.claude/rules/*.md`）も適用してください。
このファイル単独で判断してはいけません。

## 読み込み順

1. `../CLAUDE.md`（プロジェクト共通）
2. `./CLAUDE.md`（このファイル）
3. `./.claude/rules/00-frontend.md`

## コマンド

- `npm run dev` - 開発サーバー起動
- `npm run lint` - Lint 実行
- `npm test` - テスト実行
- `npm run build` - ビルド

## 補足

- UIレビュー時は `../.claude/rules/04-frontend-ui-review.md` を適用する
- React Compiler 前提で `useMemo` / `useCallback` は原則不要
- frontend 固有のテスト・セキュリティ方針は `./.claude/rules/00-frontend.md` を適用する
