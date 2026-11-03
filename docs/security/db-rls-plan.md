# DB 権限分離(RLS)計画

Issue #1675(親 #1608)。監査 7-1(中)への対応。現状はアプリの `WHERE user_id` が唯一の境界で、
将来の SQL の書き間違いが他人のデータの露出に直結する。本書は移行の計画の正本とし、
一段階ごとに実際の適用結果を追記する。

## 1. 現状

### 1.1 接続の仕組み

| 経路 | 実装 | 接続先 |
|---|---|---|
| ホスト(migrate bin・DB 結合テスト) | `backend/src/db.rs` の `connect_pool_lazy`(deadpool-postgres、`DATABASE_URL` 直結) | Neon 直結 / ローカル Docker |
| 本番リクエスト | `backend/src/db.rs` の `WorkerDb::from_hyperdrive`(Hyperdrive binding から接続文字列・host・port を取得し `worker::Socket`(StartTls)で接続) | Hyperdrive → Supabase Session pooler(5432、`postgres.<識別子>` 形式) |
| デプロイ時 migration | `cargo run --bin migrate`(`backend/src/db/migrate.rs` の `run_migrations`)を Neon/Supabase へ直接実行してから `wrangler deploy` | `DATABASE_URL`(GitHub secret) |

- プールは利用者間で接続を使い回す。1接続が特定の利用者に紐付くことはない。
- Hyperdrive は接続を束ねる層で、利用者ごとの接続分離はしない。
- Session pooler はセッションモードのため、セッション単位の設定は接続を使い回す他人に漏れる余地がある。
  トランザクションモード(6543)は使わない(`docs/db-migration-supabase.md` §1)。
- 本番 PostgreSQL は Supabase の 17。結合テストは `postgres:16-alpine`(`backend/src/test_constants.rs`)。
- 現在の接続ロールは単一で、テーブルの所有者相当として全表へ読み書きできる。`GRANT`/`POLICY` は migrations 全体に存在しない(監査で grep 済み)。

### 1.2 表と `user_id` の一覧

| テーブル | `user_id` | 分類 |
|---|---|---|
| `users` | 主キーが本人(`id`) | 本人の起点 |
| `sessions` | あり | 本人 |
| `dividends` | あり | 本人(取引明細) |
| `domestic_stocks` | あり | 本人(取引明細) |
| `mutualfunds` | あり | 本人(取引明細) |
| `asset_balances` | あり | 本人(保有残高) |
| `fund_balances` | あり | 本人(保有残高) |
| `csv_import_history` | あり(主キーの一部) | 本人(取込履歴) |
| `login_events` | あり | 本人(監査) |
| `login_notify_tokens` | あり | 本人(通知) |
| `admin_login_views` | あり | 本人(監査) |
| `stock` | なし | 共有(マスタ) |
| `security_prices` | なし | 共有(キャッシュ) |
| `dividend_per_share_cache` | なし | 共有(キャッシュ) |
| `market_data_provider_rate_control` | なし | 共有(レート制御) |
| `fund_codes` | なし | 共有(対応表) |
| `fund_prices` | なし | 共有(基準価額) |
| `asset_balances_view` / `fund_balances_view` | 合成(ビュー) | 読み取り専用 |

利用者データの境界はすべて `user_id` で、共有表には個人の値を書かない(`docs/db-design.md` §共有の3表・§個人値と共有値の合成)。

## 2. 方式の比較

| 方式 | 仕組み | 利点 | 欠点 |
|---|---|---|---|
| RLS + 接続ごとの利用者 ID | `ENABLE ROW LEVEL SECURITY` と `USING (user_id = ...)` ポリシーを本人表に付け、リクエスト開始時に `app.current_user_id` を接続へ渡す | SQL の書き間違いが他人の行に届かなくなる。監査 7-1 の直接の対応 | 接続使い回しのため設定漏れ・設定忘れの経路管理が要る。ポリシー式の誤りは全遮断(使えない)か全開放(意味なし)のどちらかに倒れる |
| ロールの分離(読み取り専用など) | アプリ用に権限を絞ったロールを作り、所有者ロールは migration 専用にする | 書き込み系の事故(`DELETE` の付け忘れなど)を権限で止められる。RLS と併用できる | ロールを増やすと接続文字列の管理・切替時の整合が必要。単独では「他人の行」の分離にならない |
| ビューによる分離 | `security_invoker` ビューやマスク用ビューを読み取り経路に使う | 読み取り列の制限(金額のマスクなど)に有効 | 行の分離の正本にはならない。ビューと実表の二重管理になる |

## 3. 選んだ方式と理由

RLS(行の分離)を主とし、ロールの分離(権限の縮小)を従とする。ビューは列の制限が必要になった時点でのみ検討し、行分離には使わない。

- 監査 7-1 の危険は「他人の行への到達」で、行の分離に効くのは RLS だけ。
- ロールの分離だけでは水平アクセスを止められないため、単独採用はしない。
- 利用者 ID の受け渡しはトランザクション内の `SET LOCAL`(または `set_config(..., true)`)に限定する。
  セッション単位の `SET` はプール・Hyperdrive の接続使い回しで他人に漏れるため使わない。
  `Db::begin` が1接続を掴む構造(`backend/src/db.rs`)のため、トランザクション開始直後の設定で漏れを防げる。

### BYPASSRLS の扱い・FORCE の有無(最優先事項)

- PostgreSQL ではテーブルの所有者とスーパーユーザーは RLS を迂回する(`BYPASSRLS` 相当)。
  現在の単一接続ロールは所有者相当のため、`ENABLE ROW LEVEL SECURITY` だけでは本番の挙動は変わらない。
  第一段階はこの性質を意図的に使い、「RLS を有効にしても今の接続ロールが通る形」にする。
- `FORCE ROW LEVEL SECURITY` は所有者にもポリシーを適用するため、**使わない**。
  専用の非所有者アプリロールへ切り替えて検証が終わるまでは、どの表にも付けない。
- 将来作るアプリ用制限ロールには `BYPASSRLS` を付けない。migration 実行ロールは所有者のままとし、
  アプリのリクエスト経路では使わない。

## 4. 移行の順番

| 順 | 対象 | 理由 |
|---|---|---|
| 1 | `csv_import_history`(本 Issue の第一段階) | 参照が2か所(`services/csv.rs` の upsert、`services/csv/stats.rs` の LEFT JOIN)のみで、金額・数量を持たない。失敗時の実害が小さい |
| 2 | 監査系(`login_events`・`admin_login_views`・`login_notify_tokens`) | 本人表だが認証経路の外側。読み書きの影響範囲が狭い |
| 3 | 取引明細・残高(`dividends`・`domestic_stocks`・`mutualfunds`・`asset_balances`・`fund_balances`) | 金額を持つ中心表。1表ずつポリシー追加→制限ロールでの検証→アプリ側 `SET LOCAL` 配線の順で進める |
| 4 | `sessions`・`users` | 認証の心臓部のため最後。誤ると全利用者が締め出される |
| 対象外 | 共有表(`stock`・`security_prices`・`dividend_per_share_cache`・`market_data_provider_rate_control`・`fund_codes`・`fund_prices`) | 利用者の行がなく RLS の対象外。制限ロールには `SELECT` のみ与える |

