---
name: local-testing
description: |
  ローカル環境でのテスト実行方針。
  デプロイ前にローカルで確認すべき項目と、デプロイが必要な項目を整理。
  Use when: テスト、動作確認、開発フローを確認したい時。
---

# ローカルテスト重視の開発フロー

## 基本方針

- デプロイ前にローカルで確認できることは全てローカルで確認
- デプロイ方針の詳細は `deploy` スキルを参照
- 具体的なビルド/テスト手順は `backend-build-test` / `frontend-build-test` を参照

## ローカルで確認可能な項目

### バックエンド

| 項目 | ローカル可能 | 備考 |
|------|-------------|------|
| コンパイル | ○ | `cargo build` |
| ユニットテスト | ○ | `cargo test` |
| モデル・型定義 | ○ | コンパイルで確認 |
| API構造テスト | ○ | モックで可能 |
| DB接続テスト | △ | ローカルDB必要 |
| 外部API連携 | △ | APIキー必要 |

### フロントエンド

| 項目 | ローカル可能 | 備考 |
|------|-------------|------|
| TypeScript型チェック | ○ | `npm run tsc` |
| ユニットテスト | ○ | `npm test` |
| コンポーネントテスト | ○ | Vitest + RTL |
| API呼び出しモック | ○ | vi.mock使用 |
| 実API連携 | × | 本番環境必要 |

## デプロイが必要な確認項目

- OAuth認証フロー（Google連携）
- 本番DB接続
- 外部API連携（J-Quants等）
- CORS設定
- 本番環境変数

## 推奨開発フロー

1. **機能実装**
   - ローカルでコード作成
   - ユニットテスト作成

2. **ローカル確認**
   - バックエンド: `backend-build-test` に従って実行
   - フロントエンド: `frontend-build-test` に従って実行

3. **PRマージ → 自動デプロイ**
   - main へマージで自動デプロイ（詳細は `deploy` を参照）

4. **本番確認**
   - 外部連携部分のみ本番で確認

## J-Quants API テスト

### ローカルテスト（モック使用）

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_fin_summary_data_deserialize_v2_format() {
        let json_data = json!({
            "DiscDate": "2023-11-14",
            "Code": "72030",
            "NxFDivAnn": "50.00",
        });
        let data: FinSummaryData = serde_json::from_value(json_data).unwrap();
        assert_eq!(data.disclosed_date, "2023-11-14");
    }
}
```

### 実API確認が必要な場合

```bash
# .env に JQUANTS_API_KEY を設定後
cd backend && cargo run
# 別ターミナルで curl や Postman でテスト
```
