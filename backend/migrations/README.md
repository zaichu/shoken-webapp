# backend/migrations

migration ファイルの置き場所と運用メモ。
ファイル形式と `_sqlx_migrations` テーブルは sqlx 互換だが、実行は自前ランナー
（`backend/src/bin/migrate.rs` = `backend/src/db/migrate.rs` の `run_migrations`）で行う。
新規ファイルを `MIGRATION_FILES`（`backend/src/db/migrate.rs`）に `include_str!` で
登録しないと適用されない。ランナーは 1 ファイル 1 トランザクションで番号順に当てる。

## 構成

現在の schema は、役割ごとの土台(baseline)`0100〜0106` で表す。1つの表の定義
(列・制約・索引・コメント・RLS・ポリシー)は、その表のファイルの1か所で読める。
番号は依存の順(先に作るもの)で振ってある。

| ファイル | 中身 |
| --- | --- |
| `0100_common.sql` | 拡張(citext / pg_trgm / pgcrypto)、アプリのロール(`app_restricted`・`app_worker`・限定関数の所有ロール)と所属、`app_enter(uuid)` |
| `0101_users_and_auth.sql` | `users`・`sessions`・`login_events`・`admin_login_views`・`login_notify_tokens` |
| `0102_transactions.sql` | `dividends`・`domestic_stocks`・`mutualfunds`・`csv_import_history` |
| `0103_shared_masters_and_caches.sql` | `stock`・`security_prices`・`fund_codes`(seed 付き)・`fund_prices`・`dividend_per_share_cache`・`market_data_provider_rate_control`(初期行付き)と、ファンド名の照合関数 |
| `0104_holdings.sql` | `asset_balances`・`fund_balances` と、共有の価格を合成するビュー(`security_invoker`) |
| `0105_grants.sql` | ロールごとの表・ビュー・列・関数への GRANT |
| `0106_limited_functions.sql` | 認証と定期処理・共有表の書き込みの限定関数(`SECURITY DEFINER`)と、その所有者・EXECUTE |
| `0107_session_idle_timeout.sql` | セッションの無操作期限付き認証関数と旧 Worker 用互換関数 |
| `0108_operation_logs.sql` | 操作記録 |
| `0109_drop_compat_auth_functions.sql` | 旧 Worker 用一引数認証関数の削除 |

その後の増分 migration は、セッション期限や互換性のための変更を番号順に適用する。
`0109_drop_compat_auth_functions.sql` は旧 Worker 向けの一引数認証関数2つを削除する。

`archive/` は土台に畳む前の旧 migration `0000〜0037`。ランナーは読まない(本番の履歴の照合用の
digest と、旧ファイルを使う一部のテストだけが参照する)。書き換えない。

テーブルの意図・関係(なぜこの分割か、共有/個人の分け方)は `docs/db-design.md` を参照。

migration を変えたら `scripts/gen-db-docs.sh` を実行して `docs/db/` をコミットする
(テーブル一覧と ER 図は `docs/db/README.md` に自動生成される)。

## _sqlx_migrations の履歴と土台

`run_migrations` は `_sqlx_migrations` の履歴を見て次のように動く。履歴の行を書き換え・削除はしない。

| DB の状態 | 動き |
| --- | --- |
| 空(新しい DB・テスト・Docker) | 土台 `0100〜0106` とその後の migration を順に当てる |
| 旧 migration `0000〜0037` が全て適用済み(本番) | 版と checksum の組が `archive/` と一致することを確かめ、土台 `0100〜0106` を **SQL を流さずに適用済みとして記録**(checksum は土台ファイルの SHA-384、`execution_time = 0`)してから、その後の migration を当てる |
| 旧 migration が途中まで | 続きは当てられないため止める(`make db-reset` で作り直す) |
| 旧 migration の版・checksum が `archive/` と違う | 土台を記録せずに止める |
| 対応するファイルが無い版、内容が現行ファイルと違う版がある | 状態と対処(`make db-reset`)を示して止める |

