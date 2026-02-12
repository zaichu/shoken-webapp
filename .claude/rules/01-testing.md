# テスト共通ルール

## テスト原則

- 本番データをテストに使用しない
- 日本語テストデータ（銘柄名など）を適切に使用
- フィクスチャは `__fixtures__/` に配置
- ローカルでフロントエンドとバックエンドを同時に使う検証は、**必ずバックエンド起動後にフロントエンドを起動**する

## ローカル起動順ルール

ローカルデバッグ / E2E / UIレビューでは起動順を固定する:

1. バックエンド起動（例: `http://127.0.0.1:3001/health` が200になるまで待つ）
2. フロントエンド起動（バックエンドURLをローカル向けに指定）

## CI/CD でのテスト

GitHub Actions でのテスト実行:

```yaml
- name: Run frontend tests
  working-directory: frontend
  run: npm test

- name: Run backend tests
  working-directory: backend
  run: cargo test
```
