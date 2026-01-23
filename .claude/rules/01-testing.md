# テスト共通ルール

## テスト原則

- 本番データをテストに使用しない
- 日本語テストデータ（銘柄名など）を適切に使用
- フィクスチャは `__fixtures__/` に配置

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
