# CLAUDE.md

このファイルはプロジェクト全体のガイド（正本）です。

## 読み込み場所

- プロジェクト全体: `./CLAUDE.md`
- 個人設定: `./CLAUDE.local.md`（gitignore 推奨）
- サブディレクトリ作業: `backend/CLAUDE.md`, `frontend/CLAUDE.md`
- 全プロジェクト共通の個人設定（任意・存在する場合のみ適用）: `~/.claude/CLAUDE.md`

## 共通ルール

- 日本語で回答する
- 思考過程・内部推論・検討ログは出力しない
- 出力は必要最小限。前置きや冗長な説明は禁止
- 指示されていない改善・設計提案・リファクタは禁止
- 情報不足時の質問は1つだけ

## 参照ルール

1. `.claude/rules/00-general.md`
2. `.claude/rules/01-testing.md`
3. `.claude/rules/02-security.md`
4. `.claude/rules/03-git.md`
5. UIレビュー時のみ `.claude/rules/04-frontend-ui-review.md`

## サブプロジェクトルール

- バックエンド作業時は `backend/CLAUDE.md` を追加で適用
- フロントエンド作業時は `frontend/CLAUDE.md` を追加で適用
