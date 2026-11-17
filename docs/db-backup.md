# DB バックアップ

Supabase の DB を毎日 artifact に残す。Neon 時代の反省 (データを取り出せなかった) を受け、保存先は「GitHub に暗号化して保存」(案A)。

## 仕組み

- `.github/workflows/db-backup.yml` が毎日 JST 03:00 (UTC 18:00) と手動 (`workflow_dispatch`) で実行する
- `scripts/db-backup.sh` が `pg_dump --schema=public` の出力をパイプで `gzip` → `age` 暗号化し、`*.sql.gz.age` だけを `actions/upload-artifact` で 2 日保存する
- 書き出すのは public だけ。`_sqlx_migrations` は public にあるので残る。auth・storage・realtime・vault などの Supabase の管理用スキーマは含めない。復元先の Supabase に元からあるものと衝突させないため
- 保存は 2 日。毎日 1 回取るので、常に最新の 1〜2 個が残り、復元には最新の 1 個を使う。失敗したら GitHub のメールで通知が来るので見逃さない。2 日続けて失敗すると、バックアップが無くなる
- 平文の dump を artifact・ログ・キャッシュ・リポジトリに出さない。暗号化できなければ失敗する
- 公開鍵は `.github/backup-recipient.txt` (age の公開鍵 1 行)。秘密鍵はリポジトリにも CI にも置かない
- 公開鍵のファイルが無い・空・形式が違うときは、バックアップせずに失敗する
- 接続は `secrets.BACKUP_DATABASE_URL` を優先し、未登録なら `secrets.DATABASE_URL` にフォールバックする。値は backup ステップの `env: DB_ADMIN_URL` だけで渡す。`pg_dump` のエラー本文は接続文字列が混ざり得るため、固定の文言に置き換える
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
#    (平文をディスクに残さないようパイプで渡す。backup_operator は restore_owner の membership と CREATE 権限を持たせる)
{ printf 'SET ROLE backup_operator;\n'; age -d -i <秘密鍵ファイル> db-backup-YYYYMMDDTHHMMSSZ.sql.gz.age | gunzip -c | sed 's/^CREATE SCHEMA public;/CREATE SCHEMA IF NOT EXISTS public;/'; } | psql "<backup_operator の membership を持つ新しい DB 接続文字列>"
# 事前に検証用 DB へ: CREATE EXTENSION IF NOT EXISTS citext WITH SCHEMA public;
# pg_trgm・pgcrypto も同様 (0000_extensions.sql と同じ)

# 2. 件数を照合する (例: 全テーブルの件数)
psql "<新しい DB の接続文字列>" -c "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public' ORDER BY table_name"
```

## Supabase 側の準備

- `pg_dump` に PostgreSQL 17 以上のクライアントを使う (PGDG の apt で `postgresql-client-17`)
- `BACKUP_DATABASE_URL` 未登録時は `DATABASE_URL` が backup に使われる。管理接続へ切り替える場合はユーザーが `BACKUP_DATABASE_URL` を GitHub secrets に登録する。接続先は同じ DB の Session pooler を使う

## 管理用接続への切り替え(ユーザーの作業)

アプリの Workers 接続 (`DATABASE_URL`) は変更しない。`BACKUP_DATABASE_URL` に backup/restore 専用ロールの接続文字列を登録すると、次の backup から管理接続が使われる。secret が未登録なら既存 `DATABASE_URL` のまま動作する。

Supabase SQL Editor で DB owner として次を実行する。パスワードは SQL に書かず、Dashboard で設定する。ロール名が既存なら作成文を飛ばし、属性と membership を確認する。

```sql
CREATE ROLE backup_operator LOGIN NOINHERIT NOSUPERUSER BYPASSRLS;
GRANT backup_operator TO postgres WITH INHERIT FALSE, SET TRUE;
```

`backup_operator` は RLS 全表を完全に読むため `BYPASSRLS` を持つ。既存のテーブル所有ロールが `postgres` 以外の場合は、その owner にも `backup_operator` への `SET` membership を付ける。GitHub Actions は `pg_dump --role=backup_operator` で接続直後に明示的な `SET ROLE` を行う。接続の `current_user` が `backup_operator` となることを最初の手動実行で確認する。`BYPASSRLS` を持つ URL は Workers の Hyperdrive に設定しない。

復元は空の disposable DB で行い、暗号化 dump を復号して `psql` に流すときも `SET ROLE backup_operator` を実行してから流す。既存スキーマを置換する場合、実行者が対象オブジェクトを所有し、拡張の作成・削除権限も持つことを事前に確認する。復元完了後に全 public base table の件数を照合する。

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