各段階は「ポリシー追加(所有者は迂回し挙動不変)→制限ロールでの隔離検証→アプリ側の `SET LOCAL` 配線→`FORCE` なしのまま本番確認」の順で進め、
`FORCE` の検討は全表の配線が終わってから行う。過去の migration は書き換えない(増分のみ)。

## 5. 性能への影響

- ポリシーは行ごとに `user_id` の比較を足すだけの述語で、本人表の既存索引はすべて先頭が `user_id` のため、
  計画の変化は想定しない。段階ごとに `EXPLAIN` で利用者 ID 付き問い合わせの計画が変わらないことを確かめる。
- `SET LOCAL` はトランザクション開始時に1往復増えるが、対象は書き込み系のトランザクションのみに限定する。
  読み取りの単発問い合わせには付けない。
- `csv_import_history` は利用者あたり最大5行(`kind` 固定値)の表で、ポリシー評価の行数は無視できる。

## 6. 止まる危険と戻し方

| 危険 | 向き | 対策 |
|---|---|---|
| ポリシー式の誤りで制限ロールが何も見えない | 使えない側(fail-closed) | 結合テストで制限ロールの読み書きを通してから進める。本番の所有者ロールは迂回するため止まらない |
| ポリシー式の誤りで他人が見える | 見える側(fail-open) | 式は `user_id` の等価比較のみに限定し、制限ロールで他人行の不可視・他人口座での書込失敗をテストで固定する |
| `SET LOCAL` の付け忘れ | 制限ロールでは何も見えない | 付け忘れは「見えない」側に倒れる設計のため、露出にはならない。配線時は該当経路の結合テストを必須にする |
| 所有者ロールの誤った `FORCE` 付け | 全停止 | `FORCE` は全段階完了まで付けないことを本書で固定する |

戻し方は新しい migration で `DROP POLICY` / `DISABLE ROW LEVEL SECURITY` を行う(過去ファイルの書き換えはしない)。
ポリシー追加のみの段階では所有者の挙動が変わらないため、戻しは当該 migration の取り消し1件で足りる。

## 7. テストの方法

- 既存の結合テストの仕組み(`backend/tests/db_integration.rs` の testcontainers 起動)に合わせ、
  Docker 上の使い捨て DB で行う。本番データ・本番 DB は使わない。
- 隔離の検証は `BYPASSRLS` を持たない制限ロールをテスト内で作り、そのロールで接続した上で
  `SET LOCAL app.current_user_id` 相当(`set_config(..., true)` をトランザクション内で実行)を行い、
  他人の行が見えないこと・他人 ID での書き込みが失敗することを確かめる。
- 既存機能の検証は所有者ロールのまま行い、RLS 有効後も読み書きが通ることを確かめる。
- 悪用手順の詳細は書かない(親 Issue の守ること)。

## 8. 第一段階の記録(#1675)

- `backend/migrations/0023_csv_import_history_rls.sql`: `csv_import_history` に RLS を有効化し、
  `user_id` と接続ごとの `app.current_user_id` が一致する行だけを許可するポリシーを追加。
  `FORCE` なしのため現行の所有者ロールの挙動は不変。比較はテキスト同士で行い、不正な値でも型変換エラーにならない。
- 結合テスト(`backend/tests/db_integration.rs`): RLS が有効であること、所有者の既存操作(2利用者の読み書き)が通ること、
  制限ロールでは設定なしで0件・利用者 A の設定で A の行のみ・利用者 B 名義の書き込みが失敗することを固定。
- アプリコードの変更なし(`SET LOCAL` の配線は第3段階で行う)。

## 9. 第二段階の記録(#1691)

- `backend/migrations/0024_audit_tables_rls.sql`: 監査系 3 表(`login_events`・`admin_login_views`・`login_notify_tokens`)に RLS を有効化し、
  各表に `user_id` と接続ごとの `app.current_user_id` が一致する行だけを許可するポリシーを追加。
  3 表とも `user_id` 列を持つことを計画書 §1.2 の表の一覧で確認したため、対象外の表はない。
  `FORCE` なしのため現行の所有者ロールの挙動は不変。比較はテキスト同士で行い、不正な値でも型変換エラーにならない。
- 結合テスト(`backend/tests/db_integration.rs`): RLS が 3 表で有効・`FORCE` なしであること、所有者の既存操作(2利用者のログイン記録・通知トークン・閲覧記録の読み書き)が通ること、
  制限ロールでは設定なしで0件・利用者 A の設定で A の行のみ・他人名義の書き込みが 3 表とも失敗することを固定。
- アプリコードの変更なし(`SET LOCAL` の配線は第3段階で行う)。

## 10. 第三段階(前半)の記録(#1697)

- `backend/migrations/0025_trading_tables_rls.sql`: 取引明細・残高の 5 表(`dividends`・`domestic_stocks`・`mutualfunds`・`asset_balances`・`fund_balances`)に RLS を有効化し、
  各表に `user_id` と接続ごとの `app.current_user_id` が一致する行だけを許可するポリシーを追加。
  5 表とも `user_id` 列を持つことを計画書 §1.2 の表の一覧と各テーブルの migration で確認したため、対象外の表はない。
  `FORCE` なしのため現行の所有者ロールの挙動は不変。比較はテキスト同士で行い、不正な値でも型変換エラーにならない。
- 結合テスト(`backend/tests/db_integration.rs`): RLS が 5 表で有効・`FORCE` なしであること、所有者の既存操作
  (2利用者の一括登録・5表すべての CSV 取り込み・一覧・集計)が通ること、
  制限ロールでは設定なしで0件・利用者 A の設定で A の行のみ・他人名義の書き込みが 5 表とも失敗することを固定。
- 性能の確認: 使い捨て DB(`postgres:16-alpine`、各表 2000 行・利用者2名)で利用者 ID 付きの主な問い合わせ
  (5表の一覧・配当と国内株式の集計)の `EXPLAIN` を取り、0025 適用前後で所有者ロールの計画が一致すること(コストまで同一)を確かめた。
  制限ロールでは `user_id` 先頭の索引を使った取り出しが残るが、RLS の絞り込みで件数の見積もりが控えめになり、
  並び順つきの一覧は索引順の利用から並べ替え併用に変わる。所有者経路(現行の本番経路)の計画は変わらないためこの段階では対応しない。
  制限ロールへの切り替え(`SET LOCAL` 配線)を進める段階で測り直す。
- アプリコードの変更なし(`SET LOCAL` の配線は第三段階の後半で行う)。

## 11. 第三段階(後半)の記録(#1709)

- 配線の場所: `backend/src/db.rs` の `Db::begin_with_user` と `Tx::set_local_user` の2関数に集約し、
  `SET LOCAL` 相当の SQL(`SELECT set_config('app.current_user_id', $1, true)`)はこの1か所だけに書いた。
  `Db::begin` がプール/共有接続の取得と `BEGIN` を担う唯一の漏斗で、トランザクション開始直後に設定すれば
  Hyperdrive の接続使い回しに値が漏れない。セッション単位の `SET` は使わない。
  ハンドラーは `&state.pool` と利用者 ID を渡す従来形のままで、サービス層の入口で `begin_with_user` を開くため、
  各ハンドラーへの散在はない。認証経路のログイン Tx(利用者 ID が upsert 後に確定する2件)は
  `begin` 後に `set_local_user` を呼ぶ。
