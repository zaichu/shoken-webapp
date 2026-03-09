# backend/migrations

SQLx マイグレーション履歴の説明書。
**既存の `.sql` ファイルは絶対に編集・削除しない**（SQLx はチェックサムで検証するため、変更すると `cargo sqlx migrate run` が失敗する）。

---

## 一覧

| No. | ファイル | 対象テーブル | 変更内容 |
|-----|---------|-------------|---------|
| 0001 | `0001_init.sql` | `stock` | pgcrypto/citext 拡張、銘柄マスタ初期作成 |
| 0002 | `0002_create_users.sql` | `users` | Google OAuth ユーザー管理テーブル |
| 0003 | `0003_create_sessions.sql` | `sessions` | セッション管理テーブル（7日有効期限） |
| 0004 | `0004_create_dividends.sql` | `dividends` | 配当金取引テーブル（DOUBLE PRECISION） |
| 0005 | `0005_create_domestic_stocks.sql` | `domestic_stocks` | 国内株式取引テーブル（DOUBLE PRECISION） |
| 0006 | `0006_create_mutualfunds.sql` | `mutualfunds` | 投資信託取引テーブル（DOUBLE PRECISION） |
| 0007 | `0007_create_asset_balances.sql` | `asset_balances` | 資産残高（保有銘柄）テーブル |
| 0008 | `0008_fix_stock_schema.sql` | `stock` | 主キーを (code) → (date, code) に変更、業種カラム長拡張 |
| 0009 | `0009_create_jquants_dividend_cache.sql` | `jquants_dividend_cache`, `jquants_rate_control` | J-Quants API キャッシュ・レートコントロールテーブル |
| 0010 | `0010_drop_domestic_stocks_unique.sql` | `domestic_stocks` | 同一内容の重複取引を許容するため UNIQUE 制約を削除 |
| 0011 | `0011_add_content_hash_to_domestic_stocks.sql` | `domestic_stocks` | `content_hash` / `occurrence_index` 追加、重複判定ロジックを実装 |
| 0012 | `0012_add_batch_fingerprint_to_domestic_stocks.sql` | `domestic_stocks` | `import_batch_fingerprint` 追加（試験実装） |
| 0013 | `0013_remove_batch_fingerprint_from_domestic_stocks.sql` | `domestic_stocks` | `import_batch_fingerprint` を削除、全件 `occurrence_index` を再付番 |
| 0014 | `0014_add_stale_at_to_dividend_cache.sql` | `jquants_dividend_cache` | `stale_at`（有効期限 7 日）カラム追加 |
| 0015 | `0015_fix_dividends_unique_index.sql` | `dividends` | UNIQUE インデックスに `security_name` を追加 |
| 0016 | `0016_alter_money_columns_to_numeric.sql` | `dividends`, `domestic_stocks`, `mutualfunds`, `asset_balances` | 金額カラムを DOUBLE PRECISION → NUMERIC へ移行 |
| 0017 | `0017_recalculate_domestic_stocks_content_hash.sql` | `domestic_stocks` | 0016 の型変更に合わせて `content_hash` と `occurrence_index` を再計算 |

---

## 変遷グループ

### domestic_stocks の重複判定履歴（0010→0011→0012→0013）

同一内容の国内株式取引が複数行存在できるユースケースへの対応。

| No. | 変更 | 理由 |
|-----|------|------|
| 0010 | UNIQUE 制約を削除 | 同日・同銘柄・同数量の取引が2件存在するケースが発生 |
| 0011 | `content_hash` + `occurrence_index` 追加 | 内容ハッシュと出現順で「同一行を何回目に見たか」を一意識別 |
| 0012 | `import_batch_fingerprint` 追加 | インポートバッチ単位で重複管理を試みた（試験実装） |
| 0013 | `import_batch_fingerprint` 削除 | バッチ単位管理では既存データの遡及適用ができないため廃止、`occurrence_index` をグローバル通番に一本化 |

**読み方**: 0011 と 0013 が事実上の「正規化完了版」。0012 は中間の試行錯誤として記録されている。

### 金額型移行（0016→0017）

| No. | 変更 | 理由 |
|-----|------|------|
| 0016 | DOUBLE PRECISION → NUMERIC | 浮動小数点の誤差を排除し金額計算の精度を保証 |
| 0017 | `content_hash` 再計算 | NUMERIC 型の文字列表現（末尾ゼロなし）に合わせてハッシュを再生成 |

**注意**: 0017 は 0016 の直後にのみ意味を持つ。0016 を適用した DB には必ず 0017 も適用すること（`cargo sqlx migrate run` は順序通り自動適用するため通常は問題ない）。

---

## 運用ルール

### 禁止事項

- **既存の `.sql` ファイルを編集・削除しない**。SQLx チェックサムが変わり `migrate run` が失敗する。
- **ファイル名の通番を変更しない**。適用順が保証されなくなる。
- **適用済みのテーブルを DROP する migration を追加しない**。本番データが消える。

### 新規 migration 追加手順

```bash
cd backend

# 1. migration ファイルを作成
sqlx migrate add <snake_case_description>

# 2. SQL を記述して保存

# 3. ローカル DB で確認
cargo sqlx migrate run

# 4. オフラインキャッシュを更新（CI/本番のコンパイル向け）
make sqlx-prepare

# 5. Cargo.lock と .sqlx/ をコミット
```

### バージョン番号命名規則

- 4桁連番 + `_` + snake_case 説明（例: `0018_add_index_to_sessions.sql`）
- 説明は「何をする migration か」を動詞から始める（例: `add_`, `drop_`, `alter_`, `create_`, `fix_`）

---

## 検証観点

### 新規 DB 構築時（fresh install）

```bash
# ローカル DB を起動して全 migration を順に適用（backend/ ディレクトリで実行）
cd backend && make db-up
cd backend && cargo sqlx migrate run

# Docker が使える環境では統合テストで全テーブルの作成を確認
# （#[ignore] テストなので --ignored が必要）
cd backend && cargo test db_integration_with_docker_and_migrations -- --ignored --nocapture
```

### 既存 DB 更新時

- `cargo sqlx migrate run` は適用済み migration をスキップし、未適用分のみ実行する
- 適用前後でアプリケーションが正常起動することを確認する（`/health` エンドポイント）
- データ量が多い場合はロック競合に注意し、本番適用は低トラフィック時間帯に行う

---

## 今後の整理 TODO（次の実装タスク候補）

- [ ] `domestic_stocks` の 0012 に記録が残る `import_batch_fingerprint` 列の経緯を集約テストでカバー
- [ ] `asset_balances` テーブルの更新戦略（UPSERT vs DELETE+INSERT）を統合テストで検証
- [ ] `jquants_rate_control` の単一行制約（`id = 1`）を明示するコメントを migration に追記
