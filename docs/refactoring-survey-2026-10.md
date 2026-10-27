# リファクタリング調査（2026年10月）

この文書は #1569 の段階1の一時的な成果物であり、シリーズの最後の PR で削除する。

調査日: 2026-10-28。基準コミット: `60e86579`。Part of #1569。
製品コード・テスト・API 契約・DB スキーマ・認証認可・UI は変更していない。

## 前提と判断基準

Issue #1569 の「守ること」「調査の観点」、`docs/architecture.md`、ADR 0001・0002 と、以下の呼び出し先まで実装を読んだ。
CSV インポート・保有銘柄・取引履歴・評価損益・配当金の計算仕様は維持する。クエリ・接続・ポーリングを増やす案は採らない。

`git log --since=2026-10-26 --oneline` と PR #1534・#1547 の調査結果を確認した。
#1527 で済んだ死んだコードの削除（#1538・#1567）、公開範囲縮小（#1546・#1559）、一覧取得経路の統合（#1545・#1558）、CSV の共通処理・型・履歴記録の集約（#1548・#1553）、件数表記（#1549）、丸め統一（#1556）、CI 共通化・キャッシュ・分割調整（#1537・#1550・#1552）は再提案しない。
CSV 統計の並行化（#1539）と cron の計測ログ（#1541）も完了済み。以下の SQL 削減案は並行化のやり直しではない。

数値はコードから数えた SQL 発行回数・全行走査の重複であり、応答時間・CPU 時間・DB 費用の実測ではない。
DB 接続・負荷試験・ブラウザ実行はしていない。P2 は発行済みの余分な SQL や全行処理を減らせるもの、P3 は運用判断を誤らせる設計文書の不一致とする。P0・P1 相当の障害はこの調査では確認していない。

## 根拠のある改善候補

おすすめの順は、効果の確実さ・呼ばれる範囲・変更リスクを合わせた順序。各行を独立した小さい PR にする。