土台が旧 migration と同じ schema を作ることは `scripts/check-migration-baseline.sh` で確かめる
(下の「検証」)。記録の判定は `backend/src/db/migrate.rs` の単体テストと
`backend/tests/db_integration.rs` の migrate テストで固定している。

### 本番の最初のデプロイで起きること

1. `deploy-cloudflare-worker.yml` の `Run database migrations` が `cargo run --bin migrate` を実行する。
2. ランナーが advisory lock を取り、`_sqlx_migrations` に `0000〜0037` が揃っていて archive と一致することを確かめる。
3. 1 トランザクションで `0100〜0106` の 7 行を挿入する。表・関数・権限には触らない(DDL もロックも無い)。
4. 未適用の migration は無いので終わり、続けて `wrangler deploy` に進む(Worker のコードは変わらない)。

一致しなかった場合は何も記録せずにエラーで止まり、Worker の配信にも進まない。

#### 0109 の初回デプロイ

0109 は `auth_session_user_by_token(bytea)` と `auth_lookup_user_id_by_session(bytea)` の DROP だけを行う。
0107 で新 Worker は期限付きの複数引数関数へ移行済みで、0109 の前にデプロイが完了している。
デプロイ手順は migration を Worker 配信より先に適用するため、その間に旧 Worker が残っていると
認証呼び出しが失敗する。初回適用前に PR #2100 の本番デプロイ完了と、現行 Worker の版/リクエストログに
一引数関数呼び出しがないことを確認する。0109 適用時の DB 変更はこの2関数の削除のみで、
DROP により関数 ACL も消える。

### 戻すとき

最初のデプロイの後は、本番の `_sqlx_migrations` に `0100〜0106` の行がある。土台を知らないランナー
(この変更より前のコミットや、この変更の revert)は「適用済みマイグレーション 100 に対応するファイルがありません」で止まる。
revert のコミットを足して進めても同じく止まる。

- Worker だけを戻す: Worker の版だけを前に戻し(Cloudflare のダッシュボードか `wrangler rollback`)、
  ランナーは新しいまま進める。DB は変わらない。
- 旧コードを再デプロイする: 再デプロイの前に、移行用の接続で
  `DELETE FROM _sqlx_migrations WHERE version BETWEEN 100 AND 106;` を流す。
  本番では土台の SQL は流れておらず記録だけなので、行を消してもスキーマは変わらない。
  この手順が使えるのは `0108` 以降を本番に当てる前だけ(当てた後は、その版の行と変更が旧コードに無いため戻らない)。

## デプロイ時（Cloudflare Workers）の動作：自動適用

デプロイ時の migration 適用は自動。手動実行は不要。

- `deploy-cloudflare-worker.yml` の deploy ジョブが `cargo run --bin migrate`（`backend/src/bin/migrate.rs`）を本番 DB へ直接実行してから `wrangler deploy` する
- Worker のリクエストパスに migration は置けない（起動という概念がないため）
- 手動 `cargo run --bin migrate` が必要なのは新規 DB・ローカル DB のみ

## 新規 DB

```bash
cd backend
cargo run --bin migrate
```

マイグレーションだけでは `stock` は空のままです。銘柄検索を動かすには
`backend/README.md` の「新規 DB で `stock` を投入する」手順（Worker の
`scheduled` 手動発火）を別途実行してください。

## 新しい schema 変更を入れる場合

