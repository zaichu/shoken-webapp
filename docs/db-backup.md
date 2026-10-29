# DB バックアップ

Supabase の DB を毎日 artifact に残す。Neon 時代の反省 (データを取り出せなかった) を受け、保存先は「GitHub に暗号化して保存」(案A)。

## 仕組み

- `.github/workflows/db-backup.yml` が毎日 JST 03:00 (UTC 18:00) と手動 (`workflow_dispatch`) で実行する
- `scripts/db-backup.sh` が `pg_dump --schema=public` の出力をパイプで `gzip` → `age` 暗号化し、`*.sql.gz.age` だけを `actions/upload-artifact` で 90 日保存する
- 書き出すのは public だけ。`_sqlx_migrations` は public にあるので残る。auth・storage・realtime・vault などの Supabase の管理用スキーマは含めない。復元先の Supabase に元からあるものと衝突させないため
- 平文の dump を artifact・ログ・キャッシュ・リポジトリに出さない。暗号化できなければ失敗する
- 公開鍵は `.github/backup-recipient.txt` (age の公開鍵 1 行)。秘密鍵はリポジトリにも CI にも置かない
- 公開鍵のファイルが無い・空・形式が違うときは、バックアップせずに失敗する
- 接続は `secrets.DATABASE_URL` (末尾 `?sslmode=require`) を、そのステップの `env: PGURL` だけで渡す。`pg_dump` のエラー本文は接続文字列が混ざり得るため、固定の文言に置き換える
- `pg_dump` は PostgreSQL 17 以上が必要 (Supabase が 17)。PGDG の apt で `postgresql-client-17` を入れる

## 秘密鍵の保管

- ユーザーが保管する。失くすと復元できない
- 秘密鍵をリポジトリ・CI・artifact に入れない

## 復元手順

書き出すのは public だけ。復元は、空の Supabase に `prepare` (0000〜0017) で空のテーブルを作ってから、データだけ入れる。
`prepare` の使い方は `docs/db-migration-supabase.md` の「空の Supabase で始める」手順を使う
(`TARGET_DATABASE_URL` だけを渡して `python3 -B scripts/db_migration.py prepare` を実行する)。
バックアップ内の `CREATE TABLE` など定義は入れ直さず、データ部分だけを流し込む。Supabase の管理用スキーマには触らない。

```bash
# 1. artifact をダウンロード・展開し、復号する (検証用の空 DB にそのまま流し込む場合)
#    --schema=public の dump には CREATE SCHEMA public が含まれるが拡張の定義は含まれないため、
#    検証用の空 DB では先に 3 つの拡張を入れてから、スキーマ作成だけ IF NOT EXISTS に読み替えて流し込む
#    (平文をディスクに残さないようパイプで渡す)
age -d -i <秘密鍵ファイル> db-backup-YYYYMMDDTHHMMSSZ.sql.gz.age | gunzip -c | sed 's/^CREATE SCHEMA public;/CREATE SCHEMA IF NOT EXISTS public;/' | psql "<新しい DB の接続文字列>"
# 事前に検証用 DB へ: CREATE EXTENSION IF NOT EXISTS citext WITH SCHEMA public;
# pg_trgm・pgcrypto も同様 (0000_extensions.sql と同じ)

# 2. 件数を照合する (例: 全テーブルの件数)
psql "<新しい DB の接続文字列>" -c "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public' ORDER BY table_name"
```

## Supabase 側の準備

- `pg_dump` に PostgreSQL 17 以上のクライアントを使う (PGDG の apt で `postgresql-client-17`)
- `DATABASE_URL` は Session pooler の接続文字列 (末尾 `?sslmode=require`) を GitHub secrets に入れる

## 実際に復元を試す手順

```bash
# 1. テスト用の空 DB を用意する (ローカルの Docker など)
# 2. 上の復元手順で書き戻す
# 3. 全テーブルの件数が本番と一致することを確かめる
```

自動テスト (`backend/tests/db_backup.rs`) が、ローカルの Docker (`postgres:17`) で
ダミーデータ → 書き出し → 暗号化 → 復号 → 別の DB に復元 → public の全テーブルの件数一致、
テスト用の別スキーマが書き出しに含まれないこと、および公開鍵が無いときの失敗を確かめる。

```bash
# 手元で回すときは pg_dump/psql 17・gzip・age/age-keygen・Docker が要る
cargo test --test db_backup -- --ignored
```
