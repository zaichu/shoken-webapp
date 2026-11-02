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
