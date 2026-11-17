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
- 5d着手前の当初関門は一覧応答の増加見込み +50ms 以内だった。Issue #1608 の最新コメントで fetch 経路の実測に基づいて実装後比較へ判断を更新したため、このPRでは読み取りを切り替えた後に本番比較する。

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

5d着手時点では +50ms を関門としていた。Issue #1608 の11/14コメントは、5dを実装して本番で `origin=fetch` の前後比較を行い、+50ms基準で判断すると更新している。

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
4. 5d: 読み取りを短い Tx 化し、認証前経路を §13.3 の既存関数に寄せる。管理者横断と cron は所有者のまま残す。
   `EXPLAIN` を測り直し、実運用の応答時間は本番の `origin=fetch` ログで前後比較する。
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

## 20. 読み取りの制限ロール切替(第五段階5d)

- `backend/src/db.rs` に利用者 ID 付きの読み取り API を追加した。各問い合わせは `BEGIN` → `app_enter($1)` → 本文 SELECT の順に同じ接続で実行し、Tx drop 時に非同期 `ROLLBACK` で設定を捨てる。
- 対象は本人のデータを読む検索一覧(count/data/summary/facets を含む)、CSV 種別統計、ログイン履歴・端末、通知設定・送信数。通知 token の peek/consume とセッションの利用者 ID 照合は既存の5b `SECURITY DEFINER` 関数を使う。認証前のユーザー情報取得、管理者横断検索、共有表、cron は所有者経路のまま。
- 一覧の接続数を抑えるため、検索の count/data/summary/facets は1つの利用者 Tx・接続で順に実行する。接続は1本にできるが並行性は下がる。
- 一覧1リクエスト中の読み取り DB 接続数とクエリ数(コード上の最大値):

  | エンドポイント | facets 数 | summary 無効/有効 | summary+facets 時の SELECT 数 | 最大同時読み取り接続 |
  |---|---:|---:|---:|---:|
  | `GET /api/v1/dividends` | 5 | 2 / 3 | 8 | 1 |
  | `GET /api/v1/domestic-stocks` | 4 | 2 / 3 | 7 | 1 |
  | `GET /api/v1/mutual-funds` | 4 | 2 / 3 | 7 | 1 |
  | `GET /api/v1/asset-balances` | 1 | 2 / 3 | 4 | 1 |
  | `GET /api/v1/fund-balances` | 2 | 2 / 3 | 5 | 1 |

  いずれも count+data+summary(有効時)+FACETS 定義分。fund-balances の summary は nav 状態の SELECT も含む。Workers 設計資料の「同時アウトバウンド接続6/呼び出し」はプラットフォームの一般上限であり、今回の一覧最大1接続はこれを下回る。認証のセッション照合は一覧 Tx の前段で終わる。
- 往復回数と性能計測: 旧一覧は独立した SELECT を並行していた。切替後は1接続上で `BEGIN` → `app_enter` → count/data/summary/facets の各 SELECT を順番に実行し、`ROLLBACK` を送る。SELECT 1件あたりの追加往復を単純に一覧応答時間の +2 と見積もることはできず、接続確立コストに加え、並列待ちから最大8 SQLの直列待ちに変わる。変更前 tail(`/home/zaichu/project/.worktrees/.requests/baseline-5d/tail-before-5d-20261115.jsonl`)は `origin=fetch` の `db_rtt` のみで、NRTのSELECTは13件(p50 9ms、p95 30ms、範囲7〜37ms)。エンドポイント応答時間は含まない。一時Workerからの独自SQL計測はRLS付き一覧APIと同等でないため、前後比較として採用しない。認証秘密を複製しない条件では認証済み一覧を一時Workerから呼べず、+50ms以内は未確認。マージ前に同一利用者・同一coloの一覧応答計測を取得する。
- 結合テスト(`backend/tests/db_integration.rs` の `rls_5d_reads_via_app_enter`)で制限ロールの一覧・集計・facet、RLSによる別ユーザー行の遮断、通知token照会・消費、Tx後のロール/利用者設定の復元を確認する。
- migration の追加はない。既存の0023〜0026のRLSポリシーと0028の関数を使う。

## 21. DB 1往復の計測(#1766)

