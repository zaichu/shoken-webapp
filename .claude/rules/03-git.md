# Git・PR ルール

このドキュメントを、Git ブランチ運用の唯一の基準（Single Source of Truth）とする。
README や skills の記載と衝突した場合は、本ドキュメントを優先する。

## ブランチ戦略

- 長期ブランチは `main` のみ
- `develop` は使わない
- 作業は必ず `main` から短期ブランチを作って行う
- `main` への直接コミット・直接 push は禁止
- 1機能・1タスクにつき 1 ブランチ
- マージ後は作業ブランチをローカル/リモート両方で削除する

### 作業ブランチ名

- `feature/<topic>`
- `fix/<topic>`
- `refactor/<topic>`
- `docs/<topic>`
- `chore/<topic>`
- `hotfix/<topic>`

## コミットルール

- `git add .` / `git add -A` は使わず、`git add <path>` または `git add -p` を使う
- 1コミット = 1目的（機能追加とリファクタを混在させない）
- コミットメッセージ（要約・本文）は日本語で書く

### コミットメッセージ形式

```text
<種別>: <変更内容の要約>

<詳細説明（任意）>

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
```

### 種別

- `feat`: 新機能
- `fix`: バグ修正
- `refactor`: リファクタリング
- `docs`: ドキュメント
- `test`: テスト
- `chore`: ビルド・設定変更

## PR とマージ

- PR は作業ブランチから `main` へ作成する
- PR マージ前に Codex レビュー（`.claude/skills/pr-review/SKILL.md`）を実施する
- タイトルと説明は日本語で、変更内容とテスト結果を明記する
- マージ方式は `Squash and merge` を推奨
- マージ後は `main` を更新して作業ブランチを削除する

## 標準フロー

```bash
# 1) main を最新化
git switch main
git pull --ff-only origin main

# 2) 1タスク1ブランチを作成
git switch -c feature/<topic>

# 3) 変更を選択してコミット
git add -p
git commit -m "feat: <変更内容の要約>"

# 4) push と PR 作成
git push -u origin feature/<topic>
gh pr create --base main --head feature/<topic>

# 5) マージ後の後片付け
git switch main
git pull --ff-only origin main
git branch -d feature/<topic>
git push origin --delete feature/<topic>
git fetch origin --prune
```

## 禁止事項

- `main` への直接コミット / 直接 push
- 1つの作業ブランチに複数タスクを混在させること
- 不要な `--force` push
- ユーザーの明示指示なしでの `git branch -D`