- 対象の基準: 書き込み経路だけを配線した(一括登録と CSV 取り込み本体・履歴記録・全削除・
  ログイン Tx・通知トークンの発行・通知設定の更新・閲覧記録・セッション削除・利用者削除)。
  いずれももともと1トランザクションか単一文の書き込みで、設定の1往復増は受け入れる。
  読み取り経路(一覧・集計・ログイン履歴と端末・通知設定の取得と回数)は配線を見送った。
  一覧は count・data・summary・facets を並行に投げており、1トランザクションの順次実行に変えると
  取引明細・資産管理の一覧を開くたびに往復が数倍になる。現段階は接続ロールが所有者のままで
  安全性の向上がなく速度だけ落ちるため、接続ロールの切り替え段階で並行を保つ形で配線する。
  配線しない読み取り以外の判断は従来どおり: 認証前に利用者が定まらないもの
  (`select_user_by_session`・`select_user_id_by_session`・トークンによる `peek_token`・`consume_token`)、
  利用者をまたぐ管理者横断ビュー(`admin::search`)、利用者 ID を受け取らない共有表専用経路
  (銘柄検索・配当利回りキャッシュ・マスタ更新・価格同期)、利用者のない cron 経路(`prune_login_events`)。
  `sessions`・`users` に触る書き込みは RLS 対象外(第四段階)のため今は迂回するが、同じ配線に寄せてある。
  未配線の迂回路として残っていた `Domain::delete_rows` は削除し、`bulk::delete_all` を Tx 経由に変えた。
- 往復回数(1リクエストあたり、Workers → Hyperdrive → Supabase の想定):
  一覧は変更前後とも count・data・summary・facets の並行問い合わせのままで増減なし。
  書き込みは設定の1往復増のみ(一括登録は `BEGIN`+設定+従来文+`COMMIT`、単一文の書き込みは
  `BEGIN`+設定+本文+`COMMIT` の4往復)。読み取りを各クエリごとの短い Tx で並行に投げる案は、
  一覧1回で約8クエリ×4往復と接続の掴み直しが増えるため採らなかった。
- 結合テスト(`backend/tests/db_integration.rs` の `rls_set_local_wiring_scopes_authenticated_paths`):
  制限ロールで書き込み経路(CSV 取り込みと履歴・残高置換・トークン発行・設定更新・閲覧記録・全削除)を通し、
  本人の行だけが書けること、別利用者の値が tx をまたいで残らないこと(`current_setting` が未設定か空文字)、
  未認証経路(プレビュー・未知セッションとトークンの参照と消費)が設定を残さないことを固定する。
  読み取りは所有者側で従来どおりの行が返ることだけ見る。
- 性能の確認: 使い捨て DB(`postgres:16-alpine`、`dividends` に利用者2名×2000 行)で配線相当の経路
  (`BEGIN`→`set_config`→問い合わせ→`ROLLBACK`、バインド変数形式)で `EXPLAIN` を取った。
  所有者ロールは `idx_dividends_user_settlement_date_id` の索引走査のままで、0025 適用前後・配線前後とも計画は変わらない。
  制限ロールでは並び順つき一覧が並べ替え併用(`Sort` + `Bitmap Heap Scan`、件数の見積もり 10 行)になり、
  §10 の申し送りどおりだった。見積もりずれの原因は `current_setting()` が不透明な安定関数で選択率が既定値になること。
  読み取りの配線時に実量とともに測り直す。
- 見直しの検討: ポリシーを `user_id = current_setting(...)::uuid` の形に変えると一覧は索引走査に戻り
  (式が1回きりの絞り込みになる)、見積もりも正しくなることを使い捨て DB で確認した。ただしこの形では
  `COMMIT`/`ROLLBACK` 後に接続へ残る空文字のプレースホルダが未設定経路の問い合わせをエラーにする
  (現行のテキスト比較は不一致で0件になり、fail-closed を保つ)。未認証のトークン経路など
  設定なしで RLS 表に触る経路が壊れるため不採用とし、ポリシーの書き換え・索引の追加は行わない。
  migration の追加なし。制限ロールへの切り替え段階で実量とともに測り直す。

## 12. 第四段階の記録(#1723)

- `backend/migrations/0026_sessions_users_rls.sql`: `sessions`・`users` に RLS を有効化し、
  `sessions` は `user_id`、`users` は `id` と接続ごとの `app.current_user_id` が一致する行だけを許可するポリシーを追加。
  `FORCE` なしのため現行の所有者ロールの挙動は不変。比較はテキスト同士で行い、不正な値でも型変換エラーにならない。
- 結合テスト(`backend/tests/db_integration.rs` の `sessions_users_rls_isolates_restricted_role`):
  RLS が 2 表で有効・`FORCE` なしであること、所有者の既存操作
  (2利用者のログインとセッション照会・セッション削除と再ログイン・第三利用者の作成と利用者削除)が通ること、
  制限ロールでは設定なしで0件・利用者 A の設定で A の行のみ・
  他人名義のセッション挿入と他人 `id` での利用者挿入が失敗することを固定。
  自分の行の更新(`users` の本人メール更新・`sessions` の本人挿入)は通る。
- アプリコードの変更なし(`SET LOCAL` の配線は行わない)。

### 利用者が決まる前に読む経路の一覧

| 経路 | 実装 | 触る表 | 利用者確定前の理由 |
|---|---|---|---|
| 全認証要求の入口 | `extractors/auth.rs` の `AuthenticatedUser` → `auth::select_user_by_session` | `users` JOIN `sessions`(token_hash 照合) | Cookie のトークンから利用者を特定する前 |
| 現在の利用者表示 | `handlers/v1/auth.rs` の `get_session` → `select_user_by_session` | 同上 | 同上 |
| ログアウト | `handlers/v1/auth.rs` の `delete_session` → `auth::delete_session`(token_hash で DELETE) | `sessions` | 利用者不明でも冪等に消す必要がある |
| アカウント削除 | `handlers/v1/auth.rs` の `delete_account` → `select_user_id_by_session` 後に `delete_account` | `sessions` 参照後に `users` 削除 | 削除対象の特定自体がセッション照合 |
| 削除確認の発行 | `create_account_deletion_confirmation`(入口で `AuthenticatedUser`) | `users` JOIN `sessions` | 照合後に確認 Cookie を発行する |
| 管理者判定の再照合 | `handlers/v1/admin_logins.rs` → `select_user_id_by_session` | `sessions` | 管理者画面の入口照合 |
| 通知設定の token 照会・消費 | `handlers/v1/login_history.rs` → `login_notify::peek_token`・`consume_token` | `login_notify_tokens`(token_hash 照合) | 平文トークンから利用者を特定する前 |
| ログイン Tx | `auth::upsert_user_and_rotate_session`(upsert 後に `set_local_user`) | `users` upsert 後に `sessions` 回転 | upsert 前は `google_id` しかない |
| 回転時の期限切れ掃除 | `rotate_session_in_tx`(他人の期限切れ行も DELETE) | `sessions`(他人行を含む) | 利用者 A の設定では他人行が見えず掃除が残る |
| 端末一覧・解決 | `login_event::list_devices`・`resolve_device_hash` | `sessions`(利用者 ID あり) | 利用者確定後は通る(配線対象) |
| 通知設定の取得・回数 | `login_notify::get_setting`・`recent_send_count` | `users`・`login_notify_tokens`(利用者 ID あり) | 利用者確定後は通る(配線対象) |
| 一括登録の利用者ロック | `domain/bulk.rs` の `lock_user_domain`(`users` 行の `FOR UPDATE`) | `users`(本人行) | 利用者確定後は本人行が見えるため通る |
| 管理者横断ビュー | `login_event/admin.rs` の `search`(`login_events` JOIN `users`) | `users`(他人行を含む) | 管理者が全利用者をまたぐ |
| 古い記録の破棄 | `login_event::prune_login_events`(利用者なしの cron) | `login_events` | 利用者がいない |