5d切替前後の比較材料として、
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
  測る間だけ 1 にし、測り終えたら 100 に戻す(戻す PR は別)(#1783)。
- ログの段階は warn。`worker-tail` は fetch イベントの error/warn 行を拾う設定のため、
  ワークフローの絞り込み変更なしで artifact(`filtered.jsonl`)に残る。
- wasm の計時は `worker::Date::now()` の ms 差分(§13.2 の関門に要る ms 精度はある)、
  ホスト側は `Instant`。計測自体は時刻取得と原子カウンタの加算のみで、
  余計な往復を送らないため問い合わせを遅くしない。

### 集めた値から 5d の関門を判断する手順

1. マージ後に統合担当が `Worker Tail` ワークフローを回し、`filtered.jsonl` を得る。
2. `db_rtt` 行だけを抜き、種別ごとに `elapsed_ms` を集める。
   `SELECT` の p50 を「1往復の目安」とする。p95 も併せて見て外れ値の有無を確認する。
3. 今回の5dは一覧を直列化して接続を1本にしたため、`+2往復 × p50` では応答時間を見積もらない。切替前後の同等リクエストの実測差を直接比較する。
4. 5dの変更前 tail は応答時間を含まないため、本番と別Workerで変更後の認証済み一覧応答を測っても前後差を算出できない。認証秘密を複製しない条件で変更前後を比較するには、マージ後に同一利用者・同一coloの一覧応答を計測する必要がある。差分が +50ms を超えた場合は読み取り切替の並行度/接続方式を再検討する。
5. `app_enter` の種別の値は、書き込み Tx の設定コストの実測として 5c の振り返りに使う。
   一覧の `SELECT` とは混ぜずに見る。

## 22. 第五段階(5e) FORCE ROW LEVEL SECURITY の検討(#1954)

### 結論

**条件つきで導入を推奨する。ただし現行の接続構成のまま全表へ一括導入しない。**

FORCE は、RLS を迂回するテーブル所有者が誤って利用者表へ触れた場合にもポリシーを適用できる。一方、現在は認証前・管理者横断・cron・migration・バックアップに所有者経路が残る。これらは利用者 ID の設定がないか、全利用者を読む必要があり、今のポリシーのまま FORCE すると認証・ログアウト・通知リンク・管理者検索・履歴掃除が失敗または空振りする。`SECURITY DEFINER` は関数所有者として実行する属性であり、FORCE を迂回する属性ではない。

FORCE はテーブル所有者自身が `ALTER TABLE ... NO FORCE ROW LEVEL SECURITY` を実行することを防がず、SUPERUSER / `BYPASSRLS` も常に迂回する。したがって、所有者資格情報の窃取や悪意ある DB 管理者に対する防御とは数えない。価値は、特権を使って DDL やロールを変更しない通常の SQL 経路で、利用者条件の欠落をポリシーが止めることに限られる。

導入条件は、(1) Workers の通常接続ロールが `NOSUPERUSER NOBYPASSRLS` であることを読み取り専用で確認する、(2)下表の残存所有者経路を、最小権限の専用ロールまたは入力・戻り値を限定した関数へ移す、(3) migration とバックアップにはアプリ接続と分離した管理経路を用意する、(4) Docker 結合テストで FORCE 下の全経路とロール属性を確かめる、の4点。通常接続ロールが SUPERUSER または `BYPASSRLS` なら FORCE だけではその接続の RLS 強制にならず、先にロールを分離する。現行の本番接続ロールの属性はこの Issue では読み取りにも触れておらず未確認。

### FORCE が及ぶ所有者経路

対象表は RLS 有効な11表: `users`・`sessions`・`dividends`・`domestic_stocks`・`mutualfunds`・`asset_balances`・`fund_balances`・`csv_import_history`・`login_events`・`login_notify_tokens`・`admin_login_views`。根拠は `backend/migrations/0023_csv_import_history_rls.sql`・`0024_audit_tables_rls.sql`・`0025_trading_tables_rls.sql`・`0026_sessions_users_rls.sql`。次の8経路をコード検索で確認した。

| # | 所有者経路(根拠) | FORCE 下の予測 | 必要な扱い |
|---:|---|---|---|
| 1 | 認証済みリクエストと `GET /api/v1/session`: [extractors/auth.rs](../../backend/src/extractors/auth.rs) の `select_user_by_session`、[auth.rs](../../backend/src/handlers/v1/auth.rs) の同じ照会 | `users`・`sessions` に利用者 ID を設定する前に検索するため、セッションが見えず未認証扱いになる | token hash から必要最小限の利用者情報を返す照会関数を、専用の非ログイン関数所有ロールへ分離する。単なる `SECURITY DEFINER` では不十分 |
| 2 | OAuth ログイン: [services/auth.rs](../../backend/src/services/auth.rs) の `auth_upsert_user` と関数後の `users` 読み取り | 利用者 ID 確定前の upsert は `WITH CHECK` に通らず失敗。後続の利用者行読み取りも ID 設定前 | upsert の責務・列を限定した専用関数/ロール経路を設計し、成功後は既存 `app_enter` の利用者 Tx へ移す |
| 3 | ログアウト: [services/auth.rs](../../backend/src/services/auth.rs) の `delete_session`。同ファイルの `select_user_id_by_session` はアカウント削除の入口でも呼ぶ | 直接 DELETE は行が見えずセッションを消せない。アカウント削除の入口照会も NULL となる | ログアウトは token hash だけを受ける限定関数へ移す。アカウント削除は認証照会の解決後、既存の利用者 Tx を使う |
| 4 | 通知解除リンク: [services/login_notify.rs](../../backend/src/services/login_notify.rs) と [handlers/v1/login_history.rs](../../backend/src/handlers/v1/login_history.rs) の peek/consume 関数 | 利用者 ID 設定前の `SECURITY DEFINER` 関数も `login_notify_tokens` のポリシーで行が見えず、リンクが無効扱いになる | token hash に限定した peek/consume 専用ロールまたは、必要な列だけを返す制御関数を用意する |
| 5 | 管理者の全利用者検索: [services/login_event/admin.rs](../../backend/src/services/login_event/admin.rs) の `search` と [handlers/v1/admin_logins.rs](../../backend/src/handlers/v1/admin_logins.rs) | 横断 SELECT が RLS で空になり、検索結果が欠落する。閲覧監査の INSERT は `begin_with_user` で `app_restricted` に入るため別途維持できる | 管理者専用ロール/関数へ検索だけを分離し、管理者認証後にのみ呼べる権限にする |
| 6 | scheduled の個人表アクセス: [services/stock_price.rs](../../backend/src/services/stock_price.rs) は `asset_balances` から更新銘柄を選び、[services/fund_price.rs](../../backend/src/services/fund_price.rs) は `fund_balances` から更新ファンドを選ぶ。[services/login_event/history.rs](../../backend/src/services/login_event/history.rs) の `prune_login_events` も [worker_entry.rs](../../backend/src/worker_entry.rs) から呼ぶ | 利用者 ID がない SELECT は更新対象を0件にし価格キャッシュが古いままになる。履歴 DELETE も対象行が見えず、エラーなしで削除件数0となる可能性がある | 価格更新対象の抽出は保守専用の最小列関数/ロールへ、90日より古い履歴の削除は削除専用経路へ分離する。scheduled の共有表(`security_prices`・`fund_prices`・`stock`・各キャッシュ)へのアクセス自体は RLS 対象外 |
| 7 | migration: [bin/migrate.rs](../../backend/src/bin/migrate.rs) と [db/migrate.rs](../../backend/src/db/migrate.rs) | DDL と migration 履歴の読み書きは表 owner のまま実行できる。利用者表を読む/更新する将来の data migration は FORCE と通常ポリシーの対象 | migration 専用の管理接続を Workers の通常接続から分離する。data migration は対象表ごとの検証を加える |
| 8 | バックアップ: [scripts/db-backup.sh](../../scripts/db-backup.sh)、[docs/db-backup.md](../db-backup.md) | `pg_dump` は既定で `row_security=off` にし、RLS がデータを落とす場合は dump をエラーにする。owner だが `BYPASSRLS` でない実行ロールではバックアップが失敗する。SUPERUSER/BYPASSRLS なら FORCE も迂回する | バックアップ専用の管理権限・資格情報をアプリ実行経路と分離し、失敗を監視する |

PostgreSQL は、テーブル所有者に対する FORCE と、SUPERUSER / `BYPASSRLS` の常時迂回を定義している([行セキュリティ](https://www.postgresql.org/docs/current/ddl-rowsecurity.html)、[`pg_dump`](https://www.postgresql.org/docs/current/app-pgdump.html))。`SECURITY DEFINER` は実行権限主体を関数所有者にするが、それ自体は `BYPASSRLS` を付けない。

検索根拠: `rg` で RLS 対象11表の SELECT/INSERT/UPDATE/DELETE と関数呼び出しを `backend/src` 全体に検索し、RLS 有効化は上記4 migration に限定されることを確認した。個人データや接続情報の値は検索・記録していない。DB 状態確認 workflow はカタログと共有表のみを照会し、利用者行を読まないため別経路として追加しない。

### 導入/非導入の比較

| 選択 | 利点 | 残る危険・費用 |
|---|---|---|
| FORCE を入れる | 非 BYPASSRLS のテーブル所有者が通常の DML 経路で利用者条件を誤った場合、policy で行を制限できる | 8つの所有者経路の再配線が必要。管理者・auth 用の例外ロール/関数の権限と戻り値の監査が必要。所有者が FORCE を解除できるため、所有者資格情報の漏えいには防御にならない。`BYPASSRLS` / superuser にも効かない |
| FORCE を入れない | 現在の auth・管理者・cron・migration・backup の挙動を維持し、追加ロールと性能差分を避けられる | owner 接続の SQL 誤りや漏えいでは、RLS がある表でも全行に届く。RLS は通常のアプリ利用者経路に限った防御となる |

脅威モデル `docs/security/threat-model.md` の「DB に直接触れる者(管理者・漏洩した接続情報)」は別境界である。FORCE は誤った DML に対する防御を増やすが、テーブル所有者は FORCE を解除でき、`BYPASSRLS` / superuser も迂回する。接続ロールの分離と DDL 権限の縮小を先行条件とし、資格情報の窃取や DB 管理者からデータを隠す機能とは位置付けない。

### 推奨手順と戻し方

1. 本番変更を伴わない別 Issue で、本番接続ロール属性を読み取り専用手段で確認する。結果を秘密・個人データなしで記録し、Workers の現在ロールが superuser/BYPASSRLS なら、まず通常実行ロールと管理用ロールを分ける。
2. 認証前照会・ログイン upsert/ログアウト・通知 token・管理者検索・cron の更新対象抽出と履歴掃除を、それぞれ専用の `NOLOGIN` 関数所有ロール/限定関数か別の接続ロールへ分ける。関数は `search_path` 固定、`PUBLIC` の EXECUTE を剥奪し、引数と返却列を最小にする。`SECURITY DEFINER` だけで FORCE を回避できる前提にしない。
3. migration とバックアップを Workers の通常接続から分け、別資格情報が RLS 有効全表の完全な dump/restore と data migration をできることを Docker で確認する。
4. RLS 有効11表のうち1表を選び、既存の権限・policyを保ったまま `ALTER TABLE ... FORCE ROW LEVEL SECURITY` を追加する。表 owner・app_restricted・例外関数ロールで成功/拒否/行数が期待通りであることを結合テストで確認する。移行中はその表を触る API/cron のエラー監視を行う。
5. 安定後に次の表へ進む。auth/session と管理者/cron が触る表は、それらの経路を本番に先行配信してから FORCE を有効にする。全11表を個別に確認し、最後に本番のアプリ操作・cron・バックアップ・migration の結果を比較する。

停止時は、対象表だけに対する**新しい増分 migration**で `ALTER TABLE <table> NO FORCE ROW LEVEL SECURITY` を適用する。`ENABLE ROW LEVEL SECURITY`・policy・権限は残す。前の表まで戻さず、失敗した表だけ戻せるようにする。FORCE migration 自体を含むPRは本 Issue の対象外である。

### 性能の見込み

FORCE の DDL 自体はリクエストごとの DB 往復を追加しない。既に `app_restricted` として利用者 ID を設定している 5c/5d の個人経路では、policy の適用は FORCE 前から行われているため、原則として影響は owner バイパス状態から RLS 適用状態へ変わる経路に限る。管理者検索・login event prune・認証前照会では実行計画や返却行が変わる。専用関数/ロールや接続分離を追加すれば、その実装分の往復と直列化が発生し得る。

Issue #1608 の最新計測メモでは scheduled の DB 往復は p50 195ms、fetch の SELECT は p50 44ms / p95 60ms。scheduled は複数の共有表処理に加え login event prune を含むため、この総値を prune 単体の増分とはみなさない。Docker で FORCE 前後の `EXPLAIN (ANALYZE, BUFFERS)` と各経路の経過時間を比較し、本番に進む場合は同等操作の前後値で確認する。現時点では専用経路の設計も本番比較もしておらず、+50ms 以内とは判定しない。

### 使い捨て DB での検証(#1954)

`backend/tests/login_rls.rs` の `force_rls_blocks_remaining_owner_paths_but_keeps_app_restricted_paths` を既存の Docker (`postgres:16-alpine`) 結合テストで実行する。migration を適用した `NOSUPERUSER NOBYPASSRLS` の非特権テーブル所有者を使い、11表すべてに一時的に FORCE を設定する。検証では実データや本番設定を使わない。

- FORCE 前後で `asset_balances` / `fund_balances` から価格更新候補を選ぶ SELECT を直接実行し、FORCE 前の通常 owner では各候補が返り、FORCE 後の通常 owner では0行、`app_enter` 後の `app_restricted` では各候補が返ることを確認する。価格更新関数の成功件数は外部価格取得の成否に左右されるため、遮断判定には使わない。投信の対応行は `fund_name_normalize` で求めた値で投入する。加えて、通常 owner によるセッション照会は行を返さず、OAuth upsert はポリシー違反になり、直接ログアウト DELETE は成功応答でも行を消さず、通知 token 照会/消費は見つからず、管理者横断検索は0行、cron の履歴掃除は0件となることを確認する。
- `row_security=off` の owner 読み取りは SQLSTATE `42501` となることを確認する。これは `pg_dump` が既定で不完全なダンプを避ける挙動に対応する。
- 同じ FORCE 状態で `app_enter` を通した本人のログイン履歴読み取りとセッション削除に加え、価格更新候補の列挙も通ることを確認する。適用済み migration 履歴の再確認も通るが、将来の data migration は未検証。
- `FORCE` の変更はテストコンテナ内だけで破棄する。migration ファイル・本番設定・本番 DB は変更しない。

結果: 上記の所有者経路は現状のままでは動かず、5c/5d の利用者設定付き制限ロール経路は動くことを再現した。`row_security=off` によるバックアップと同等の読み取りは SQLSTATE `42501` で失敗し、既存 migration 履歴の検査は通った。よって全経路の再配線を完了する前の FORCE 導入はしない。

### 実装 Issue の下書き

1. `security: FORCE 前提のアプリ接続ロールと認証前照会の分離` — Workers の通常接続が `NOSUPERUSER NOBYPASSRLS` であることを確認し、必要ならロールを分ける。セッション認証・OAuth upsert/利用者特定を引数と返却値を限定した関数経路へ移す。ログイン・通常認証・ログアウト・アカウント削除を Docker で検証する。
2. `security: 通知 token・管理者横断検索・cron を FORCE 対応` — token peek/consume、管理者の全利用者検索、保有資産を使った価格更新対象の抽出、90日経過 login event 掃除を専用の限定権限経路に移す。権限拒否と監査記録を検証する。
3. `security: migration とバックアップの管理接続を分離` — migration と `pg_dump` / restore の資格情報・権限を通常 Workers 接続から分け、全表の完全なバックアップと data migration を使い捨て DB で検証する。
4. `security: RLS 対象表へ FORCE を段階適用` — 前提3件の完了後、11表に1表ずつ FORCE を追加する migration と Docker 結合テストを実装し、段階ごとの停止/`NO FORCE` 戻し手順を記録する。本番確認は個別の実施判断・承認のもとで行う。

## 23. 第五段階(5e-1)の記録(#1956)

- `backend/migrations/0032_auth_session_user_function.sql`: セッション認証の利用者行取得を
  引数(token_hash のみ)・返却値(利用者行の最小 7 列)に限定した
  `auth_session_user_by_token(bytea)`(`STABLE`/`SECURITY DEFINER`、`search_path` 固定)へ移す土台にした。
  `EXECUTE` は `app_restricted` にだけ与え、`PUBLIC` からは取り上げる(0028 と同じ定型)。
  ロール不在時は `NOTICE` に留めデプロイを止めない。`FORCE` なしのため所有者の挙動は変わらない。
  過去の migration は書き換えていない。
- アプリの変更は `backend/src/services/auth.rs` のみ。`select_user_by_session` は直接の JOIN の代わりに
  上記関数を1回呼び、`delete_session` は直接の DELETE の代わりに既存の
  `auth_delete_session_by_token` を呼ぶ。`select_user_id_by_session`(利用者特定)・
  upsert(`auth_upsert_user`)・回転(`auth_rotate_session`)は 5b/5c で関数化済みのため変更なし。
  アカウント削除は認証照会の解決後に既存の利用者 Tx を使う向きを維持する。
  ログイン Tx(`upsert_user_rotate_session_with_login_event`)の利用者行取得は
  upsert 直後の所有者読み取りから `app_enter` 後の制限 Tx 内の読み取りへ移し、文数は変えていない。
  ログイン・通常認証・ログアウト・アカウント削除の挙動は変えない。
- 通常接続ロールの読み取り専用確認: `DB RLS State` ワークフローの「接続ロール」欄に
  `rolsuper`・`rolbypassrls`・`rolcanlogin` を追加し、`BEGIN READ ONLY` のまま確かめられるようにした。
  期待値は `NOSUPERUSER NOBYPASSRLS`。異なる場合は FORCE 適用(5e-4)の前に
  実行ロールと管理用ロールの分離が必要で、本 Issue では分離用の migration を追加しない。
  関数所有者を分ける専用ロールの新設も FORCE 導入時(5e-4)の対象とし、本 Issue では行わない。
- 本番の通常接続ロールの確認は未実施。本 Issue の実装範囲は確認手段の用意までとし、
  マージ後に統合担当が main から `DB RLS State` ワークフローを手動実行して結果を記録する。
  期待値は `NOSUPERUSER NOBYPASSRLS` で、外れた場合は FORCE 適用(5e-4)の前に
  実行ロールと管理用ロールの分離が必要になる。
- 結合テスト(`backend/tests/login_rls.rs` の `auth_paths_use_limited_functions`):
  非特権所有者が `NOSUPERUSER NOBYPASSRLS` であること、新旧照会関数の権限カタログ
  (`app_restricted` は実行可・`public` は不可)・`DEFINER` 属性・`STABLE`・`search_path` 固定、
  新規・端末付き再ログインを含むログインと通常認証、未知・期限切れ・失効トークンの未認証扱い、
  ログアウトの冪等な成功と行の消去、アカウント削除後の未認証化と利用者行の消去を、
  所有者接続と `app_enter` 後の `app_restricted` の両経路で固定する。
- 往復回数(1リクエストあたり): 認証照会は変更前後とも関数呼び出し1往復(+0)、
  ログアウトの消去も1往復(+0)、ログイン Tx は文の順序のみ変更で文数不変(+0)。
  性能基準 +50ms 以内の追加なし。
- 戻し方はコードの revert が正本。0032 の関数は残しても所有者の挙動を変えず、
  取り消す場合は新しい増分 migration で `DROP FUNCTION` を行う。

## 24. 第五段階(5e-2)の記録(#1957)

- `backend/migrations/0033_force_limited_maintenance_paths.sql` で、
  通知 token・管理者検索・価格候補抽出・履歴掃除の新しい限定関数を追加した。
  株価/基準価額の試行時刻、取得値の反映、投信の取得失敗記録と解除も限定関数にし、
  Worker から共有価格表へ直接書き込まない。
  旧関数と既存の権限は変更せず、`PUBLIC` の `EXECUTE` を剥奪した新関数には
  固定 `search_path` と `row_security=off` を設定する。
- FORCE は `SECURITY DEFINER` の関数所有者にも適用されるため、関数所有者として
  `NOLOGIN BYPASSRLS` の専用ロールを4つ作成した。各ロールは担当表の必要な読み取り列と
  更新権限だけを持つ。`app_restricted` の所属はなく、migration 終了後にアプリ接続ロールから
  SET/INHERIT できない。PostgreSQL は `CREATEROLE` による作成者へ ADMIN membership を
  自動付与するため、その ADMIN option は残る。
- migration 中は所有権移譲に必要な SET/INHERIT と schema CREATE を与え、関数 owner と
  ACL を設定した後、接続ロールの SET/INHERIT と schema CREATE を取り上げる。
- 通知 token peek/consume は `app_restricted` から呼ぶ。管理者検索・価格対象列挙/更新・
  失敗記録と解除・90日経過履歴の削除は `app_worker` から直接限定関数を呼ぶ。
  `app_restricted` には管理者検索と cron 関数の `EXECUTE` を付けない。
  検索フィルターの wildcard escape、
  sort・page、価格の対象日・優先順・上限、履歴の90日境界、削除件数を従来どおり保つ。
  管理者閲覧監査は既存の利用者 Tx を維持する。
- 使い捨て Postgres で全 RLS 表にテスト中だけ FORCE を設定し、旧 owner 経路の空振り/
  拒否、新経路の結果、関数権限・`search_path`・ロール属性、閲覧監査を検証する。
  FORCE を設定する migration は追加していない。

## 25. Workers 接続ロールの切り替え(#1956)

- `0034_app_worker_role.sql` は `app_worker` を `LOGIN NOSUPERUSER NOBYPASSRLS`
  で作成し、パスワードは設定しない。`app_worker` は表の所有者にせず、
  `app_restricted` へ `SET ROLE` できる所属だけを持つ。必要な管理者・cron 関数は
  `app_worker` にだけ直接 `EXECUTE` を付け、表権限は SECURITY DEFINER の所有ロールに限定する。
- `app_restricted` に管理者検索と cron 関数の実行権は付けない。通知 token の関数は
  引き続き app_restricted が呼び出せる。管理者検索を別接続に分けるかは選ばず、
  アプリ側で管理者を確認する方針とする。
- 認証開始時の upsert、セッション照会・削除も、既存関数とその権限を変えず、
  `app_restricted` に切り替えて呼ぶ。旧 Worker が接続する `postgres` の権限や旧関数は
  この PR で変更しない。旧関数の削除は別 PR とする。
- 本番切り替えは利用者が行う。先に Supabase Dashboard の Database > Roles で
  `app_worker` を選び、パスワードを設定する。
- 次に Hyperdrive の既存設定でユーザーとパスワードを更新する。接続先 ID を設定した後、
  シェルのトレースを無効にして `read -s` でパスワードを入力し、画面に表示しない。
  ```bash
  set +x
  read -r -s -p 'app_worker password: ' APP_WORKER_PASSWORD
  printf '\n'
  npx wrangler hyperdrive update <HYPERDRIVE_ID> --origin-user app_worker --origin-password "$APP_WORKER_PASSWORD"
  unset APP_WORKER_PASSWORD
  ```
  パスワードをコマンドへ直接書かず、ログにも貼らない。
- 切り替え後、本番の `/ready` とログインを確認する。戻す場合は同じ Hyperdrive ID に
  `postgres` の資格情報を設定し直す。
  ```bash
  set +x
  read -r -s -p 'postgres password: ' POSTGRES_PASSWORD
  printf '\n'
  npx wrangler hyperdrive update <HYPERDRIVE_ID> --origin-user postgres --origin-password "$POSTGRES_PASSWORD"
  unset POSTGRES_PASSWORD
  ```
- Docker の使い捨て DB で migration と app_worker のログイン、ログアウト、通知解除、
  管理者検索、cron の価格対象抽出/価格反映/失敗記録/履歴掃除、利用者の読み書きを確認する。
  app_restricted から管理者・cron 関数が権限エラーになること、および旧関数が従来の
  呼び方で動くことも確かめる。本番 DB には接続しない。

## 26. 共有表の読み取りと配当・レート制御・マスタの限定書き込み(#1957 レビュー対応)

- `0035_shared_cache_worker_paths.sql` は、Hyperdrive を `app_worker` に切り替えた後に
  動かなくなる共有表の経路(銘柄検索・配当見積もりと更新・マスタ更新)を限定経路に移す。
  共有表(`stock`・`security_prices`・`dividend_per_share_cache`・
  `market_data_provider_rate_control`・`fund_codes`・`fund_prices`)と
  2 計算関数は利用者の行を持たないため、`app_worker` に読み取り(`SELECT`/`EXECUTE`)だけを与え、
  個人表の権限は付けない。読み取りを直接付与にしたのは、共有表の読み取りに
  利用者 ID が要らないため。書き込みは `NOLOGIN BYPASSRLS` の専用ロール
  `app_shared_write_executor` が所有する 9 つの `SECURITY DEFINER` 関数に閉じ込め、
  0033 の方針(Worker から共有表へ直接書き込まない)を維持する。
  `EXECUTE` は `app_worker` と移行期の旧接続(`SESSION_USER`)にだけ付け
  (`app_restricted` には付けない)。旧接続でも新 Worker でも同じ呼び方で動く。
- 残高ビュー(`asset_balances_view`・`fund_balances_view`)は対象外にした。
  両ビューは `security_invoker` なし=所有者権限で動くため、利用者 ID なしの
  `app_worker` 直に `SELECT` を付けると全利用者の残高が見えてしまう。
  アプリの残高読み取りは利用者 Tx 内の `app_restricted` 経由(`WHERE user_id` 付き)の
  既存経路のまま変えない。
- 関数は配当キュー投入・取得反映・失敗記録、レート枠の取得と延長、
  マスタの枠取り・再試行延期・全置換に対応する。再試行延期は枠取りの 7 日後を
  上書きする(延長と違い `GREATEST` にしない)。置換は複数引数の `unnest` のため
  修飾なしで書く(`pg_catalog.unnest` は単一引数しか持たない)。
- 新 owner への `ALTER FUNCTION ... OWNER` には新 owner の schema `CREATE` が要るため、
  0033 と同じく所有権の移譲前だけ与え、最後に取り上げる。
- `backend/tests/login_rls.rs` で `app_worker` の共有表読み取りと直接書き込み不可、
  新関数の新旧接続での実行可と `app_restricted` での実行不可、銘柄検索・配当・
  マスタの新旧接続での動作、枠取りの空振り、置換の件数を固定する。
  `backend/tests/db_integration.rs` の履歴期待値を `0..=35` に更新し、失敗時の
  再試行延期が 1 時間後になることを固定する。

## 27. 残高ビューの RLS 迂回と、索引の効くポリシー式(#1990)

- 問題 1(セキュリティ): `asset_balances_view`・`fund_balances_view` は `security_invoker` なしで、
  所有者(移行ロール)の権限で参照先を読んでいた。`app_enter` 後の `app_restricted` でも、
  ビューを `WHERE user_id` なしで読むと他人の行が見えた(使い捨て DB で再現)。
  アプリは必ず `WHERE user_id = $1` を付けているため実際の漏えいは確認していないが、
  「SQL の書き間違いでも漏れない」という本計画の目的がビューでは成り立っていなかった。
  所有者が BYPASSRLS のため FORCE では塞がらない。
- 問題 2(性能): ポリシー `user_id::text = current_setting(...)` は列側を変換するため索引に使えず、
  不透明な関数で件数の見積もりも崩れる。制限ロールの配当一覧(1 ページ目)は利用者の全行を
  Bitmap Heap Scan で読んでから並べ替えていた(§10・§11 の申し送りの事象)。
  §11 は `::uuid` 形を「空文字が残るとエラーになる」ため不採用にしたが、
  `NULLIF(..., '')` で空文字を NULL にすれば 0 件(fail-closed)のままエラーにならない。
- `backend/migrations/0037_rls_views_invoker_and_indexable_policies.sql`:
  残高ビュー 2 件に `security_invoker = true` を付け、利用者表 11 件のポリシーを
  `user_id = (SELECT NULLIF(current_setting('app.current_user_id', true), '')::uuid)`
  (`users` は `id`)へ `ALTER POLICY` で置き換える(`USING`・`WITH CHECK` とも)。
  先頭に `SET LOCAL lock_timeout = '5s'`。列・権限・ロールは変えない。
  ビューが読む共有表(`security_prices`・`fund_codes`・`fund_prices`)は 0036 で
  `app_restricted` 向けの読み取りポリシーを持つため、呼び出し側の権限でも結合できる。
- 旧 Worker との共存: 旧 `postgres` 接続は BYPASSRLS の所有者なので、ビューを呼び出し側の権限にしても
  これまでと同じ行を読む。`app_worker` 経路は `app_enter` が設定する値の形が変わらないため、
  同じ行を返す(式が索引に効くようになるだけ)。
- 性能(使い捨て `postgres:17`、利用者 3 人 × 配当 3 万行、`app_enter` 後の `app_restricted`、
  `ORDER BY settlement_date DESC, id DESC LIMIT 50`):
  0036 まで 12.5〜13.8ms(Bitmap Heap Scan 3 万行 + top-N 並べ替え、1959 buffers)→
  0037 後 0.25ms(`idx_dividends_user_settlement_date_id` の Index Scan 50 行、53 buffers)。
- 結合テスト(`backend/tests/login_rls.rs` の
  `balance_views_and_user_policies_isolate_rows_for_app_restricted`): 本番と同じく共有表を含む全表で
  RLS が有効な状態で、0037 前のビューでは他人の行が見えること、0037 適用後は
  ビューを `WHERE user_id` なしで読んでも他人の行が 0 件であること、空文字の設定で
  利用者表とビューが 0 件・エラーなしであること、別の利用者名義の INSERT が拒否されること、
  `app_worker` から保有銘柄・投信の一覧(今の検索経路)が本人の行だけで読めることを固定する。
  `backend/tests/db_integration.rs` の履歴期待値は `0..=37`。

### 以後の決まり

- ビューを作る・変えるときは `WITH (security_invoker = true)` を必ず付ける。
  `CREATE OR REPLACE VIEW` を WITH 句なしで書くと付いていた `security_invoker` も外れる。
- 本番には `public` の新しい表へ RLS を自動で有効にする event trigger `ensure_rls` があるため、
  新しい表には読み書きするロール向けのポリシーを同じ migration で必ず付ける
  (無いと NOBYPASSRLS の接続からはエラーなしで 0 行に見える)。利用者の表は 0037 の式、
  共有の表は 0036 の `FOR SELECT TO app_worker, app_restricted USING (true)` に揃える。
- 詳細は `backend/migrations/README.md` の「ビューと新しい表の RLS の決まり」。