- 土台 `0100〜0106` と、本番に当てた migration は書き換えない(本番の記録と checksum が食い違い、デプロイが止まる)。
- `0108_<description>.sql` のように次番号のファイルを足し、`backend/src/db/migrate.rs` の `MIGRATION_FILES` に `include_str!` で登録する。
- 1つの目的ごとに1本にまとめ、細かく分けすぎない(同じ表への列追加・索引・ポリシーは同じファイルに書く)。
- 構造の変更(DDL)と、行数の多い静的データ(seed)は別のファイルに分ける。目安は seed が 50 行を超えるとき。
- `fund_codes` の seed を更新するときは、マーカー(`-- gen:fund-codes-seed begin/end`)付きの新しい migration を作り、`scripts/update_fund_codes_seed.py --migration <そのファイル>` で書く(土台の `0103` は書き換えない)。
- テスト(`backend/tests/db_integration.rs`)と `scripts/check-migration-baseline.sh` は、版の範囲を
  `backend/migrations` のファイルとランナーの `BASELINE_LAST_VERSION` から求めるので、ファイルを足しても直す所はない。
  parity のジョブは、足したファイルを本番と同じ形の DB(archive の後ろ)と新しい DB(土台の後ろ)の両方に当てて比べる。
- 数が増えて読みにくくなったら、下の「土台の作り直し」で畳む。

検出は正規表現ベースのため、SQL 文字列リテラルやコメントアウトした SQL の中で
誤検出があり得る。逆に検出漏れ(例: `DROP` 以外の手段で列を実質消す手口)もある。
そういう場合には `-- contract:` 印(意図した縮退のとき)で回避できる。

## ビューと新しい表の RLS の決まり

- **ビューを作る・変えるときは `WITH (security_invoker = true)` を必ず付ける。**
  付けないビューは所有者(移行ロール)の権限で動き、参照先の表の RLS を迂回する。
  `CREATE OR REPLACE VIEW` を WITH 句なしで書くと、付いていた `security_invoker` も外れる。
- **新しい表には、読み書きするロール向けの RLS ポリシーを同じ migration で必ず付ける。**
  本番(Supabase)には `public` の新しい表へ RLS を自動で有効にする event trigger `ensure_rls` がある。
  ポリシーが無いと、NOBYPASSRLS の `app_worker` / `app_restricted` からはエラーなしで 0 行に見える(旧 0036 の障害)。
  どの環境でも同じ状態になるよう、`ENABLE ROW LEVEL SECURITY` も migration に明示する。
  - 利用者の表は `user_id` を持たせ、`0101`・`0102`・`0104` と同じ式のポリシーにする:
    `USING (user_id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid))`
    (`WITH CHECK` も同じ式)。`user_id::text = ...` のように列側を変換すると索引が使えず、一覧が全行読みになる。
  - 共有の表(利用者の行なし)は `0103` と同じく `FOR SELECT TO app_worker, app_restricted USING (true)` にし、
    書き込みは BYPASSRLS の限定関数(`0106` と同じ定型)に閉じ込める。
  - GRANT は `0105` と同じく、必要なロールに必要な操作だけを与える。
- `FORCE ROW LEVEL SECURITY` は付けない。表の所有者(移行ロール)が BYPASSRLS のため効かない。
  RLS が効く根拠は「アプリのロールが所有者ではなく、所有者の権限も得られず、NOBYPASSRLS であること」で、
  `app_worker` / `app_restricted` を表の所有者にしたり、所有者ロールへ所属させたりしない。
  所有者を NOBYPASSRLS のロールに変えるときは、同じ変更で FORCE を付ける
  (理由と手順は `docs/security/db-rls-plan.md` の §28)。
- ロックを取る DDL(`ALTER POLICY`・`ALTER VIEW ... SET`・`ALTER TABLE` など)の先頭には
  `SET LOCAL lock_timeout = '5s';` を置く(ランナーは 1 ファイル 1 トランザクション)。
  土台は空の DB にしか当たらないため置いていない。

## 破壊的変更(列の削除・改名)は2リリースに分ける

列の削除・改名(破壊的変更)を含む schema 変更は、デプロイの仕組みと組み合わせて
考えると単一のリリースでは安全に実施できない。

**原則**

- 追加(expand)は同じリリースでよい。
- 削除・改名(contract)は「その列を参照しなくなった Worker が本番に出た後の
  リリース」で行う。

**理由**

