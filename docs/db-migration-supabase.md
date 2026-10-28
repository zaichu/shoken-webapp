# Neon から Supabase へのデータ移行

Issue #1568 の本番移行手順。2026年11月1日に Neon の月間計算枠が戻り、読み書きできることを確認してから書き出す。
データを移さず空の Supabase で始める場合は、`prepare` → Hyperdrive の付け替え(§4) → GitHub の `DATABASE_URL` の更新(§4b) → 本番確認(§5)の短い手順で始める。今回はこの短い手順で復旧した。
旧データを移す場合だけ、順序は **Supabase に空のテーブルを作る → データを入れる → 全テーブルの件数・主キーを照合する(§3) → Hyperdrive を付け替える → 本番を確認する**。
Neon は本番確認と戻しの必要性の判断が終わるまで削除しない。

この手順書のコマンドは、wrangler 4.155.0 の `npx wrangler hyperdrive update --help` と `npx wrangler hyperdrive list --help` で確かめたものだけを書く。確かめていないオプションは足さない。

## 担当と秘密情報

| 担当 | 行うこと |
| --- | --- |
| ユーザー | Supabase プロジェクト作成、接続方式の選択、秘密情報の環境変数への入力、ユーザーの端末での本番コマンド実行、Hyperdrive の付け替え・戻し |
| 統合担当 | 実装者以外によるレビュー、作業時間と書き込み停止の調整、照合結果の確認、付け替え・再開・戻しの判断、本番確認 |
| 実装担当 | スクリプト・手順書の保守、架空データによる Docker 検証、秘密情報を含まない結果の報告 |

実装担当・統合担当には接続情報を渡さない。本番の操作は、ユーザーが管理する端末で行う。
接続文字列・パスワード・鍵の値は、ファイル、チャット、PR、Issue、ログ、画面撮影に残さない。
シェルは `set +x` を維持する。`env`、`printenv`、デバッグ出力、端末の録画、DB クライアントのエラー原文の共有をしない。

接続値は `SOURCE_DATABASE_URL`（Neon）と `TARGET_DATABASE_URL`（Supabase）の環境変数だけで渡す。
既存のパスワード管理ツールから環境に注入するか、Bash の非表示入力を使う。値をコマンド行や `.env` に書かない。

```bash
set +x
read -r -s -p 'SOURCE_DATABASE_URL: ' SOURCE_DATABASE_URL
export SOURCE_DATABASE_URL
read -r -s -p 'TARGET_DATABASE_URL: ' TARGET_DATABASE_URL
export TARGET_DATABASE_URL
```

スクリプトは URL をメモリ内で libpq の環境変数へ分解する。`psql` / `pg_dump` の引数には接続値を渡さず、標準エラーを破棄する。
DB の応答も端末へ出さず、終了コードと固定の完了・失敗メッセージだけを返す。失敗時に `set -x` で再実行しない。
URL のクエリパラメーターは `sslmode`、`channel_binding`、`sslrootcert` に対応する。本番は TLS を必須にする。

## 1. ユーザー: Supabase の用意（11月1日より前に可能）