### ロール切替の段階での通し方の案の比較(実装はしない)

| 案 | 仕組み | 利点 | 欠点 |
|---|---|---|---|
| A 照会専用の SECURITY DEFINER 関数 | トークン照合(`sessions`・`login_notify_tokens`)と利用者特定だけを所有者権限の関数に閉じ込め、引数は token_hash のみ・戻りは利用者 ID のみに絞る。`search_path` を固定し、戻り値以外の列を返さない | 接続ロールは制限のまま特権を最小化できる。呼び出し箇所が照合の正本になり監査しやすい | 関数の所有者管理・`search_path` 固定・`STABLE`/`SECURITY DEFINER` の定型が要る。期限切れ掃除や `consume` の更新も関数内に入れる設計が要る |
| B 照会専用ロール(読み取り限定の別接続) | 認証前の照合だけ別プール・別接続文字列で照会ロールを使う。トークン表と `sessions`・`users` には照合に必要な `SELECT` のみ与え、書き込み(`consume` の `used_at` 更新・回転時の DELETE/INSERT)は認証確定後の制限ロール側に残す | SQL はそのまま使え、特権関数の管理が要らない | 接続文字列とプールが増え、切替ミスで権限混同が起きやすい。Hyperdrive の束ね先が増える。読み書きの境界が2接続にまたがる |
| C 所有者ロールのまま照合だけ迂回 | 照合経路だけ現行の所有者接続に残し、それ以外を制限ロールに寄せる | 変更が最小で、期限切れ掃除や管理者横断ビューも壊れない | 特権の境界が接続ごとに割れ、将来の書き間違いが照合経路に残る。監査 7-1 の対応として不十分になりやすい |

- 期限切れ掃除は A では回転関数内に含めるか cron の破棄に寄せ、B/C では別権限の掃除経路が要る。
  現行の回転文(`WHERE user_id = $1 OR id IN (expired)`)は制限ロールでは他人行が見えず掃除が残るため、
  切り替え時に文を変えるか掃除を分離する。
- ログアウトの DELETE は冪等に成功応答を返す実装のため、ポリシーだけでは行が残る。
  A では所有者関数の消去、B では照会ロールに DELETE を足すか認証後の消去に寄せる判断が要る。
- 管理者横断ビューと cron 破棄は利用者をまたぐため、制限ロールの `SET LOCAL` 方式では通らない。
  切り替え時に管理者専用ロールか所有者のまま残すかを決める。
- 次の段階(接続ロールの切り替え)で、A を主案・B を退路として使い捨て DB で比べ、
  照合・消費・回転・掃除・管理者横断の5経路が制限ロールでどう振る舞うかを固定してから決める。

## 13. 第五段階の設計(接続ロールの切り替え)(#1736)

### 結論

- 主案は (b) 所有者の接続のまま Tx 内で `SET LOCAL ROLE` する。新しい秘密は要らない。
  退路は (a) 新しいログインロールと接続文字列を作る。Supabase 側で (b) の `SET ROLE` が
  許されない場合だけ (a) に切り替える。
- 進め方は設計の本章を含めて全5 PR。実装は 4 本に分ける:
  - 5a: 制限ロール作成の migration(0027)。アプリコードの変更なし。
  - 5b: 照合専用の `SECURITY DEFINER` 関数の migration(0028)。アプリコードの変更なし。
  - 5c: 書き込み Tx の切替(`begin_with_user` の設定を `app_enter` 呼び出しに寄せる)。読み取りは変えない。
  - 5d: 読み取りの切替(問い合わせごとの短い Tx 化)と管理者横断・cron の扱い決定。
- 往復回数の見込みは §13.2 の表のとおり。準備は `app_enter(uuid)` 1呼び出しにまとめ、
  書き込みは 1 リクエストあたり +0 往復、読み取りの一覧は並行を保ったまま
  1 問い合わせあたり待ち 1 往復→3 往復になる。`FORCE` は 5d の完了まで付けない。
- 5d(読み取りの切替)には関門を置く。一覧の応答の増加見込みが +50ms 以内のときだけ進める。
  見積もりの方法と計算例は §13.2 に書く。

### 13.1 ロールの切り替え方: (a) と (b) の比較

| 観点 | (a) 新しいログインロールと接続文字列 | (b) 所有者の接続のまま `SET LOCAL ROLE` |
|---|---|---|
| 仕組み | 制限ロールを `LOGIN` で作り、Hyperdrive/Wrangler に2つ目の接続文字列を登録する。認証後の経路だけ新プールに寄せる | 制限ロールを `NOLOGIN` で作り、所有者から `SET LOCAL ROLE 制限ロール` する。Tx 開始直後に1文足すだけで、接続文字列とプールは1つのまま |
| 秘密 | 新しい秘密が要る(Hyperdrive の付け替えと secrets 登録)。管理対象が2倍になる | 新しい秘密は要らない。登録手順もない |
| コードの変化 | `Db` が接続を2つ持ち、経路ごとに使い分ける。使い分けミスが権限混同になる | `begin_with_user` と読み取り用の短い Tx 補助の2か所に集約できる。使い分けは Tx の有無だけ |
| プールとの相性 | プールが2つに割れ、Hyperdrive の束ね先が増える。読み書きの境界が接続をまたぐ | Tx 単位の `SET LOCAL` のため、プール・Hyperdrive の接続使い回しに漏れない。`COMMIT`/`ROLLBACK` で自動的に戻る |
| 失敗の向き | 接続文字列の取り違えは特権混同(見える側)に倒れやすい | 設定失敗は権限不足で Tx が止まる(使えない側)に倒れる |

使い捨て DB(`postgres:16-alpine`)での確認結果:

- 所有者接続で `BEGIN` → `SET LOCAL ROLE app_restricted` →
  `SELECT set_config('app.current_user_id', $1, true)` → 照会の順で、
  本人の行だけが見え、他人名義の書き込みは行共有違反で失敗し、
  設定なしでは 0 件になる。`COMMIT` 後は所有者に戻り全行が見える。
- 非特権の所有者もどきでは、所属付与なしの `SET LOCAL ROLE` は権限不足で失敗し、
  `GRANT app_restricted TO <所有者>` の後は通る。特権者では所属なしでも通るため、
  migration には所属付与を含め、非特権でも成り立つ形にする。
