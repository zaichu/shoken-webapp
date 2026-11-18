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

書き出すのは public の定義とデータ(`pg_dump --schema=public`)。ロール定義は含まないが、
`OWNER TO`・`GRANT` は含むため、復元先には先にロールを作る。土台の 0100 全体は当てない
(`CREATE FUNCTION app_enter` が dump と重複して失敗することを確認済み)。
`pg_dumpall --roles-only` でロールだけを作り、拡張は別に用意する。
ロール定義はバックアップに含まれないため、復元時は書き出し元 DB から取る
(元 DB が無い場合に備え、必要なら `pg_dumpall --roles-only` の出力を別途保管する)。

```bash
# 1. 復元先にロールと拡張を用意する(復元先の superuser で実行)
pg_dumpall --roles-only -d "<書き出し元の接続文字列>" \
  | grep -vE '^(CREATE ROLE postgres;|ALTER ROLE postgres WITH )' \
  | psql "<復元先の接続文字列>" -v ON_ERROR_STOP=1
#    CREATE/ALTER ROLE postgres の2行は復元先に既にあるため除く(そのまま流すと失敗する)。
#    所属情報は残るため、限定関数の所有ロール(app_*_executor)への OWNER TO・GRANT も復元できる
psql "<復元先の接続文字列>" -v ON_ERROR_STOP=1 \
  -c "CREATE EXTENSION IF NOT EXISTS citext WITH SCHEMA public;" \
  -c "CREATE EXTENSION IF NOT EXISTS pg_trgm WITH SCHEMA public;" \
  -c "CREATE EXTENSION IF NOT EXISTS pgcrypto WITH SCHEMA public;"

# 2. artifact をダウンロード・展開し、復号してそのまま流し込む
#    (平文をディスクに残さないようパイプで渡す。復元先のオブジェクトを所有する復元用ロールで接続する)
{ age -d -i <秘密鍵ファイル> db-backup-YYYYMMDDTHHMMSSZ.sql.gz.age | gunzip -c | sed 's/^CREATE SCHEMA public;/CREATE SCHEMA IF NOT EXISTS public;/'; } | psql "<復元先のオブジェクト所有者となる新しい DB 接続文字列>"

# 3. 件数を照合する (例: 全テーブルの件数)
psql "<新しい DB の接続文字列>" -c "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public' ORDER BY table_name"
```

## Supabase 側の準備

- `pg_dump` に PostgreSQL 17 以上のクライアントを使う (PGDG の apt で `postgresql-client-17`)
- `BACKUP_DATABASE_URL` 未登録時は `DATABASE_URL` が backup に使われる。専用接続へ切り替える場合はユーザーが `BACKUP_DATABASE_URL` を GitHub secrets に登録する。接続先は同じ DB の Session pooler を使う

## 管理用接続への切り替え(ユーザーの作業)

アプリの Workers 接続 (`DATABASE_URL`) は変更しない。`BACKUP_DATABASE_URL` には `backup_operator` ロールでログインする接続文字列を登録する。`postgres` の接続文字列を登録すると、アプリと同じ強い資格情報を CI に保存することになり分離にならない。専用 secret が未登録なら既存 `DATABASE_URL` が使われる。

Supabase SQL Editor で DB owner として次を実行する。`backup_operator` のパスワードは SQL に書かず、Supabase Dashboard の Database Settings でこのログインロールに設定する。設定したパスワードの `backup_operator` 接続文字列を GitHub Actions の `BACKUP_DATABASE_URL` secret に登録する。pooler を使う場合もユーザー部分は `backup_operator` とする。ロールが既存なら作成文を飛ばし、属性と権限を確認する。

```sql
CREATE ROLE backup_operator LOGIN NOINHERIT NOSUPERUSER BYPASSRLS;
GRANT USAGE ON SCHEMA public TO backup_operator;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO backup_operator;
GRANT SELECT ON ALL SEQUENCES IN SCHEMA public TO backup_operator;
ALTER DEFAULT PRIVILEGES FOR ROLE postgres IN SCHEMA public
  GRANT SELECT ON TABLES TO backup_operator;
ALTER DEFAULT PRIVILEGES FOR ROLE postgres IN SCHEMA public
  GRANT SELECT ON SEQUENCES TO backup_operator;
```

`backup_operator` は RLS と FORCE RLS を迂回して完全な dump を取るため `BYPASSRLS` を持つ一方、テーブル所有権や migration 権限は持たず、既存表・シーケンスへの SELECT だけを持つ。default privileges は各オブジェクトを作成するロールごとに必要である。マイグレーション接続が `MIGRATION_DATABASE_URL` の `migration_admin` なら、既存オブジェクトへの GRANT に加え、次を `migration_admin` として実行する。

```sql
ALTER DEFAULT PRIVILEGES FOR ROLE migration_admin IN SCHEMA public
  GRANT SELECT ON TABLES TO backup_operator;
ALTER DEFAULT PRIVILEGES FOR ROLE migration_admin IN SCHEMA public
  GRANT SELECT ON SEQUENCES TO backup_operator;
```

GitHub Actions は URL のログインロールで直接 `pg_dump` を実行する。最初の手動実行で専用 URL による dump が成功することを確かめる。`BYPASSRLS` を持つ URL は Workers の Hyperdrive に設定しない。

復元は空の disposable DB で行い、暗号化 dump を復号して `psql` に流すときは復元先のオブジェクトを所有する復元用ロールで接続する。`backup_operator` は読み取り専用のため復元に使わない。既存スキーマを置換する場合、実行者が対象オブジェクトを所有し、拡張の作成・削除権限も持つことを事前に確認する。復元完了後に全 public base table の件数を照合する。

## 実際に復元を試す手順

```bash
# 1. テスト用の空 DB を用意する (ローカルの Docker など)
# 2. 上の復元手順で書き戻す
# 3. 全テーブルの件数が本番と一致することを確かめる
```

自動テスト (`backend/tests/db_backup.rs`) が、ローカルの Docker (`postgres:17`) で
ダミーデータ → 書き出し → 暗号化 → 復号 → 別の DB に復元 → public の全テーブルの件数一致、
テスト用の別スキーマが書き出しに含まれないこと、復元先に `pg_dumpall --roles-only` で
ロールを作ること、復元した DB でアプリのロール(`app_worker`)の利用者 Tx から利用者の行が読めること、
および公開鍵が無いときの失敗を確かめる。移行ロール(`migration_admin`)による全 migration 適用も、
本番の移行ロールと同じ条件(非 superuser・`CREATEROLE`・`BYPASSRLS`)で確かめる。

```bash
# 手元で回すときは pg_dump/psql 17・gzip・age/age-keygen・Docker が要る
cargo test --test db_backup -- --ignored
```