1. Supabase で無料枠のプロジェクトを作る。Specific region の **Northeast Asia (Tokyo)、`ap-northeast-1`** を選ぶ。APAC の自動選択は東京を保証しない。[リージョン一覧](https://supabase.com/docs/guides/platform/regions)
2. このアプリは Worker の認証を通して DB を使う。テーブルを作る前に Supabase の Data API を無効化し、`public` を REST / GraphQL から公開しない。プロジェクト作成時に Data API・自動公開をオフにする。[Data API の保護](https://supabase.com/docs/guides/api/securing-your-api)
3. Connect で **Session pooler** を選ぶ。共有プーラーのセッションモードは IPv4 で利用でき、ポートは 5432。ユーザー名は `postgres.<識別子>` の形式で、Session pooler の表示に従う。Direct connection(`db.<識別子>.supabase.co`)は IPv6 のみで使えないため、この手順では使わない。Transaction pooler の 6543 はこの手順では使わない。[接続方式](https://supabase.com/docs/guides/database/connecting-to-postgres)
4. Session pooler の値を `TARGET_DATABASE_URL` に非表示入力する。URI 内の特殊文字は URL エンコードする。パスワードは英数字だけにする。パスワードのある画面を共有しない。
5. Python 3、`pg_dump` / `psql`、Docker、Wrangler を用意する。`pg_dump` はサーバーと同じか新しい版が必要になる。Supabase は PostgreSQL 17 だったので 17 以上を用意する。版が古いと書き出しに失敗するため、事前に版を確認する。PostgreSQL 16 のローカルテストと、本番のサーバーバージョン互換性の確認は別に扱う。

## 2. ユーザーが実行、統合担当が結果確認: 空のテーブルを作る

リポジトリルートで実行する。`TARGET_DATABASE_URL` は新規 Supabase に限る。

```bash
mkdir -m 700 .db-migration
python3 -B scripts/db_migration.py prepare
```

`prepare` は `public` にテーブルがない場合だけ実行できる。0000〜0017 を順に1トランザクションで適用し、
backend と互換の SHA-384 チェックサムを `_sqlx_migrations` に記録する。
既存の `citext` / `pg_trgm` / `pgcrypto` は、マイグレーションが参照する `public` に配置する。
アプリのテーブルは空になるが、`fund_codes` と `market_data_provider_rate_control` にはマイグレーションの初期行が入る。
Supabase 管理用の `auth` / `storage` などのスキーマには触らない。

## 3. 旧データを移す場合だけ: 11月1日にユーザーが実行、統合担当が書き込み停止を確認

1. Neon の枠が戻り、DB に接続できることを確認する。失敗したまま Supabase に付け替えない。
2. 統合担当がメンテナンス時間を確保する。ユーザー操作による CSV・更新・ログインに加え、Worker の Cron、外部 API のキャッシュ更新、進行中のリクエストなど、**両 DB への全書き込みを止める**。実行中の書き込みが完了してから書き出す。停止を保証できなければ作業を進めない。
3. `SOURCE_DATABASE_URL` に Neon の値を非表示入力する。Neon と Supabase を逆にしていないことはユーザー自身で確認する。
4. リポジトリルートで順に実行し、各コマンドが成功してから次へ進む。

```bash
python3 -B scripts/db_migration.py export .db-migration/neon-data.sql
python3 -B scripts/db_migration.py import .db-migration/neon-data.sql
python3 -B scripts/db_migration.py verify .db-migration/neon-data.sql
```

同じ処理は `scripts/db-migration.sh` の同名サブコマンドからも呼び出せる。ラッパーは先頭で `set +x` にする。
書き出しファイルは 0600、保存ディレクトリは 0700 とし、Git の対象外にする。
ファイルの中身や DB の行をエージェントに見せない。検証・開発には架空データだけを使う。

### 対象・順序・照合

| 対象 | 扱い |
| --- | --- |
| `public` の全15テーブル | `pg_dump --data-only --schema=public --no-owner --no-privileges` で単一スナップショットを書き出す。所有者・権限・DDL は持ち込まない |
| 外部キー | `pg_dump` の依存順序で親の `users` から子を読み込む。制約・トリガーの無効化や superuser 権限を使わない |
| `fund_codes`、`market_data_provider_rate_control` | 移行先の初期行を移行元の全行で置き換える。キャッシュ、価格、レート制御状態も保持する |
| `_sqlx_migrations` | 18件の履歴を含めて移す。読み込み前に両 DB の version / checksum / success とテーブル・主キー一覧が一致することを確認する。不一致なら中止し、統合担当が履歴の互換性を調査する。履歴を偽装して続行しない |
| `csv_import_history` | `(user_id, kind)` の複合主キーと最終取り込み日時をそのまま保持する |
| `sessions` | ダンプには全行を含めるが、読み込みと同じトランザクションで期限切れを削除する。読み込み開始時点の移行先 DB 時刻を基準とし、その時点で有効な行だけをコミットする。トークンハッシュ・ID・期限を再生成しない |
| シーケンス | 現行0000〜0017にはアプリ用シーケンスはない。存在する場合は `pg_dump` の `setval` で現在値と `is_called` を保持し、照合する。Docker テストでは専用シーケンスでも確認する |
| ビュー・生成列 | ビューと関数はマイグレーションで作る。`asset_balances_view` / `fund_balances_view` はデータを持ち込まず、元テーブルを読み込んで再計算する。生成列も移行先で計算する |

読み込みは全テーブルをロックし、業務データが空であることを確認してから、初期行を含めて全行を置き換える。
業務データがある移行先への追記・再読み込みは拒否する。読み込み中に失敗した場合、テーブルの変更はすべてロールバックする。
シーケンスの `setval` はロールバックされないため、失敗後は専用の移行先を作り直してやり直す。

読み込みに成功すると `.db-migration/neon-data.checkpoint`（0600）にセッション判定時刻だけを保存する。
`verify` は、両 DB の全テーブルについて主キー集合・全列の行集合を比較するので、件数の不一致も検出する。
`sessions` の移行元だけを判定時刻で絞り、期限切れによる意図した減少を除いて一致を求める。
シーケンスの一覧・現在値・`is_called` も一致が必要。**終了コード0のときだけ**統合担当が付け替え可と判断する。
照合が終わるまで両 DB の書き込みを再開しない。照合後に Neon へ書き込んだ場合は、新しいダンプでやり直す。

失敗・不一致時は Hyperdrive を変更しない。元の Neon とダンプを保持し、原因を接続可否・権限・バージョン・マイグレーション履歴・書き込み停止の条件から確認する。
DB クライアントの生エラーは表示しない。再実行時は新しいファイル名を使い、読み込み先を新規に用意する。既存ファイルは上書きしない。

## 4. ユーザーだけが実行: Hyperdrive の付け替え

`backend/wrangler.toml` の本番 binding の ID は `00000000000000000000000000000000` の仮値で、実IDはリポジトリに置かない。実IDは `npx wrangler hyperdrive list` の `shoken-backend-db` の行で確認する(秘密ではない)。デプロイ時は `.github/workflows/deploy-cloudflare-worker.yml` の Ensure Hyperdrive config exists と同じく名前で検索して ID を解決している。
`TARGET_DATABASE_URL` は Supabase Session pooler のまま使う。Wrangler の認証はユーザーの端末で済ませる。TLS は接続先が必須で既定になるため、`--sslmode` は付けない。実際に成功した手順(10/29、ユーザーが実行して Hyperdrive が Supabase を指すようになった)も付けずに実行した。

```bash
set +x
npx wrangler hyperdrive list
```

一覧から `shoken-backend-db` の行の ID を確認し、設定する。`wrangler.toml` の仮値は使わない。

```bash
set +x
read -r -p 'HYPERDRIVE_ID: ' HYPERDRIVE_ID
export HYPERDRIVE_ID
```

照合成功後(空の Supabase で始める場合は `prepare` 成功後)、ユーザーが `backend` で次の1コマンドを実行する。接続文字列・パスワードは画面に出さず、URL とパスワードらしい部分を伏せて表示し、固定の終了コードだけを確認する。

```bash
cd backend
WRANGLER_LOG=none WRANGLER_WRITE_LOGS=false WRANGLER_SEND_METRICS=false WRANGLER_SEND_ERROR_REPORTS=false npx wrangler hyperdrive update "$HYPERDRIVE_ID" --connection-string "$TARGET_DATABASE_URL" 2>&1 | sed -E 's#[A-Za-z]+://[^ ",]+#<URL伏せ>#g; s#(password"?[ :=]+)[^ ,]+#\1<伏せ>#Ig' | head -30; echo "終了コード: ${PIPESTATUS[0]}"
```

このコマンドの本番での実行はユーザーが行う。成功するまで書き込みは停止したままにする。`--help` で確かめていないオプションは足さない。秘密値を引数に付けたままの表示やデバッグログへ切り替えず、失敗時は固定メッセージだけで実装担当へ調査を依頼する。

付け替えの確認は、同じく `npx wrangler hyperdrive list` で行う。`shoken-backend-db` の行の host・user・database が移行先(Supabase Session pooler)になっていること。

### 4b. ユーザーだけが実行: CI の `DATABASE_URL` secret の切り替え

CI のマイグレーション（`.github/workflows/deploy-cloudflare-worker.yml:189` の `cargo run --bin migrate` が `backend/src/bin/migrate.rs:6` の `DATABASE_URL` を使う）は GitHub secret の `DATABASE_URL` を接続先にする。Hyperdrive だけを付け替えても次のバックエンドデプロイは Neon に接続し、新しいマイグレーションは Supabase に適用されない。照合成功(空の Supabase で始める場合は `prepare` 成功)・Hyperdrive 付け替え後に、ユーザーが GitHub のリポジトリ設定で `DATABASE_URL` secret を Supabase Session pooler の値へ更新する。値は画面に表示せず、パスワード管理ツールからの貼り付けか非表示入力で扱い、チャット・PR・Issue・ログに残さない。安定動作の確認が終わるまで Neon の値はパスワード管理ツールに残し、切り戻し時は同じ secret を実際の接続先に合わせて戻す。

migrate バイナリは `DATABASE_URL` の `sslmode` を libpq と同じ意味で扱う。Supabase の Session pooler は独自 CA の証明書を提示するため、OS の信頼ストアだけでは検証できない。CI の secret には次のいずれかを付ける。`sslmode` を付けない既定(検証あり)のままでは接続できない。`sslrootcert` は tokio-postgres が解釈できないため migrate バイナリ側で読み替える。検証したい場合は Dashboard の Database settings からサーバ CA を取得し `sslmode=verify-full` と `sslrootcert` の CA ファイルを組み合わせる。

- 暗号化のみで接続する: `sslmode=require`
- 検証して接続する: `sslmode=verify-full` と `sslrootcert` の CA ファイル

## 5. 統合担当とユーザー: 本番確認と再開

1. Worker 経由の `/health` と、実際に DB を問い合わせる `/ready` が200になることを確認する。`/health` だけでは DB 接続成功の判断にしない。
2. ユーザー自身が既存ログインと新規ログイン、配当・取引・保有一覧、口座別残高、CSV の最終取り込み日時、価格の表示を確認する。実データの行や画像は共有せず、成功・失敗だけを報告する。
3. 統合担当が読み取り結果を確認後、架空の検証用アカウントで更新・CSV 取り込み・再読み込みを確認する。本物のユーザーデータを検証のために更新しない。
4. Worker の定期処理とユーザーの書き込みを再開する。`/ready` と DB エラーの発生件数を確認し、本文や接続値を含むログは共有しない。
5. 安定動作と戻し不要をユーザー・統合担当が確認するまで Neon を保持する。Neon の削除は Hyperdrive と CI の `DATABASE_URL` secret の両方が Supabase を指し、本番確認が終わってから判断する。ダンプとチェックポイントはユーザーが安全な保管場所で扱い、確認後に端末から削除する。接続環境変数も `unset SOURCE_DATABASE_URL TARGET_DATABASE_URL` で破棄する。

## 戻し方

- **付け替え前に失敗**: Hyperdrive は Neon のまま。移行先に付け替えず、Neon の状態を確認してから書き込みを再開する。
- **付け替え後、Supabase にまだ書き込んでいない**: 両 DB の書き込み停止を維持し、ユーザーが `TARGET_DATABASE_URL` を元の Neon の値で非表示入力し直す。同じ Hyperdrive 更新コマンドで元へ戻し、CI の `DATABASE_URL` secret も Neon に戻して実際の接続先と揃える。`/ready` とログイン・一覧を確認して再開する。
- **Supabase に書き込み済み**: 古い Neon にそのまま戻すと新しいデータを失う。両 DB の書き込みを止め、Supabase を `SOURCE_DATABASE_URL`、新規の戻し先 Neon を `TARGET_DATABASE_URL` に非表示入力する。この手順の `prepare` → `export`（新しいファイル名）→ `import` → `verify` を逆方向に行い、一致後に同じ Hyperdrive 更新コマンドで戻す。CI の `DATABASE_URL` secret も戻し先に合わせて更新する。既存のデータ入り Neon への読み込みはしない。戻し先を確保できなければ停止を保ち、統合担当が判断する。

Neon と Supabase は、戻しの確認が済むまでどちらも削除しない。

## 実装担当: ローカルでの検証

PostgreSQL 16 のクライアントと Docker を使い、リポジトリルートで実行する。

```bash
shellcheck scripts/db-migration.sh
python3 -B -m unittest discover -s scripts/tests -p test_db_migration.py -v
```

`backend/tests/db_integration.rs` と同じ `postgres:16-alpine` を起動し、ランダムポートと別名の2 DB を使う。
両 DB に0000〜0017を適用し、全15テーブルに架空データを入れ、書き出し → 読み込み → 全テーブルの件数・主キー・全列一致を確認する。
期限切れセッションの除外、複合主キー、生成列と2つのビュー、シーケンスの次の値、既存データへの再読み込み拒否、
途中失敗時のロールバック、履歴不一致の拒否、同じ件数でも主キー・値が違う場合の検出、接続エラーの出力抑止、既存ファイル保護も確かめる。
実在のユーザーデータ、本番 DB、手元の E2E は使わない。コンテナと一時データはテスト終了時に削除する。
