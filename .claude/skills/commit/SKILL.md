---
name: commit
description: |
  Git コミット作成。日本語コミットメッセージで規約に従う。
  Use when: コミット、commit、変更をコミットを依頼された時。
---

# コミット作成

## コミットメッセージ形式

```
<種別>: <変更内容の要約>

<詳細説明（任意）>

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
```

## 種別

- `feat`: 新機能
- `fix`: バグ修正
- `refactor`: リファクタリング
- `docs`: ドキュメント
- `test`: テスト
- `chore`: ビルド・設定変更

## 手順

1. 変更状況確認
```bash
git status
git diff
```

2. 最近のコミットスタイル確認
```bash
git log --oneline -10
```

3. ファイルをステージング（個別に追加）
```bash
git add <file1> <file2>
```

4. コミット作成
```bash
git commit -m "$(cat <<'EOF'
feat: 機能の説明

詳細な説明（必要に応じて）

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

## 注意事項

- `git add -A` は避け、ファイルを個別に追加
- .env, credentials などの機密ファイルをコミットしない
- pre-commit hook 失敗時は --amend ではなく新規コミット
- ユーザーが明示的に依頼しない限りコミットしない