- Supabase/Hyperdrive 固有の残件は、接続ロールが `SET ROLE` できるかの一点に絞られる。
  Session pooler(5432)はセッションモードだが `SET LOCAL` は Tx 内に閉じるため、
  使い回しへの漏れはない。切替前に利用予定の Session pooler 利用者で
  `BEGIN` → `SET LOCAL ROLE` → `ROLLBACK` が通ることだけを確かめる。
  通らない場合は (a) を使う。

### 13.2 読み取りの並行と往復回数

現行の一覧は `domain/search.rs` の `try_join4`(count・data・summary・facets)と
`try_join_all`(facet 内訳)を `&Db` の並行問い合わせで投げ、1 問い合わせ 1 往復で済ませている。
書き込みは `begin_with_user`(`BEGIN` + `set_config` + 本文 + `COMMIT`)で 1 往復増を受け入れ済み。

切替後の読み取りは並行を保つ。各問い合わせをロールと利用者 ID を設定した短い Tx で投げる。

### 準備のまとめ方: `app_enter(uuid)` を使う

準備(`SET LOCAL ROLE` + 利用者 ID 設定)を1つの関数呼び出しにまとめる。
`SECURITY INVOKER` の普通の関数 `app_enter(uuid)` を migration で作り、
中では束縛変数なしの固定文だけを実行する。アプリ側は `BEGIN` のあと
`SELECT app_enter($1)`(束縛変数あり)を1回呼ぶだけで、SQL の文字列連結はしない。

```sql
CREATE FUNCTION app_enter(uid uuid) RETURNS void
    LANGUAGE plpgsql SECURITY INVOKER SET search_path = public, pg_temp AS $$
BEGIN
    PERFORM set_config('role', 'app_restricted', true);
    PERFORM set_config('app.current_user_id', uid::text, true);
END $$;
REVOKE ALL ON FUNCTION app_enter(uuid) FROM PUBLIC;
```

使い捨て DB(`postgres:16-alpine`)での確認結果:

- 非特権の所有者もどき(所属付与あり)の接続で `BEGIN` → `SELECT app_enter($1)` →
  照会の順で、本人の行だけが見え、他人名義の書き込みは行共有違反で失敗する。
  `COMMIT`/`ROLLBACK` 後はロールと利用者 ID の設定が消える。
- `set_config('role', ...)` は `SET LOCAL ROLE` と同じく所属検査を受ける。
  所属なしの非特権ロールが呼ぶと権限不足で失敗し、Tx が止まる(使える側には倒れない)。
  制限ロール名は関数内の固定値で、利用者 ID は `uid::text` の変換のみのため、
  呼び出し側の文字列連結は要らない。
- `BEGIN` と `SELECT app_enter($1)` を1往復にまとめることはしない。
  `batch_execute` には束縛変数がなく、`BEGIN` と束縛付き呼び出しは別送になる。
  `db.rs` は逐次 await のためパイプラインで重ねても往復回数は減らない。
  1 問い合わせは `BEGIN`(1)・`app_enter`(1)・本文(1)・`COMMIT`(1)の4往復になる。

### 読み取り専用の短い Tx の後始末

読み取り専用の短い Tx は、結果を返したあとの `COMMIT`/`ROLLBACK` を待たない。
読み取りでは書くものがなく `COMMIT` と `ROLLBACK` は等価で、どちらも
Tx 内の設定を捨てるだけのため。`Tx` を drop すれば `rollback_and_release` が
裏側で `ROLLBACK` を送ってから接続をプールへ戻す(呼び出し側は待たない)。
接続の再利用前に後始末が終わることはプールへの返却順で保証される。
書き込みは耐久の確認が要るため `COMMIT` の待ちを残す。

### 1リクエストあたりの往復回数と体感の見積もり

待ちだけを数える(後始末の非同期分は除く)。一覧は `try_join4` + `try_join_all` の
並行問い合わせのため、待ちは並列の最大値になる。

| 経路 | 現行(待ち) | 切替後(待ち) | 増分 |
|---|---|---|---|
| 一覧1リクエスト全体(例: 10並行) | 約1往復 | 約3往復(`BEGIN`・`app_enter`・本文を並行、後始末は非同期) | +2往復 |
| 単一文の書き込み | 4往復(`BEGIN`・`set_config`・本文・`COMMIT`) | 4往復(`BEGIN`・`app_enter`・本文・`COMMIT`) | +0往復 |
| ログイン Tx | 5往復(`BEGIN`・upsert・`set_config`・回転・`COMMIT`) | 5往復(`BEGIN`・upsert・`app_enter`・回転・`COMMIT`) | +0往復 |

1往復の目安は、5d の着手時に本番の計測値から求める。
`/api/v1/session` の 401 経路は DB 問い合わせ1回のため、その p50 が
「1往復 + アプリ処理」の上限になる。増加見込みは `+2往復 × 1往復の目安` で出す。
計算例: 1往復の目安が 10ms なら一覧の増加は約 +20ms になる。
なお #1114 の本番計測(`/api/v1/session` の 401 経路で 37-45ms)は
Fly・Neon 時代の値のため上限の参考に留め、5d では現行経路で測り直す。
Hyperdrive のクエリ枠に対しては、一覧1回あたりの送信数が約4倍になることも
5d の確認事項に含める。

5d の関門: 増加見込みが一覧の応答 +50ms 以内のときだけ読み取りの切替に進める。
超える場合は立ち止まり、読み取りを所有者のまま残すか (a) 案に戻るかを選び直す。

なお §11 のとおり制限ロールでは並び順つき一覧が並べ替え併用になりやすい。
5d で実量とともに測り直す。索引の追加・ポリシーの書き換えは行わない。

### 13.3 利用者確定前の経路の通し方

§12 の A 案を主とする。照合と消費・回転・消去だけを所有者権限の関数に閉じ込め、
引数はハッシュのみ・戻りは利用者 ID のみに絞る。接続ロールは制限のままにする。
定型は `SECURITY DEFINER`・`SET search_path = public, pg_temp`・`REVOKE ALL ON FUNCTION ... FROM PUBLIC`・
制限ロールへの `GRANT EXECUTE` とする。列の持ち出しが必要な入口は利用者 ID の特定に留め、
利用者行の取得自体は特定後の制限 Tx で行う。

| 関数 | 引数 | 戻り値 | 揮発性 | 用途 |
|---|---|---|---|---|
| `auth_lookup_user_id_by_session(bytea)` | `token_hash` | `uuid` または NULL | `STABLE` | 全認証要求の入口と管理者判定の再照合 |
| `auth_delete_session_by_token(bytea)` | `token_hash` | `void` | `VOLATILE` | ログアウトの冪等な消去 |
| `login_notify_peek_by_token(bytea)` | `token_hash` | `(user_id uuid, expires_at timestamptz)` | `STABLE` | 通知設定の token 照会 |
| `login_notify_consume_by_token(bytea)` | `token_hash` | `user_id uuid` または NULL | `VOLATILE` | 通知設定の token 消費(`used_at` 更新を含む) |
| `auth_rotate_session(uuid, bytea, bytea)` | `user_id`・`token_hash`・`device_hash`(NULL 可) | `void` | `VOLATILE` | 回転時の期限切れ掃除と旧行削除・新行挿入を1か所に含める |
| `auth_upsert_user(text, text, text, text)` | `google_id`・`email`・`name`・`picture_url` | `uuid` | `VOLATILE` | ログイン Tx の upsert 部分(回転前は利用者が定まらないため) |

