# Git ブランチ運用ポリシー

このドキュメントを、ブランチ運用の唯一の基準（Single Source of Truth）とする。
README と各 skill の記載が衝突した場合は、本ドキュメントを優先する。

## 目的

- コミット履歴を読みやすく保つ
- 1タスク単位で差分を分離し、レビュー容易性を上げる
- `main` を常に本番反映可能な状態に保つ

## ブランチの役割

- `main`
  - 本番反映用の長期ブランチ
  - 直接コミット禁止
- 作業ブランチ（短期）
  - `feature/<topic>`
  - `fix/<topic>`
  - `refactor/<topic>`
  - `docs/<topic>`
  - `chore/<topic>`

## 基本ルール

- 1機能・1タスクにつき作業ブランチは 1 本
- 作業ブランチは必ず最新の `main` から作る
- コミットメッセージ（要約・本文）は日本語
- `git add .` / `git add -A` は使わず、`git add <path>` または `git add -p` を使う
- PR は作業ブランチから `main` へ作成する
- PR マージ後に作業ブランチをローカル・リモート両方で削除する

## 標準フロー

```bash
# 1) main を最新化
git switch main
git pull --ff-only origin main

# 2) 作業ブランチ作成（1タスク1ブランチ）
git switch -c feature/<topic>

# 3) 変更を選択してコミット
git add -p
git commit -m "feat: <変更内容の要約>"

# 4) push と PR（作業ブランチ -> main）
git push -u origin feature/<topic>
gh pr create --base main --head feature/<topic>

# 5) マージ後にブランチ削除
git switch main
git pull --ff-only origin main
git branch -d feature/<topic>
git push origin --delete feature/<topic>
git fetch origin --prune
```

## マージ方式

- `Squash and merge` を推奨（1タスク1コミット化しやすいため）

## デプロイフロー

- `main` へのマージ後に CI/CD で自動デプロイ

## 禁止事項

- `main` への直接コミット
- 1つの作業ブランチに複数タスクの変更を混在させること
- 不要な強制操作（例: 不要な `--force` push、`git branch -D`）

## 例外運用（緊急時）

- 緊急修正も `main` から `hotfix/<topic>` ブランチを切って対応する
- 緊急修正も PR 経由で `main` へ反映する