| 問題 | ファイルパスと行 | 根拠 | 重要度 | 影響範囲 | 変更リスク | 期待効果 | おすすめの順・対応案 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| CSV 統計の6 SQL と、handler 内の集計・履歴優先ルール | [backend/src/handlers/v1/csv_kind_stats.rs:31–120](../backend/src/handlers/v1/csv_kind_stats.rs#L31-L120) | 5表の COUNT/MAX を並行発行した後、履歴を別途 SELECT。履歴優先・件数0なら日時なし、という業務ルールまで handler が持つ。実 DB テストもこの handler 内関数を直接呼ぶ（[backend/tests/db_integration.rs:3775](../backend/tests/db_integration.rs#L3775)）。 | P2 | CSV 管理の統計 API | 中。5種類の順序、履歴がない場合の MAX(created_at)、全削除後の日時消去、利用者の分離を維持する。 | 業務 SQL を6→1、認証込み7→2。5表の集計走査は残るが往復を5回削減でき、業務ルールと SQL の所在も service に揃う。時間短縮量は未測定。 | 1。既存 services 配下へ集計関数を移し、固定 SQL の UNION ALL と履歴結合で1回にする。トランザクションや追加の事前確認クエリは足さない。 |
| 取引明細が同じ行・検索語を何度も複製・絞り込みする | [frontend/src/features/receipts/view/main_content.rs:31–150](../frontend/src/features/receipts/view/main_content.rs#L31-L150)、[frontend/src/features/receipts/store.rs:78–85](../frontend/src/features/receipts/store.rs#L78-L85)、[frontend/src/features/receipts/filter.rs:247–252](../frontend/src/features/receipts/filter.rs#L247-L252) | `tab_state` は全データを clone する。`filter_receipts` の同一条件の呼び出しが5か所にあり、`Signal::derive` の読み取りごとに再実行される。配当情報の判定・集計・見出し・期間・件数・表がそれぞれ読み、表向けには全行をさらに clone する（[同 main_content.rs:247–252](../frontend/src/features/receipts/view/main_content.rs#L247-L252)）。 | P2 | 取引明細3タブ。特に明細の多い利用者の検索操作 | 中。配当情報に切り替わる条件、検索後の集計、期間、開閉状態、セッション世代を維持する。 | 同一タブ・データ・検索語に対する絞り込みを共有し、複数消費先による全行走査・複製を減らす。実際の再実行回数と ms は未測定。DB 呼び出し数は増やさない。 | 2。タブデータと検索語から絞り込み結果を1つの Memo で導き、判定・集計・期間・件数・表が共有する。件数等は借用で読み、全データ clone を避ける。全画面の store 統合はしない。 |
| 資産管理の検索で、検索に依存しない全保有分の変換・集計も再計算する | [frontend/src/features/asset_balance/store.rs:184–209](../frontend/src/features/asset_balance/store.rs#L184-L209)、[frontend/src/features/asset_balance/view/main_content.rs:136–156](../frontend/src/features/asset_balance/view/main_content.rs#L136-L156) | `filtered_portfolio` が絞り込み後と全保有分の両方を変換する。検索 signal を読む Effect が毎回これを呼び、`filtered.set` により全保有分の summary Memo も再計算する。初回も手動計算と Effect で同じ処理がある。 | P2 | 資産管理の国内株式一覧の検索・集計 | 中。lookup 更新時の反映、検索中も全保有分の合計を維持する仕様、取得上限超過時の summary フォールバックを維持する。 | 検索入力ごとの全保有分の変換・集計を不要にし、初回の二重計算も減らす。検索対象の絞り込みは残す。通信回数は変えない。 | 3。全保有分の変換結果を rows・lookup・世代に依存させ、検索結果と分ける。summary は全保有分と配当だけを読む。Effect による派生状態の書き戻しを必要な件数通知に絞る。 |
| 資産管理の件数・有無の判定だけで全行を複製する | [frontend/src/features/asset_balance/view.rs:176–198](../frontend/src/features/asset_balance/view.rs#L176-L198)、[frontend/src/features/asset_balance/store/csv.rs:388–426](../frontend/src/features/asset_balance/store/csv.rs#L388-L426) | タブ件数と並べ替え欄の有無を調べるたびに `resolve_*` を呼ぶ。成功時の `resolve_balance` は rows 全体を clone して `Row::Saved` に変換してから返し、呼び出し側はその長さだけを使う。 | P2 | 資産管理の株式・投信・すべてのタブ件数と操作欄 | 低〜中。世代不一致・未取得・失敗時の扱い、株式の取得行数と投信の total 優先という現行の違いを維持する。 | 件数・有無の読み取りを O(N) の複製から O(1) の参照にする。表示データ作成のための変換は残す。 | 4。件数と有無は元 slot を `with` で読んで算出する。件数を得るための全行変換を除き、API や件数仕様は変えない。 |
| 投信の自動更新状態で同じ結合を失敗理由取得のために再度読む | [backend/src/services/fund_balance.rs:94–160](../backend/src/services/fund_balance.rs#L94-L160)、[同:194–205](../backend/src/services/fund_balance.rs#L194-L205) | 状態集計が fund_balances・fund_codes・fund_prices を読み、失敗件数が1以上なら同じ3表をもう一度結合して最後の失敗理由を取る。summary の要求ごとに発生する。 | P2 | 投信残高一覧の summary、失敗状態を表示するホーム等 | 中。口座をまたぐ DISTINCT、最新失敗の並び順と NULL、JST 分表示、状態は検索条件に依存しない仕様を維持する。 | 失敗ありの状態取得を2→1 SQL。facets なしの一覧は業務5→4 SQL、認証込み6→5。失敗なしは現行と同数。 | 5。状態集計 SQL で失敗理由も選び、同じ絞り込み範囲の結果から返す。検索条件つきの金額 summary と利用者全体の更新状態を混同しない。 |
| 現行構成の早見表と ADR の現行方針が実装と違う | [docs/architecture.md:28](architecture.md#L28)、[同:120–127](architecture.md#L120-L127)、[docs/adr/0001-keep-axum-and-harden-api.md:81](adr/0001-keep-axum-and-harden-api.md#L81)、[同:110](adr/0001-keep-axum-and-harden-api.md#L110) | 現行説明に SQLx、起動完了ゲート、TraceLayer、CompressionLayer がある。実際の DB は tokio-postgres（[backend/src/db.rs:1–2](../backend/src/db.rs#L1-L2)）、本番 router の層は [backend/src/worker_entry.rs:100–120](../backend/src/worker_entry.rs#L100-L120) で、記載された3層はない。 | P3 | 設計・障害調査・後続 PR のレビュー | 低。文書のみ。ただし過去の Context の記録と現行の方針は区別する。 | 存在しないゲートやトレース・圧縮を前提に運用・改修する誤判断を防ぐ。性能効果は主張しない。 | 6。architecture の現行説明と ADR の Architecture Stance を実装に合わせる。記述に合わせて middleware や別の DB ライブラリを追加しない。DB 移行先の記述は #1568 と整合させる。 |

## DB: 1リクエストのクエリ数

通常成功時の読み取り経路を数えた。以下の「業務 SQL」は HTTP の認証を除く。認証付き API は、Cookie が有効なら sessions と users の結合 SELECT をさらに1回発行する（[backend/src/extractors/auth.rs:44–48](../backend/src/extractors/auth.rs#L44-L48)、[backend/src/services/auth.rs:119–134](../backend/src/services/auth.rs#L119-L134)）。
以下は SQL 文の発行回数であり、1 SQL 内の結合・走査の数や DB サーバーの接続数ではない。

| 経路 | 業務 SQL / リクエスト | 根拠・条件 |
| --- | --- | --- |
| CSV 種類統計 | 6 | 5表の COUNT/MAX と履歴1回。認証込み7。改善候補1。 |
| 配当・株式取引・投信取引の一覧 | 2＋summary があれば1＋facets の個数 | COUNT と data は常時。facets は配当5、株式取引4、投信取引4。最大は順に8・7・7（認証込み9・8・8）。[backend/src/services/domain/search.rs:116–146](../backend/src/services/domain/search.rs#L116-L146)、[同:160–196](../backend/src/services/domain/search.rs#L160-L196)、[配当:77–91](../backend/src/services/dividend.rs#L77-L91)・[株式:71–84](../backend/src/services/domestic_stock.rs#L71-L84)・[投信:75–88](../backend/src/services/mutualfund.rs#L75-L88) の FACETS 定義。行数に比例して SQL が増える N+1 ではない。 |
| 株式残高一覧 | 2＋summary があれば1＋facets があれば1 | data/summary は asset_balances_view、COUNT/facet は asset_balances を読む。[backend/src/services/asset_balance.rs:100–158](../backend/src/services/asset_balance.rs#L100-L158)。最大4（認証込み5）。 |
| 投信残高一覧 | 2＋summary があれば2〜3＋facets があれば2 | summary は金額集計1＋更新状態1＋失敗あり時の理由1。最大6〜7（認証込み7〜8）。改善候補5。 |
| 1株配当の一括照会 | 非空コード列なら1〜2 | `ANY($1)` でキャッシュを1回読み、更新対象があり J-Quants 設定済みの場合だけ一括 enqueue を1回。空コード列は0。[backend/src/services/dividend_cache.rs:39–103](../backend/src/services/dividend_cache.rs#L39-L103)。コードごとの SELECT はない。 |
| 公開の銘柄検索 | 1 | [backend/src/services/stock.rs:21–36](../backend/src/services/stock.rs#L21-L36)。認証 SQL なし。 |
| `/ready` | 1 | SELECT 1（[backend/src/worker_entry.rs:77–94](../backend/src/worker_entry.rs#L77-L94)）。`/health` は0。外部からの監視頻度は未確認。 |

実際の frontend の全ページ取得は summary を先頭ページだけで要求する。facets は既定 false で、全ページ取得側も要求していない（[frontend/src/support/pagination.rs:69–85](../frontend/src/support/pagination.rs#L69-L85)、[shared/src/common.rs:87–94](../shared/src/common.rs#L87-L94)）。したがって一覧の最大値が通常の画面表示で毎回発生するとは扱わない。
COUNT・data・summary は同じデータを別目的で読むが、空ページの total と集計の意味を守る必要がある。ここを一括で変える提案はしない。
CSV 書き込み側も [backend/src/services/domain/bulk.rs:173–221](../backend/src/services/domain/bulk.rs#L173-L221) の UNNEST 一括処理を使う。利用者ロック・上限確認・全置換のトランザクションは必要で、行ごとの INSERT に変えない。

## DB: 接続と毎分 cron

### 接続の単位

本番の fetch と scheduled はイベントごとに `build_state` し、`Db::from_hyperdrive` は接続情報だけを持つ。最初の SQL で Hyperdrive への接続を作り、同一 Db 内で再利用する（[backend/src/worker_entry.rs:123–174](../backend/src/worker_entry.rs#L123-L174)、[backend/src/db.rs:348–424](../backend/src/db.rs#L348-L424)）。state 構築だけで DB が起きるとは数えない。

読み取りの通常経路は接続が切れなければ1イベントで1クライアントを再利用する。書き込みの `begin` は共有接続での混線を防ぐため独立接続を作る（[backend/src/db.rs:439–463](../backend/src/db.rs#L439-L463)）。認証後にトランザクションを使う経路は共有接続と tx 接続の2系統になり得る。
これは Worker→Hyperdrive の接続数で、Hyperdrive→DB の物理接続数ではない。Neon の稼働時間・同時接続数・Hyperdrive の再利用状況は未測定。isolate をまたぐ Socket キャッシュや接続確認用の SELECT 追加は提案しない。

### 更新対象なしでも発行する SQL

[backend/wrangler.toml:44–45](../backend/wrangler.toml#L44-L45) は毎分1本。scheduled は以下の4処理を順に呼ぶ（[backend/src/worker_entry.rs:209–258](../backend/src/worker_entry.rs#L209-L258)）。
以下は正常に予定どおり届き、配当キューが空・マスタ期限が未来・価格更新対象が0件の1環境を仮定した数。更新本体・再試行・失敗時の SQL は別である。

| 処理 | DB に入る条件 | 対象なし時の SQL | 予定どおりの1日分 |
| --- | --- | --- | --- |
| 配当キュー | J-Quants クライアントあり。時刻による間引きなし | 対象コード SELECT を毎分1回 | 1,440 |
| 銘柄マスタ | 同クライアントあり、有効な scheduled 時刻 | 次回更新可能時刻 SELECT を毎分1回。期限未来なら終了 | 1,440 |
| 株価 | scheduled の分が5の倍数 | 保有コードと共有価格の候補 SELECT を1回 | 288 |
| 投信基準価額 | scheduled が JST 20:00 / 23:00 | 保有ファンド・コード対応・共有価格の候補 SELECT を1回 | 2 |

根拠: 配当は [backend/src/services/dividend_cache.rs:113–138](../backend/src/services/dividend_cache.rs#L113-L138)、マスタは [backend/src/services/stock_master.rs:119–128](../backend/src/services/stock_master.rs#L119-L128)・[同:171–190](../backend/src/services/stock_master.rs#L171-L190)、株価は [backend/src/services/stock_price.rs:244–265](../backend/src/services/stock_price.rs#L244-L265)・[同:340–342](../backend/src/services/stock_price.rs#L340-L342)、投信は [backend/src/services/fund_price.rs:329–350](../backend/src/services/fund_price.rs#L329-L350)・[同:437–442](../backend/src/services/fund_price.rs#L437-L442)。

J-Quants 設定ありなら、対象0でも計3,170 SQL/日。通常の分は2、5分境界は3、JST 20:00/23:00 は4で、毎分 DB に到達し得る。読み取り用クライアントは最大1,440イベント分作られる。これが実際に DB の休止を妨げる量・費用は未測定で、#1568 の移行調査と区別する。
J-Quants 未設定なら配当とマスタは既に接続前に終了する（[backend/src/worker_entry.rs:285–301](../backend/src/worker_entry.rs#L285-L301)）。残り290 SQL/日、SQL を発行するイベントは288回/日（投信の2回が5分境界に重なる）。

### 間引き条件の判断

- 株価の5分境界以外、投信の2時刻以外、J-Quants 未設定時の配当・マスタは、既に DB の手前で間引かれている。候補コード0件なら各処理は書き込み・外部 API 呼び出しを行わない。
- 配当キューが空、全価格が最新、マスタ期限が未来という条件は DB を読んだ後にしか分からない。現在の構成でこれを事前判定する別の SELECT は SQL を増やすため採らない。
- マスタは週次更新・失敗時1時間待ちだが、期限の読み取りは毎分である。単に時刻ゲートを追加すると、任意の分に来る初回投入や再試行の開始が遅れる。配当を5分おき等にすると enqueue 後の処理開始が遅れ、frontend の pending 再照会にも影響する。外部仕様を維持する段階2の候補には入れない。
- 土日・夜間でも未取得銘柄や新規 CSV に対する更新はあり得る。営業時間だけを根拠に止めない。稼働時間を変えずに休止効果を保証できる追加の間引き条件は、このコード調査では確認できなかった。

## 境界の確認と、やらないと決めたこと

| 対象・案 | 判断と理由 |
| --- | --- |
| `frontend/src/api/` から業務計算を移動する | 見送り。`client.rs` は URL・Cookie 付き送信・タイムアウト・リトライ・エラー変換、`dto.rs` は shared wire 型の再公開と別名が中心。CSV 処理は support/features、評価額や税は model/shared にある。api 配下に持株・税・配当の計算が混ざる実例は確認できなかった。 |
| ApiError の表示文言だけを別層へ移す | 見送り。`client.rs:61–115` の案内文とバッジ文言の違いは `frontend/README.md` の共通処理表でも意図が明記されている。表示の不具合を確認しておらず、文言を移すだけでは今回の問題を解消しない。 |
| handler を行数だけで分割する、全 handler の service 化 | 見送り。一覧は `handlers/common.rs:17–30` → `services/domain/search.rs`、CSV は `handlers/v1/csv_import.rs:35–72` → `services/csv/import.rs` と domain に委譲済み。業務判断がまとまって残っている統計だけを候補1にした。Multipart・Cookie・HTTP 応答・OpenAPI 用 wrapper は HTTP 側に置く。 |
| すべての `Signal::derive` を Memo にする、Effect を全廃する | 見送り。軽い件数・フラグにキャッシュは必須ではない。セッション世代の切り替え・非同期処理の開始など副作用のある Effect は必要。重複走査と不要な全行複製を実際に辿れた箇所だけを候補にした。 |
| 全一覧の COUNT・summary・facets の全面統合、カーソル API への変更 | 見送り。facets は通常 off、summary は先頭ページのみ。#1534 でも count/summary 等は記録に留めている。契約を維持する今回の目的に対し変更リスクが大きく、空ページの total や国内株式の日次課税集計を壊す恐れがある。 |
| Repository trait・DI コンテナ・全面クリーンアーキテクチャ | 採用しない。Issue と ADR 0001 の明示的な非対象。既存の関数と AppState で足り、今回確認した SQL と再計算を減らさない。 |
| stock と価格・配当キャッシュの表統合、FK の追加 | 採用しない。ADR 0002 のコード空間と更新周期の違いを実装でも確認。DB スキーマ変更は禁止で、任意コード・週次全置換・失敗再試行の意味を壊す。 |
| cron の一律低頻度化、isolate のメモリだけで期限や空キューを記憶する | 採用しない。上記の開始時刻・世代をまたぐ状態の問題がある。既存のレート予約・cooldown・更新失敗時の前回値維持も削らない。計測ログ追加は #1541 済みで再提案しない。 |
| DB 移行・接続先・認証・計算仕様・UI の修正 | やらない。DB 移行は #1568、他は #1569 の保護対象。この調査では追加のスキーマ・API・画面変更を必要とする問題を候補にしない。 |

## 確認の範囲と後続 PR

この段階ではソース・既存テストの読み取り、履歴との突き合わせ、fmt と Markdown の確認に限定する。既存テストの変更・削除・無効化、手元 E2E、DB 実行、性能計測は行わない。UI を変えないため撮影用 spec・画像も作らない。

後続の実装 PR では、候補1は `backend/tests/db_integration.rs` の CSV 統計（5種類・履歴フォールバック・利用者分離・削除後）、候補5は投信状態の既存テストを維持し、SQL 発行数が減ることを確かめる。frontend の候補は検索・集計・世代切り替えの挙動を維持し、全行処理の回数と clone の削減を変更前後で確認する。Memo 化だけで速度改善したとは判断しない。重い E2E は CI に任せる。
