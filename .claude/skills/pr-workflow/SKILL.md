---
name: pr-workflow
description: |
  コミット履歴をきれいに保つための Git ブランチ運用と PR 作成ワークフロー。
  `main` と短期作業ブランチの役割、コミット分割、PR、マージ後のブランチ削除までを定義する。
  基準ルールは `.claude/rules/03-git.md` に統一し、本スキルはその実行手順を扱う。
  機能・タスクごとに `main` から 1 本ずつ作業ブランチと専用 worktree を作成する前提で運用する。
  Use when: ブランチ運用の相談、PR作成、PRマージ、main更新、マージ後の worktree/local branch/remote branch 削除、PRが多い時の整理を依頼された時。
---

# PR ワークフロー

## 基準ルール

- ブランチ運用の唯一の基準は `.claude/rules/03-git.md`
- 本スキルと他ドキュメントで記載が衝突した場合は `.claude/rules/03-git.md` を優先する

## 基本判断

- durable な記録が必要な作業は `issue-task-lifecycle` を使い、Issue を正本にする。
- 実装を Claude に委譲する場合は `codex-claude-handoff` を使い、専用 worktree で進める。
- PR マージ前は `pr-review` と外部レビューコメントを確認する。

## コミット分割の原則

1. 1コミット = 1目的（機能・修正・リファクタを混在させない）
2. `git add .` / `git add -A` は使わず、`git add <path>` または `git add -p` を使う
3. PR 前に `fixup!` + `rebase -i --autosquash` で不要コミットを整理する
4. `WIP` のような暫定コミットは push 前に整理する

## コミットメッセージ形式

```bash
<種別>: <変更内容の要約>

<詳細説明（任意）>

Co-Authored-By: Claude <noreply@anthropic.com>
```

利用モデルを CLI 出力・API ログ・実行環境の明示情報などで検証できる場合は、`Co-Authored-By: Claude <具体モデル名> <noreply@anthropic.com>` のように具体名を書いてよい。検証できない場合は `Co-Authored-By: Claude <noreply@anthropic.com>` を使う。

- `<変更内容の要約>` と `<詳細説明>` は日本語で記述する

### 種別

- `feat`: 新機能
- `fix`: バグ修正
- `refactor`: リファクタリング
- `docs`: ドキュメント
- `test`: テスト
- `chore`: ビルド・設定変更

## 標準フロー

```bash
# 1) main を最新化
git switch main
git pull --ff-only origin main

# 2) 1タスク1ブランチ + 1worktree を作成
git worktree add -b feature/<topic> /tmp/<repo>-<topic> main

# 3) 変更を目的単位でコミット
cd /tmp/<repo>-<topic>
git add -p
git commit -m "feat: <変更内容の要約>"

# 4) リモートへ push
git push -u origin feature/<topic>
```

## 履歴整理（PR前）

```bash
# 直近コミットを fixup として積む例
git commit --fixup <target-commit-hash>

# main 基準で自動 squash/fixup
git rebase -i --autosquash origin/main
```

## PR作成手順

作業ブランチは `main` 向けに PR を作成する。
PR 本文は実改行で渡す。`\n` を含む1行文字列を使わない。

```bash
# /tmp/<topic>-pr-body.md を apply_patch で作成してから投稿する
gh pr create --base main --head <work-branch> --title "タイトル" --body-file /tmp/<topic>-pr-body.md
```

`/tmp/<topic>-pr-body.md`:

```markdown
## 概要
変更内容の説明

## 変更種別
- [x] 新機能
- [ ] バグ修正

## 主な変更内容
- 項目1
- 項目2

## テスト
- [x] テスト実行確認
- [x] ビルド確認
```

## マージ方式

- 作業ブランチ -> `main`: `Squash and merge` を推奨（1タスク1コミット化）
- `gh pr merge --squash --delete-branch` が worktree 制約で失敗した場合は、PR が merge 済みか確認してから cleanup を個別に行う。

## ブランチ削除（マージ後）

```bash
git switch main
git pull --ff-only origin main
git worktree remove /tmp/<repo>-<topic>
git branch -D <work-branch> # squash merge 済み・remote 削除済みの短期ブランチのみ
git push origin --delete <work-branch>
git fetch origin --prune
```

削除後に `git worktree list`、`git branch --list <work-branch>`、`git branch -r --list origin/<work-branch>` で残骸がないことを確認する。

## 禁止事項

- `main` への直接コミット
- `git add .` / `git add -A`
- 未マージ・未確認のブランチに対する `git branch -D`
- ユーザーが明示的に依頼していないコミット・PR作成