上記以外の利用者をまたぐ経路は関数化しない。管理者横断ビュー(`login_event/admin.rs` の `search`)と
cron の破棄(`prune_login_events`)は 5d で所有者のまま残すか管理者専用ロールに寄せるかを決める。
現行の回転文の他人行掃除は上記 `auth_rotate_session` 内に移し、制限 Tx 側の文からは他人行条件を外す。
ログアウト後の参照は冪等な成功応答のままとする(ポリシーで残っても成功を返す現行の向きを変えない)。

### 13.4 切替手順と戻し方

順番は 5a → 5b → 5c → 5d とする。表ごと・経路ごとの段階的な切替になるよう、
各 PR は後続なしでも本番に入れられる形にする。過去の migration は書き換えない。

1. 5a: 制限ロール(`NOLOGIN`)の作成と各表への権限付与・所有者への所属付与の migration を足す。
   `FORCE` は付けない。所有者の挙動は変わらず、制限ロールの直接接続で隔離だけを固定する。
2. 5b: §13.3 の関数の migration を足す。アプリはまだ呼ばない。使い捨て DB で引数・戻り・権限だけを固定する。
3. 5c: 書き込み Tx(`begin_with_user` とログイン Tx の `set_local_user` 呼び出し)だけを (b) に寄せる。
   ログイン Tx では upsert で利用者 ID が決まったあとに `app_enter`/`set_local_user` で設定してから
   回転関数を呼ぶ順にする(`auth_rotate_session` は引数と設定の一致を確かめるため)。
   読み取り・認証前・管理者・cron は変えない。問題が出たらコードを戻すだけでよい。
4. 5d: 読み取りを短い Tx 化し、認証前経路を §13.3 の関数に寄せ、管理者横断と cron の扱いを決める。
   `EXPLAIN` を測り直し、増加見込みが +50ms 以内であることを関門として確かめてから進める。
   `FORCE` の検討は全経路の配線が終わってから行う。

戻し方はコードの revert が正本になる。migration の追加分(ロール・関数・権限)は残しても
所有者の挙動を変えず、露出にはならない。取り消す場合は新しい migration で `DROP FUNCTION`・
`REVOKE`・`DROP ROLE` を行う。`DISABLE ROW LEVEL SECURITY`・`DROP POLICY` は使わない
(本番の行分離を弱める方向の戻しはしない)。

### 13.5 秘密の登録手順

- (b) の主案では新しい秘密は要らない。利用者の登録手順はない。
- 退路の (a) が必要になった場合だけ、利用者が行う手順は以下になる。値は書かない。
  1. 制限用のログインロールに対応する接続文字列を用意する。
  2. Cloudflare の Hyperdrive 設定に新しい接続先として登録する。
  3. デプロイ先の secrets に新しい接続文字列を登録する。
  4. 付け替え後は旧接続文字列を残し、新経路の疎通を確認してから切り替える。
- 実装担当・統合担当は秘密を見ない。手順書・PR・ログに接続文字列・鍵を残さないことは
  `docs/db-migration-supabase.md` の担当分離と同じにする。

## 14. 第五段階(5a)の記録(#1747)

- `backend/migrations/0027_app_restricted_role.sql`: 制限ロール `app_restricted` を `NOLOGIN` で作成し、
  本人表 11 表(`users`・`sessions`・`dividends`・`domestic_stocks`・`mutualfunds`・`asset_balances`・
  `fund_balances`・`csv_import_history`・`login_events`・`login_notify_tokens`・`admin_login_views`)に
  `SELECT, INSERT, UPDATE, DELETE`、共有表 6 表(`stock`・`security_prices`・`dividend_per_share_cache`・
  `market_data_provider_rate_control`・`fund_codes`・`fund_prices`)と
  ビュー 2 件(`asset_balances_view`・`fund_balances_view`)に `SELECT`、
  関数 2 件(`fund_name_normalize(text)`・`fund_expected_nav_date(timestamptz)`)に `EXECUTE`、
  `public` スキーマに `USAGE` を与え、実行者(`CURRENT_USER`)を所属させる。
  `FORCE` なしのため所有者の挙動は変わらない。ロール名は §13 の設計どおり `app_restricted`。
- 作成は存在確認つきで、所属付与は未所属のときだけ行う。権限不足時は失敗させず `NOTICE` に留める
  (ロール作成なし→権限付与全体を省略、所属付与だけ失敗→そこだけ省略)。表権限の付与は厳密に扱い、
  所有者以外の実行など本当に通らない場合は失敗させる。過去の migration は書き換えていない。
- 結合テスト(`backend/tests/db_integration.rs` の `app_restricted_role_has_least_privilege`):
  ロールが `NOLOGIN`・`BYPASSRLS` なしであること、所有者が所属していること、
  本人表 11 表が読み書き・共有表 6 表とビュー 2 件が読み取りだけ・関数 2 件が実行可であること(権限カタログで固定)、
  `SET LOCAL ROLE app_restricted` + `set_config('app.current_user_id', ...)` で
  本人の行だけが見え・他人名義の書き込みが失敗し・共有表は読めるが書けず、
  終了後は所有者に戻り両利用者の行が読めることを固定する。
- アプリコードの変更なし。
- 権限不足時の挙動の確認: 使い捨て DB(`postgres:16-alpine`)で `NOCREATEROLE` の移行用ロールに
  `CONNECT, CREATE`(DB)と `CREATE, USAGE`(schema)のみ与えて全 migration を実行した。
  0027 は成功扱い(`_sqlx_migrations` に記録)で `app_restricted` は作られず、表は移行用ロール所有で残る。
  デプロイは止まらない。ロールだけ別管理者が先作りした場合の所属付与の失敗も `NOTICE` で止めないことを確かめた。
  `CREATEROLE` ありの非特権ロールが自作ロールへ所属付与できることも確かめた。
  よって本番の移行ロールに `CREATE ROLE` がなくてもデプロイは止まらず、
  その場合は管理者が SQL エディタ等で作成と権限付与・所属付与を1回だけ行う(PR 本文に全文を載せる)。
- 付随する修正: `0026_sessions_users_rls.sql` が `migrate.rs` に未登録だったため、
  この PR で 0026 と 0027 の登録を足した(過去ファイル自体の書き換えなし)。
  未登録のままでは `sessions`・`users` の RLS が適用されず、第四段階の結合テストが空振りになる。

## 15. 本番の状態確認(#1749)

- 本番の `app_restricted` と RLS の状態は、GitHub の Actions 画面から `DB RLS State` ワークフローを手動実行して読み取りだけで確かめる(main ブランチから実行する)。

## 16. 第五段階(5b)の記録(#1750)

