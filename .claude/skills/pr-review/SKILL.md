---
name: pr-review
description: |
  Codex が実装・push した PR を Claude が自動検出してレビューする。
  Use when: 「PRをレビューして」「レビューしてください」と依頼された時。
  または Codex から PR 作成完了の通知を受けた後。
---

# PR Review（Claude 実行）

## Overview
Codex が実装して push した PR を Claude がレビューする。
全体開発ルール（`.claude/rules/00-general.md`）における標準フローは
`Codex が実装する → Claude がレビューする` とし、本スキルで運用する。

ブランチ運用の基準は `.claude/rules/03-git.md` を参照する。

## Workflow

### 1. レビュー対象 PR を自動検出する
- `git fetch origin` で比較元を最新化する
- `git branch --show-current` で現在ブランチを確認する
- `gh pr list --state open --head "$(git branch --show-current)" --json number,title,url,body` で open PR を探す
- 見つからない場合は、ユーザーに PR 番号 or URL を確認する

### 2. レビュー材料を収集する
- `git diff origin/main...HEAD --name-status`
- `git diff origin/main...HEAD --stat`
- `git diff origin/main...HEAD`
- `gh pr view <PR番号> --json number,title,url,body`

### 3. 検証結果を確定する
変更範囲に応じて実行:

- フロントエンド変更がある場合:
  - `cd frontend && npm run lint`
  - `cd frontend && npx tsc --noEmit`
  - `cd frontend && npm test`
  - `cd frontend && npm run build`
- バックエンド変更がある場合:
  - `cd backend && cargo fmt --check`
  - `cd backend && cargo clippy --all-targets -- -D warnings`
  - `cd backend && cargo test`
- API 定義変更がある場合:
  - `bash scripts/check-openapi.sh`

### 4. レビューを実行する
findings first で重大度順に出す:

1. **Findings**（重大度順、`path:line` 付き）
   - `[high]` / `[medium]` / `[low]` で分類
2. **Open questions / assumptions**（あれば）
3. **判定**: LGTM / 要修正

### 5. レビュー結果を PR にコメントする
- 既存の `🤖 Claude review` コメントを確認: `gh api repos/{owner}/{repo}/issues/<PR番号>/comments --jq '[.[] | select(.body | startswith("🤖 Claude review"))] | last | .id'`
- 存在する場合は更新: `gh api repos/{owner}/{repo}/issues/comments/<comment_id> -X PATCH -f body="..."`
- 存在しない場合は新規投稿: `gh pr comment <PR番号> --body "..."`
- コメント先頭は `🤖 Claude review` で始める

### 6. 修正が必要な場合
- `.claude/skills/claude-codex-handoff/SKILL.md` を使って Codex に修正を依頼する
- 修正 push 後は本スキルで再レビューする（LGTM まで繰り返す）

### 7. LGTM 後
- PR をマージする: `gh pr merge <PR番号> --squash --delete-branch`
- main を更新: `git switch main && git pull --ff-only origin main`

## Review Rules
- findings first / 重大度順 / ファイルパスと行番号を必須とする
- 検証コマンドと pass/fail を必ず記録する
- PR コメントへの返信も必ず行う（CodeRabbit 等の自動レビューも含む）

## Output
レビュー結果（findings first）を出力し、PR にコメントする。
