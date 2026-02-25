---
name: pr-workflow
description: |
  コミット分割とPR作成のワークフロー。
  機能ごとにコミットを分け、PRを作成する。
  Use when: コミット、PR作成、プルリクエストを依頼された時。
---

# PR ワークフロー

## ブランチ戦略

### 基本ルール

- **`main` を常に最新化し、作業は短命 feature ブランチで行う**
- 1タスク = 1ブランチ（例: `fix/asset-balance-lock-test`）
- PR は **feature ブランチ → `main`**
- `develop` は既定の開発ブランチとして使わない（必要時のみ検証用に限定）
- マージ完了後は feature ブランチをローカル/リモートとも削除する

### 禁止事項

- `main` に直接コミット/マージしない
- `develop` への直接コミットを日常運用にしない
- 長期間の `develop` 集約後に巨大な `develop -> main` PR を作らない
- feature ブランチで `merge` を多用しない（`rebase` で履歴を保つ）
- マージ済みブランチを放置しない

### ワークフロー

```
1. main を最新化
2. feature ブランチ作成
3. feature ブランチで作業・コミット
4. feature → main の PR を作成
5. レビュー後に squash merge
6. マージ済み feature ブランチを削除
```

### 開始手順（毎回）

```bash
git checkout main
git pull origin main
git switch -c <type>/<short-topic>
```

### PR前の整形（必須）

```bash
# feature ブランチを main に追従
git fetch origin
git rebase origin/main

# 必要ならコミット統合（fixup/squash）
git rebase -i origin/main
```

### 既存 `develop` からクリーンブランチへ載せ替える手順

```bash
# 1. main を最新化
git checkout main
git pull origin main

# 2. 新しい feature ブランチを作る
git switch -c <type>/<short-topic>

# 3. develop の必要コミットだけを順に取り込む
git cherry-pick <commit1> <commit2> <commit3>

# 4. 競合解消後に履歴整形
git rebase -i origin/main

# 5. プッシュして PR 作成（base は main）
git push -u origin <type>/<short-topic>
gh pr create --base main --head <type>/<short-topic>
```

### マージ後クリーンアップ（必須）

```bash
# 1. PRを squash merge し、リモートブランチを削除
gh pr merge <PR番号 or PR URL> --squash --delete-branch

# 2. main を最新化
git checkout main
git pull origin main

# 3. ローカルの作業ブランチを削除
git branch -d <type>/<short-topic>
```

`gh` で削除できなかった場合のみ、リモートは手動削除する。

```bash
git push origin --delete <type>/<short-topic>
```

## コミット分割の原則

1. **機能単位で分割** - 1コミット = 1機能
2. **共通基盤は最初のコミットに含める**
3. **統合ファイルは最後のコミットに含める**

## コミットメッセージ形式

```
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

## 分割例（複数機能追加時）

```
1. 共通基盤 + 機能A
   - 認証エクストラクター、APIクライアント基盤
   - 機能A固有ファイル

2. 機能B
   - 機能B固有ファイル

3. 機能C + 統合
   - 機能C固有ファイル
   - main.rs、handlers.rs などの統合ファイル
```

## コミット手順

```bash
# 1. 変更状況確認
git status
git diff

# 2. 機能ごとにステージング・コミット
git add <files>
git commit -m "$(cat <<'EOF'
feat: 機能の説明

- 変更点1
- 変更点2

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"

# 3. 繰り返し
```

## PR作成手順

```bash
# 1. プッシュ
git push -u origin <branch>

# 2. PR作成
gh pr create --base main --head <branch> --title "タイトル" --body "$(cat <<'EOF'
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

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

## よくあるパターン

### Skills + 機能追加
```
1. feat: Agent Skills を追加
2. feat: 機能AのDB取込機能を追加
3. feat: 機能BのDB取込機能を追加
4. feat: 機能CのDB取込機能を追加
```

### リファクタリング + 機能追加
```
1. refactor: 共通処理を抽出
2. feat: 新機能を追加
```

## 注意事項

- ユーザーが明示的に依頼しない限りコミット・PRを作成しない
- `git add -A` は避け、ファイルを個別に追加
- 機密ファイル（.env等）をコミットしない
- PRは小さく保つ（目安: 1目的、レビュー可能な差分量）
