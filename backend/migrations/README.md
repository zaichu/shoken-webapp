# backend/migrations

SQLx マイグレーション管理。
**既存の `.sql` ファイルは絶対に編集・削除しない**（SQLx はチェックサムで検証するため、変更すると `cargo sqlx migrate run` が失敗する）。

---

## 現行ファイル一覧（テーブル単位）

| No. | ファイル | 内容 |
|-----|---------|------|
| 0001 | `0001_extensions.sql` | pgcrypto / citext 拡張 |
| 0002 | `0002_stock.sql` | 銘柄マスタ |
| 0003 | `0003_users.sql` | Google OAuth ユーザー管理 |
| 0004 | `0004_sessions.sql` | セッション管理（7日有効期限） |
| 0005 | `0005_dividends.sql` | 配当金取引 |
| 0006 | `0006_domestic_stocks.sql` | 国内株式取引（content_hash / occurrence_index による重複判定） |
| 0007 | `0007_mutualfunds.sql` | 投資信託取引 |
| 0008 | `0008_asset_balances.sql` | 資産残高（保有銘柄） |
| 0009 | `0009_jquants.sql` | J-Quants API キャッシュ・レートコントロール |

---

## 運用ルール

### 禁止事項

- **既存の `.sql` ファイルを編集・削除しない**
- **ファイル名の通番を変更しない**
- **適用済みのテーブルを DROP する migration を追加しない**

### ファイル数を増やさないためのルール

0001〜0009 はテーブルの最終状態を定義している。**テーブル構造を変える場合は ALTER TABLE / CREATE INDEX を新ファイルに追記する**。
「新しいカラムを既存ファイルに書き込む」ことはしない（SQLx チェックサムが変わるため）。

次の migration を追加するときは、変更目的を 1 つに絞ること。複数の目的を 1 ファイルに混在させない。

### 新規 migration 追加手順

```bash
cd backend

# 1. migration ファイルを作成
sqlx migrate add <snake_case_description>

# 2. SQL を記述して保存

# 3. ローカル DB で確認
cargo sqlx migrate run

# 4. (任意) SQLx クエリを含む変更の場合はオフラインキャッシュを更新
make sqlx-prepare
git add .sqlx/ Cargo.lock
```

### バージョン番号命名規則

- 4桁連番 + `_` + snake_case 説明（例: `0010_add_index_to_sessions.sql`）
- 説明は「何をする migration か」を動詞から始める（例: `add_`, `drop_`, `alter_`, `create_`, `fix_`）

---

## 過去の migration 履歴

旧 `0001〜0017` は `backend/migrations_legacy/` に保管している。
設計変遷の経緯は `backend/migrations_legacy/` 内のファイルを参照すること。

---

## 検証

```bash
# ローカル DB への適用
cd backend && make db-up && make migrate-local

# Docker 統合テスト
cd backend && cargo test db_integration_with_docker_and_migrations -- --ignored --nocapture
```
