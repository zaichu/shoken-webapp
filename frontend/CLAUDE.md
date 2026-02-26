# CLAUDE.md

このファイルは frontend 作業時に適用するサブディレクトリルールです。

## 読み込み順

1. `../CLAUDE.md`（プロジェクト共通）
2. `./CLAUDE.md`（このファイル）
3. `./.claude/rules/00-frontend.md`
4. `./.claude/rules/01-testing.md`
5. `./.claude/rules/02-security.md`

## コマンド

- `npm run dev` - 開発サーバー起動
- `npm run lint` - Lint 実行
- `npm test` - テスト実行
- `npm run build` - ビルド

## 補足

- UIレビュー時は `../.claude/rules/04-frontend-ui-review.md` を適用する
- React Compiler 前提で `useMemo` / `useCallback` は原則不要
