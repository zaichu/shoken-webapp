# Git・PR ルール

## ブランチ戦略

```
main        # 本番環境（保護ブランチ）
develop     # 開発統合ブランチ
feature/*   # 機能開発
fix/*       # バグ修正
hotfix/*    # 緊急修正
```

### ブランチ命名規則

```bash
feature/add-stock-search      # 機能追加
fix/stock-price-display       # バグ修正
hotfix/auth-session-expired   # 緊急修正
```

## コミットメッセージ

### 形式

日本語で記述:

```
<種別>: <変更内容の要約>

<詳細説明（任意）>
```

### 種別

- `feat`: 新機能
- `fix`: バグ修正
- `refactor`: リファクタリング
- `docs`: ドキュメント
- `test`: テスト
- `chore`: ビルド・設定変更

### 例

```bash
feat: 株式検索機能を追加

- 銘柄コードによる検索
- 会社名による部分一致検索
- J-Quants API との連携
```

```bash
fix: 取引履歴のCSVインポートエラーを修正

Shift-JISエンコーディングの自動検出に対応
```

## Pull Request

### PRの作成

- `develop` → `main` へPRを作成
- タイトルは日本語で簡潔に
- テンプレートに従って記述

### PRテンプレート

```markdown
## 概要
<!-- 変更内容の説明 -->

## 変更種別
- [ ] 新機能
- [ ] バグ修正
- [ ] リファクタリング
- [ ] その他

## テスト
- [ ] ユニットテスト追加/更新
- [ ] 手動テスト実施

## チェックリスト
- [ ] コードレビュー依頼済み
- [ ] CIパス確認
- [ ] ドキュメント更新（必要な場合）
```

### レビュー観点

- 機能要件を満たしているか
- コーディング規約に準拠しているか
- テストが適切に書かれているか
- セキュリティ上の問題がないか

## マージ戦略

- **Squash and Merge** を基本とする
- コミット履歴をクリーンに保つ
- マージ後はブランチを削除

## リリースフロー

1. `develop` で開発・テスト
2. `develop` → `main` へPR作成
3. レビュー・承認
4. マージ後、自動デプロイ

## .gitignore

### 共通

```
.env
.env.local
*.log
.DS_Store
```

### フロントエンド

```
node_modules/
dist/
.env.development.local
```

### バックエンド

```
target/
Secrets.toml
```

## Git操作のベストプラクティス

### 作業開始前

```bash
git checkout develop
git pull origin develop
git checkout -b feature/new-feature
```

### コミット前

```bash
# フロントエンド
cd frontend && npm run lint && npm test

# バックエンド
cd backend && cargo check && cargo test
```

### プッシュ

```bash
git push -u origin feature/new-feature
```

## コンフリクト解決

1. `develop` を最新化
2. 作業ブランチにマージ
3. コンフリクト解決
4. テスト実行で動作確認