- `backend/migrations/0028_app_enter_and_auth_functions.sql`: 準備を1呼び出しにまとめる
  `app_enter(uuid)`(`SECURITY INVOKER`、Tx 内で `SET LOCAL ROLE app_restricted` 相当の
  `set_config('role', ...)` と利用者 ID の `set_config('app.current_user_id', ...)` を行う)と、
  §13.3 の照会用 `SECURITY DEFINER` 関数 6 件
  (`auth_lookup_user_id_by_session`・`auth_delete_session_by_token`・
  `login_notify_peek_by_token`・`login_notify_consume_by_token`・
  `auth_rotate_session`・`auth_upsert_user`)を追加。
  回転は期限切れ掃除と旧行削除・新行挿入を1関数に含め、端末なし(`device_hash` NULL)は
  全端末・端末ありは当該端末と未設定の旧行だけを消す(現行の2文の向きを保つ)。
  `auth_rotate_session` は引数の利用者 ID が接続の設定(`app.current_user_id`)と一致しないと
  例外にする。制限ロールから他人の利用者 ID で回転を呼んでも他人の行に触れない。
  ログイン Tx では upsert で利用者 ID が決まったあとに利用者 ID を設定してから回転を呼ぶ順にする。
  いずれも `SET search_path = public, pg_temp` を固定し、戻り値は利用者 ID 等の最小限に絞る。
  `EXECUTE` は `app_restricted` にだけ与え、`PUBLIC` からは取り上げる
  (`app_enter` は切り替え前の所有者が呼ぶため所有権のままにする)。
  ロール不在時の `GRANT` は `NOTICE` に留め、デプロイを止めない(0027 と同じ向き)。
  `FORCE` なしのため所有者の挙動は変わらない。過去の migration は書き換えていない。
- 結合テスト(`backend/tests/db_integration.rs` の `stage5b_definer_functions_and_app_enter`):
  照会 6 関数の権限カタログ(`app_restricted` は実行可・`public` は不可)と
  `app_enter` が所有者専用(`app_restricted`・`public` とも不可)であること・`DEFINER` 属性・
  揮発性(照会と参照のみ `STABLE`)・`search_path` 固定を固定し、
  権限なしロールからは照会関数と `app_enter` が権限不足で呼べないこと、
  `app_restricted` からは所有するハッシュで本人の利用者 ID が返り・未知のハッシュでは
  他人の行を返さないこと(照会・参照・消費のいずれも)、`app_enter` のあとは本人の行だけが見え
  `COMMIT` で所有者に戻ること、回転・消去・upsert が所有者権限で通ること、
  設定と違う利用者 ID での回転が失敗することを固定する。
- 付随する修正: `fresh_database_applies_all_migrations` と
  `baseline_applied_database_applies_increments` の期待 version が `0..=25` のまま止まっており、
  0026・0027 の追加後も更新されていなかったため `0..=28` に直した。
  登録漏れの再発防止として `migrate.rs` に Docker 不要の単体テスト
  (`all_migration_files_are_registered`: `migrations/*.sql` と `MIGRATION_FILES` の一致)を足した。
- アプリコードの変更なし(関数の呼び出しは 5c・5d で行う)。

## 17. 第五段階(5c)の記録(#1753・#1757)

初回の #1754 は本番ログインの `DATABASE_ERROR` により #1755 で戻した。
#1757 で下記の配線を復元し、§18 の修正と検証を加えて出し直す。

- 配線の変更は `backend/src/db.rs` の1か所のみ。`Tx::set_local_user` が
  `SELECT set_config('app.current_user_id', ...)` の代わりに
  `SELECT app_enter($1)`(束縛変数あり)を呼ぶ。`Db::begin_with_user` はこの関数を呼ぶため、
  書き込みの全経路(CSV 取り込み・一括登録・全削除・ログイン記録・通知トークン・
  通知設定の更新・閲覧記録・セッション削除・利用者削除)が制限ロールで実行される。
  読み取り・認証前の照会・管理者の横断・cron は変えない。
- ログインの取引は `auth_upsert_user` で利用者を決めたあとに `app_enter` を呼び、
  `auth_rotate_session` で回転する順にした(関数は 5b で用意済み)。
  利用者行の取得は upsert と同じ文にはまとめず、upsert の次の文で所有者として読む。
  同じ文の外側からは関数が挿入した新規行が見えないため、新規利用者が RowNotFound になる。
  回転は直接の `DELETE`/`INSERT` の代わりに `auth_rotate_session` を呼ぶ。
  回転は引数と設定の一致を確かめるため、他人の利用者 ID での回転は失敗する。
- 往復回数は、一括登録などは `BEGIN`+`app_enter`+本文+`COMMIT` の4往復のまま、
  単純なログイン Tx は `BEGIN`+upsert+`app_enter`+回転+`COMMIT` の5往復のまま。
  端末付きログインだけは利用者行の取得を別文にしたため +1 往復になる。
- 結合テスト(`backend/tests/db_integration.rs` の `rls_5c_writes_via_app_enter`):
  本番と同じ所有者の接続から書き込みの全経路を `app_enter` 経由で実行し、
  成功すること・Tx 内では `current_user` が `app_restricted` で `COMMIT` 後に所有者に戻ること・
  本人の行だけが変わり他人の行は変わらないこと・Tx 外に利用者 ID の設定が残らないことを固定する。
  対象は CSV 取り込みと履歴・一括登録・全削除・ログイン記録・トークン発行・設定更新・
  閲覧記録・端末と全体のセッション削除・利用者削除・ログインと再ログイン(旧失効)・端末付きログイン。
- 初回は migration を追加しなかったが、出し直しでは §18 の 0029 が必要。
  過去の migration は書き換えていない。
- 戻し方は 5c のコードの revert。0029 の切替権限を残しても、所有者で動くコードの挙動は変わらない。

## 18. 5c のログイン失敗の再現と修正(#1757)

### 原因と再現

- #1754 の差分を戻し、既存の `rls_5c_writes_via_app_enter` を Docker で実行すると成功した。
  このテストの接続は特権者であり、所属の `SET` オプション欠落を検出できなかった。
- `NOSUPERUSER NOBYPASSRLS CREATEROLE` の利用者を作り、その利用者自身が migration を
  適用して表と関数の所有者となる接続を用意した。新規・既存利用者 × 端末あり・なしの
  4 通りがすべて `app_enter` 内の `set_config('role', ...)` で SQLSTATE `42501` となった。
  API の分類は `DATABASE_ERROR`。利用者の情報・引数は診断出力に使わない。
