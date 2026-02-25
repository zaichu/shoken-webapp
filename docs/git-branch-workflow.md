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
- `develop`
  - 統合用の長期ブランチ
  - 作業ブランチの起点
  - 直接コミット禁止
- 作業ブランチ（短期）
  - `feature/<topic>`
  - `fix/<topic>`
  - `refactor/<topic>`
  - `docs/<topic>`
  - `chore/<topic>`

## 基本ルール

- 1機能・1タスクにつき作業ブランチは 1 本
- 作業ブランチは必ず `develop` から作る
- コミットメッセージ（要約・本文）は日本語
- `git add .` / `git add -A` は使わず、`git add <path>` または `git add -p` を使う
- PR マージ後に作業ブランチをローカル・リモート両方で削除する

## 標準フロー

```bash
# 1) develop を最新化
git switch develop
git pull --ff-only origin develop

# 2) 作業ブランチ作成（1タスク1ブランチ）
git switch -c feature/<topic>

# 3) 変更を選択してコミット
git add -p
git commit -m "feat: <変更内容の要約>"

# 4) push と PR（作業ブランチ -> develop）
git push -u origin feature/<topic>
gh pr create --base develop --head feature/<topic>

# 5) マージ後にブランチ削除
git switch develop
git pull --ff-only origin develop
git branch -d feature/<topic>
git push origin --delete feature/<topic>
git fetch origin --prune
```

## リリースフロー

- `develop` から `main` へ PR を作成して反映する
- マージ方式は履歴を直線に保てる方法を使う（`Rebase and merge` 推奨）
- `main` 反映後に CI/CD で自動デプロイ

## 禁止事項

- `main` / `develop` への直接コミット
- 1つの作業ブランチに複数タスクの変更を混在させること
- 不要な強制操作（例: 不要な `--force` push、`git branch -D`）

## 例外運用（緊急時）

- 緊急修正が必要な場合のみ `hotfix/<topic>` を `main` から作成して対応可
- `main` へ反映後、同内容を `develop` にも取り込んで差分を解消する
