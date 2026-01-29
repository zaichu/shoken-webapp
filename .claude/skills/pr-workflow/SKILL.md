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

- **ブランチは `develop` と `main` のみ**
- feature ブランチは作成しない
- 作業は `develop` に直接コミット
- `main` への反映は **必ず `develop` → `main` の PR 経由**

### 禁止事項

- `main` に直接コミット/マージしない
- feature ブランチを作成しない
- `develop` を経由せずに `main` を更新しない

### ワークフロー

```
1. develop で作業・コミット
2. develop → main の PR を作成
3. レビュー・マージ
```

## コミット分割の原則

1. **機能単位で分割** - 1コミット = 1機能
2. **共通基盤は最初のコミットに含める**
3. **統合ファイルは最後のコミットに含める**

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
git push origin <branch>

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
