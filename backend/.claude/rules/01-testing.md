# バックエンドテストルール

## テストフレームワーク

- **cargo test**: テストランナー
- **mockall**: モック生成
- **tokio-test**: 非同期テスト

## ディレクトリ構成

```
backend/
├── src/
│   └── handlers/
│       └── stock.rs      # #[cfg(test)] mod tests {}
└── tests/
    └── integration/      # 統合テスト
        └── stock_api.rs
```

## テスト実行

```bash
make test                  # cargo test
cargo test -- --nocapture  # 出力表示
```

## ユニットテストの書き方

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_stock_info() {
        // Arrange
        let pool = setup_test_db().await;

        // Act
        let result = fetch_stock_info(&pool, "7203").await;

        // Assert
        assert!(result.is_ok());
    }
}
```

## モックの使用

```rust
use mockall::predicate::*;
use mockall::*;

#[automock]
trait StockRepository {
    async fn find_by_code(&self, code: &str) -> Option<Stock>;
}

#[tokio::test]
async fn test_with_mock() {
    let mut mock = MockStockRepository::new();
    mock.expect_find_by_code()
        .with(eq("7203"))
        .returning(|_| Some(Stock::default()));

    // テスト実行
}
```

## カバレッジ

```bash
cargo tarpaulin --out Html
```
