---
name: bulk-processing
description: |
  バルクデータ処理の最適化パターン。
  UNNEST、UPSERT、重複スキップ、パフォーマンス計測。
  Use when: 一括登録、バルク処理、CSV取り込み、大量データ処理を依頼された時。
---

# バルク処理 実装ガイド

## 目的

- 大量データを効率的に DB に登録する
- 重複データを適切に処理する（スキップまたは更新）
- パフォーマンスを計測し、ボトルネックを特定する

## 適用場面

- CSV ファイルからのデータ取り込み
- API からのバルクデータ登録
- 既存データの一括更新
- データ移行処理

## バルク INSERT パターン

### 1. UNNEST による一括挿入（推奨）

1回のクエリで全件挿入。ループより圧倒的に高速。
ユーザー行ロック・行数上限チェック・空配列ガード・計測ログは
`services/domain/bulk.rs` の `bulk_insert`（追記型）/ `bulk_replace`（置換型）が持つため、
ドメイン側はカラム配列の組み立てと UNNEST の INSERT クエリだけを書く。

```rust
// services/your_domain.rs（実例: services/dividend.rs）
use crate::db::{Bind, Db};
use crate::services::domain::bulk::{bulk_insert, user_ids_for_bulk_insert, RowLimit};

pub async fn bulk_create(
    pool: &Db,
    user_id: UserId,
    items: &[CreateRequest],
    limit: RowLimit,
) -> Result<BulkCreateResponse, ApiError> {
    let user_ids = user_ids_for_bulk_insert(user_id, items.len());
    let field1s: Vec<String> = items.iter().map(|i| i.field1.clone()).collect();
    let field2s: Vec<Decimal> = items.iter().map(|i| i.field2).collect();

    bulk_insert::<YourDomain, _>(
        pool,
        user_id,
        items,
        limit,
        crate::db::query(
            r#"
            INSERT INTO your_table (user_id, field1, field2)
            SELECT * FROM UNNEST($1::uuid[], $2::text[], $3::numeric[])
            ON CONFLICT (user_id, field1) DO NOTHING
            "#,
            vec![
                Bind::from(user_ids),
                Bind::from(field1s),
                Bind::decimal_vec(field2s),
            ],
        ),
    )
    .await
}
```

### 2. UPSERT（既存データを更新）

同じキーで値を更新したい場合（実例: `services/auth.rs` のユーザー upsert、
`services/dividend_cache.rs`）。

```rust
let affected = crate::db::query(
    r#"
    INSERT INTO your_table (user_id, security_code, shares, current_price)
    SELECT * FROM UNNEST($1::uuid[], $2::varchar[], $3::numeric[], $4::numeric[])
    ON CONFLICT (user_id, security_code)
    DO UPDATE SET
        shares = EXCLUDED.shares,
        current_price = EXCLUDED.current_price,
        updated_at = NOW()
    "#,
    vec![
        Bind::from(user_ids),
        Bind::from(security_codes),
        Bind::decimal_vec(shares),
        Bind::decimal_vec(current_prices),
    ],
)
.execute(pool)
.await?;

// UPSERT では skipped は常に 0（更新も rows_affected に含まれる）
```

## PostgreSQL 型マッピング（UNNEST 用）

バインドは `Vec<Bind>` で渡し、配列は宣言型つきの `Bind::custom` に変換される。
`From<Vec<T>>` impl があるのは下記だけ（`db/bind.rs`）。それ以外の配列型は
`Bind::custom(Type::<…>_ARRAY, vec)` で包むか `From` impl を追加する。

| `Bind::from` の対象 | PostgreSQL キャスト |
|---------------------|---------------------|
| `Vec<UserId>` | `$n::uuid[]` |
| `Vec<String>` / `Vec<Option<String>>` | `$n::text[]` |
| `Vec<SecurityCode>` | `$n::varchar[]` |
| `Vec<NaiveDate>` | `$n::date[]` |
| `Vec<Decimal>` → `Bind::decimal_vec` | `$n::numeric[]` |

## チェックリスト

### 実装前
- [ ] ユニーク制約を確認（ON CONFLICT の対象キー）
- [ ] 重複時の挙動を決定（スキップ or 更新）
- [ ] バリデーションを実装（空配列、フィールド長など）

### 実装時
- [ ] UNNEST を使用（ループ処理は避ける）
- [ ] 処理時間を計測してログ出力
- [ ] inserted/skipped を正確に計算

### テスト
- [ ] 空配列の場合のテスト
- [ ] 重複データ挿入時のテスト
- [ ] 大量データ（1000件以上）でのパフォーマンス確認

## よくある失敗

### 1. ループで1件ずつ INSERT

```rust
// NG: N回のクエリ発行で遅い
for item in payload.items {
    crate::db::query("INSERT INTO ...", vec![...]).execute(pool).await?;
}

// OK: 1回のクエリで全件挿入
crate::db::query("INSERT INTO ... SELECT * FROM UNNEST(...)", vec![...])
    .execute(pool)
    .await?;
```

### 2. ON CONFLICT の対象キーが PK と不一致

```sql
-- NG: PK が (date, code) なのに code のみ指定
ON CONFLICT (code) DO NOTHING

-- OK: PK に合わせる
ON CONFLICT (date, code) DO NOTHING
```

### 3. rows_affected の解釈ミス

```rust
// UPSERT の場合、UPDATE も rows_affected（execute の戻り値）に含まれる
// → skipped = total - inserted は意味がない

// 正しい解釈
let upserted = query.execute(pool).await?;  // INSERT + UPDATE の合計 (u64)
let skipped = 0;  // UPSERT では常に 0
```

### 4. 空配列チェック漏れ

```rust
// NG: 空配列で UNNEST する経路を自前で組むとエラーになり得る
crate::db::query("INSERT INTO ... SELECT * FROM UNNEST($1::uuid[])", vec![...])
    .execute(pool)
    .await?;

// OK: bulk_insert のガード（BulkTimer::new_with_guard）が空なら DB に触れず即時応答する
```

## パフォーマンス目安

| 件数 | 処理時間（目安） |
|------|------------------|
| 100件 | < 50ms |
| 1,000件 | < 200ms |
| 10,000件 | < 1s |

※ Neon PostgreSQL（サーバーレス）の場合。接続プールの状態により変動。

## レスポンス形式

```rust
#[derive(Debug, Serialize)]
pub struct BulkCreateResponse {
    pub inserted: usize,  // 新規挿入件数
    pub skipped: usize,   // 重複スキップ件数（UPSERT では 0）
}
```

フロントエンドでの表示例:
```
100件中 95件を登録しました（5件は重複のためスキップ）
```

## 参考ファイル

- `backend/src/services/domain/bulk.rs` - `bulk_insert` / `bulk_replace` の共通骨格
- `backend/src/services/dividend.rs` - 追記型（ON CONFLICT DO NOTHING）実例
- `backend/src/services/asset_balance.rs` - 置換型実例
- `backend/src/services/auth.rs` - UPSERT（ON CONFLICT DO UPDATE）実例
- `backend/src/db/bind.rs` - `Bind` / 配列バインドの宣言型対応