- PostgreSQL 16 以降では、非特権の `CREATEROLE` 利用者が作ったロールへの自動所属は
  `ADMIN TRUE, SET FALSE, INHERIT FALSE` になる。
  0027 は `pg_auth_members` に所属があれば `GRANT` を省くため、`SET FALSE` のまま残る。
  所属確認は `MEMBER` と `SET` を分ける必要がある。
  仕様: [PostgreSQL Role Attributes](https://www.postgresql.org/docs/current/role-attributes.html)。
- 本番の読み取り専用の [DB RLS State 実行結果](https://github.com/zaichu/shoken-webapp/actions/runs/41555886355)では、
  `app_restricted` が `NOLOGIN`・`NOBYPASSRLS`、所有者が所属、利用者表 11 表は RLS 有効・
  FORCE なし、migration 28 が適用済みであることを確認した。この結果には `set_option` が
  含まれず、本番時の SQLSTATE も残っていないため、本番そのもののエラー詳細との照合は未実施。
  再現した欠陥と、本番で観測したエラーの分類・既存の所属確認の不足が一致する。

### 修正と検証

- `0029_app_restricted_set_option.sql` を新設し、
  `GRANT app_restricted TO CURRENT_USER WITH SET TRUE, INHERIT FALSE` で移行用の所有者に切替を許可する。
  自動継承を追加せず、表・関数の権限、RLS ポリシー、PUBLIC の権限も変えない。
  `migrate.rs` に登録し、既存履歴の期待値を 29 まで更新した。
  ロール不在・管理権限不足の場合は移行を失敗させ、切替不能のまま 5c を配信しない。
- `backend/tests/login_rls.rs` は非特権の所有者から実際のサービス関数を呼び、
  `BEGIN` → `auth_upsert_user` → (端末ありでは利用者行取得) → `app_enter` →
  (端末ありでは履歴照会) → `auth_rotate_session` → (端末ありでは履歴登録) → `COMMIT` を通す。
  新規・既存の 4 通り、名前の更新、旧セッションの失効、履歴の分類、設定の後始末を確認する。
  自動所属相当の `SET FALSE` で `42501`/`DATABASE_ERROR` を再現してから 0029 で修復し、
  再適用が成功し `INHERIT FALSE` が維持されることも確認する。
- `DbError` のログを共通化し、Worker では `console_error!`、ホストでは `tracing::error!` に
  操作、エラー種別、SQLSTATE、制約名、PostgreSQL の routine、許可した固定の文種別・関数名だけを出す。
  SQL 本文、引数、エラー本文・detail・hint・context、接続文字列は出さない。
  API 応答への変換、接続タスク、ロールバックの失敗も同じ方法で扱う。
  ログのテストは SQL 本文の漏出と制約名欠落で失敗することを先に確認し、修正後の成功を確認した。
- `make test-ignored` に新しいログイン結合テストを追加した。#1756 の PR #1760 が
  未マージのため、今回の再現と修正の確認は手元の Docker で実行し、結果を PR に記録する。
  PR での実行は #1760 の取り込み後の workflow に任せる。
- CI で `rls_set_local_wiring_scopes_authenticated_paths` が `app_enter` の実行権限なしで失敗した。
  5c 以降の `begin_with_user` は所有者専用の `app_enter` を呼ぶため、試験用の `rls_app` ロールには
  テスト内で `EXECUTE` と `SET` 付き所属を与える。旧試験の本文は変えない。
- CI で `session_token_hash_rolling_deploy_compat` が停滞で失敗した。
  `auth_rotate_session` の期限切れ掃除が単純な `DELETE` になり、他者のロック待ちをしていた。
  旧 SQL の `FOR UPDATE SKIP LOCKED` の向きを関数に戻すため `0030_auth_rotate_session_skip_locked.sql` を新設し、
  `CREATE OR REPLACE` で置き換える(署名・所有者・権限は不変。過去の migration は書き換えない)。

### Hyperdrive の取引の扱い

- [Cloudflare の接続プール仕様](https://developers.cloudflare.com/hyperdrive/concepts/connection-pooling/)では、
  明示的な取引中は同じ接続を保持し、終了後の返却時に設定をリセットする。
  [PostgreSQL SET の仕様](https://www.postgresql.org/docs/current/sql-set.html)でも、
  `SET LOCAL` は `COMMIT`・`ROLLBACK` まで有効である。
- `db.rs` は取引専用に `fresh_client` を作り、その同一クライアントで逐次実行する。
  `app_enter` は `set_config(..., true)` を使い、セッション設定を残さない。
  Docker では取引内の backend PID が同じで、COMMIT・ROLLBACK・drop 後に所有者と利用者設定が
  復元されることを確認する。Hyperdrive 本体を通した実測は未実施であり、仕様上の確認と区別する。
  今回の `42501` は Hyperdrive なしでも再現するため、接続の入れ替わりを修正対象にはしない。

## 19. DB 1往復の計測(#1766)

5d の関門(§13.2、一覧の応答の増加見込みが +50ms 以内のときだけ読み取りの切替に進める)の判断材料として、
本番の Worker で DB の問い合わせ1回にかかった時間(Hyperdrive 経由の往復)を測ってログに出す。

### 測り方

- 測る場所は `backend/src/db.rs` の `Executor` 実装の入口1か所のみ(`Db` と `Tx` の
  `query_rows`・`execute`・`batch_execute`)。各ハンドラーには散らさない。
  計時は `DbTimer::start` で開始し、唯一の出口 `DbTimer::finish` で出す。
  `BEGIN`・`COMMIT`・`ROLLBACK` の枠組みは対象外で、本文の問い合わせ1回 = 1往復を測る。
  計時は接続の取得後に開始し、初回問い合わせの接続確立・認証の待ちは含めない。
  `app_enter`・`auth_*` の関数呼び出しも `Executor` 経由のため種別つきで測られる。
- 出すのは固定の種別(`query_tag` の `SELECT`・`INSERT`・`UPDATE`・`DELETE`・`WITH`・
  `app_enter` 等)と ms だけ(例: `db_rtt statement=SELECT elapsed_ms=3`)。
  SQL 本文・引数・利用者 ID・接続情報・エラー本文は出さない。
- 頻度は 100 回に 1 回(既定)。`DB_RTT_SAMPLE_EVERY`(`wrangler.toml` の `[vars]` と
  `[env.dev.vars]` に `100` で登録)で変更でき、`0` で停止する。
  ホスト側は起動時に環境変数から、Worker 側はリクエスト開始時に `Env` から解決し、
  問い合わせごとの解決はしない。未設定・不正値は既定の 100 に倒れる。
- ログの段階は warn。`worker-tail` は fetch イベントの error/warn 行を拾う設定のため、
  ワークフローの絞り込み変更なしで artifact(`filtered.jsonl`)に残る。
- wasm の計時は `worker::Date::now()` の ms 差分(§13.2 の関門に要る ms 精度はある)、
  ホスト側は `Instant`。計測自体は時刻取得と原子カウンタの加算のみで、
  余計な往復を送らないため問い合わせを遅くしない。

### 集めた値から 5d の関門を判断する手順

1. マージ後に統合担当が `Worker Tail` ワークフローを回し、`filtered.jsonl` を得る。
2. `db_rtt` 行だけを抜き、種別ごとに `elapsed_ms` を集める。
   `SELECT` の p50 を「1往復の目安」とする。p95 も併せて見て外れ値の有無を確認する。
3. 増加見込みは `+2往復 × 1往復の目安`(§13.2 の表より一覧は待ち +2往復)で出す。
   計算例: p50 が 10ms なら増加見込みは約 +20ms。
4. 増加見込みが +50ms 以内のときだけ 5d(読み取りの切替)に進める。
   超える場合は立ち止まり、読み取りを所有者のまま残すか (a) 案に戻るかを選び直す(§13.2 の関門どおり)。
5. `app_enter` の種別の値は、書き込み Tx の設定コストの実測として 5c の振り返りに使う。
   一覧の `SELECT` とは混ぜずに見る。
