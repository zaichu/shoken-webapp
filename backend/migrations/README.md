# backend/migrations

migration ファイルの置き場所と運用メモ。
ファイル形式と `_sqlx_migrations` テーブルは sqlx 互換だが、実行は自前ランナー
（`backend/src/bin/migrate.rs` = `backend/src/db/migrate.rs` の `run_migrations`）で行う。
新規ファイルを `MIGRATION_FILES`（`backend/src/db/migrate.rs`）に `include_str!` で
登録しないと適用されない。

## 現行方針

現在の schema はテーブルごと1ファイルの baseline 群(`0000〜0010`)だけで表す。
旧 `0001〜0017` と増分をすべて適用した結果(= 現行 schema)の
`pg_dump --schema-only` 出力から所有者・権限・SET・コメント行を除き、
テーブル単位に分割して生成した(手書きではない)(#1358)。
`pg_dump --schema-only` に含まれない初期行
(`market_data_provider_rate_control` / `security_prices` の INSERT)は
対応するテーブルのファイル末尾に追記してある。

テーブルの意図・関係(なぜこの分割か、共有/個人の分け方)は
`docs/db-design.md` を参照。

- `0000_extensions.sql`: 共通拡張(citext / pg_trgm / pgcrypto)
- `0001_users.sql` 〜 `0006_asset_balances.sql`: 外部キーで参照される側
  (`users`)から番号順
- `0007_stock.sql` 〜 `0010_security_prices.sql`: 外部キーを持たない独立テーブル。
  `0009` にレート制御の seed、`0010` に保有残高からの価格 backfill 用 INSERT と
  `asset_balances_view`(`asset_balances` と `security_prices` の両方に依存するため
  後に作られる側のファイルで定義)を持つ

## _sqlx_migrations 履歴の扱い

baseline 張り直し前に migrate を動かしていた DB のうち、旧 baseline + 増分の
履歴 `0〜18`(#1351 以降の DB)が残っているものは、migrate 実行時に baseline
`0〜10` の記録へ置き換える(一度だけ。テーブル・データには触れない)。
旧履歴かどうかは version=1 の checksum が現行 `0001_users.sql` と異なるかで
判定する。

旧運用履歴 `1〜17`(初回 baseline 化の前の DB)は**エラーで停止する**。
この状態の DB には `security_prices` テーブルも `asset_balances_view` も無い
(schema が現行と違う)ため、rotate では修復できない。`make db-reset` で
DB を作り直す必要がある。途中までしか適用されていない履歴も同様にエラーで
停止する。

## デプロイ時（Cloudflare Workers）の動作：自動適用

デプロイ時の migration 適用は自動。手動実行は不要。

- `deploy-cloudflare-worker.yml` の deploy ジョブが `cargo run --bin migrate`（`backend/src/bin/migrate.rs`）を Neon へ直接実行してから `wrangler deploy` する
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

baseline(`0000〜0010`)は適用済み環境があるため編集しない。
`0011_<description>.sql` のように次番号の新規 migration を追加し、
`backend/src/db/migrate.rs` の `MIGRATION_FILES` に `include_str!` エントリを足す。

検出は正規表現ベースのため、SQL 文字列リテラルやコメントアウトした SQL の中で
誤検出があり得る。逆に検出漏れ(例: `DROP` 以外の手段で列を実質消す手口)もある。
そういう場合には `-- contract:` 印(意図した縮退のとき)で回避できる。

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

## baseline の再生成(baseline ファイル群を張り直す場合)

新規 DB に全マイグレーションを適用した状態に対して `pg_dump --schema-only` を取り、
SET・`SELECT pg_catalog.set_config`・コメント・`OWNER TO`・権限行を除いて整形し、
テーブル単位に分割する。`--schema-only` にはデータが含まれないため、必要な
初期行 INSERT は別途対象テーブルのファイル末尾に追記する。

(#1358 の張り直しもこの手順で行った。旧増分の内容は baseline に畳まれているため
増分ファイルは削除する。)

```bash
cd backend && make db-up   # ローカル PostgreSQL (docker-compose)
cargo run --bin migrate
pg_dump --schema-only "$DATABASE_URL"
```

## 検証

```bash
cd backend && cargo fmt --check
cd backend && cargo clippy --all-targets -- -D warnings
cd backend && cargo test
cd backend && cargo test --test db_integration -- --ignored --nocapture
```