デプロイは `Run database migrations`(`cargo run --bin migrate`)を実行してから
`wrangler deploy`(Worker 配信)を行う順序で動く。その間は旧 Worker が本番で動き
続ける。破壊的な migration を同じリリースに入れると、この窓で旧 Worker が
削除・改名された列を参照し、列不存在エラーになる。Worker の配信が失敗すると、
その壊れた状態が続く。

**手順(2リリース)**

1. リリース1: コードから列の参照を外す(列は残す)。migration を入れるなら追加のみ。
2. リリース2: 参照が本番から消えた後に、列の削除・改名の migration を入れる。

**例外**

誰も参照していない列、または新規 DB にしか存在しない列は即時でよい。その場合は
PR に理由を書く。

**実例**: [#1355](https://github.com/zaichu/shoken-webapp/pull/1355)

詳細は `.github/workflows/deploy-cloudflare-worker.yml` の `Run database migrations`
ステップのコメントを参照。

## 土台の作り直し

土台の後ろの migration が増えて読みにくくなったら、同じ手順で畳み直す。
畳むのは**本番に当て終わった migration だけ**にする(未デプロイの変更を含めると本番の履歴と食い違う)。

1. 今の土台とその後の migration(本番に当てた最後の版まで)を `archive/` に移す。
   `archive/` には以前の旧 migration も残す(本番の履歴にはそれらの行も残るため)。
2. 新しい土台を、空いている番号帯(例: `0200〜`)に役割ごとに書く。表ごとに列・制約・索引・コメント・
   RLS・ポリシーを1か所にまとめ、GRANT と限定関数は同じ対象への付与の順を旧 migration と同じにする
   (付与の順は ACL の並びとして pg_dump に出る)。
3. `backend/src/db/migrate.rs` を直す: `MIGRATION_FILES` を新しい土台に差し替え、
   `LEGACY_LAST_VERSION` を archive の最後の版、`BASELINE_LAST_VERSION` を新しい土台の最後の版にし、
   `LEGACY_HISTORY_SHA384` を archive 全体から計算し直す(「版の 8 バイト big endian + ファイル本文の
   SHA-384」を版の順に連結した SHA-384。単体テスト `legacy_history_digest_matches_archive` が同じ計算で照合する)。
   2回目以降は archive に `0000〜0037` と `0100〜` の両方が入り、この digest は両方の履歴を照合する。
   そのため、1回目の土台から作った手元の DB(履歴が `0100〜` だけで `0000〜0037` が無い)は旧履歴の照合を通らず、
   新しい土台へは移れない。手元の DB は `make db-reset` で作り直す。
4. `scripts/check-migration-baseline.sh` で、archive を全て当てた DB と新しい土台だけを当てた DB が
   一致することを確かめ、差を1つずつ潰す。
5. 旧ファイルを参照するテスト(`include_str!("../migrations/archive/...")`)のパスを直す。版の範囲はファイルから求めるので直さなくてよい。
6. この README と `docs/` の migration への参照を新しい形に合わせる。

## 検証

```bash
cd backend && cargo fmt --check
cd backend && cargo clippy --all-targets -- -D warnings
cd backend && cargo test
cd backend && make test-ignored          # Docker の結合テスト(login_rls・db_integration など)
scripts/check-migration-baseline.sh      # 本番と同じ形の DB と新しい DB の一致(Docker の postgres:17)
```

`scripts/check-migration-baseline.sh` は使い捨ての Docker Postgres(既定は本番と同じ 17)を立て、
本番と同じ形の DB(archive を順に当て、`BASELINE_LAST_VERSION` より後ろのファイルを足す)と、
新しい DB(`backend/migrations` を全て当てる)の `pg_dump --schema-only`(所有者・権限・RLS・
ポリシー・関数・`security_invoker` を含む)、`pg_dumpall --roles-only`(ロールの属性と所属・付与者)、
seed の行を比べる。移行ロールは superuser と、本番(Supabase)に近い非 superuser の `CREATEROLE` ロールの2通りで比べる。
CI では `Backend DB Integration` の `Migration baseline parity` ジョブで回る。
