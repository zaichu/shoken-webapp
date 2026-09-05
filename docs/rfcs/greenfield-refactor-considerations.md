# グリーンフィールド作り直し検討（RFC）

- 作成日: 2026-09-05
- ブランチ: `docs/greenfield-refactor-proposal`
- 性質: **設計検討のみ。コード変更なし**。結論を誘導しない。
- 対象: 個人用証券ポートフォリオ管理 Web アプリ（Google OAuth / 資産管理・取引明細・配当金 / J-Quants API 連携）

## 1. 概要（結論を先に書く）

**結論: フルスクラッチの作り直しは正当化できない。現行スタック（axum + sqlx + React + Vite + Neon + Fly.io + Vercel）を維持し、段階的な部分改善に留めるのが妥当。**

- 作り直す価値があるか: **なし（現時点では）**。理由は (a) 現行構成が 7 観点のいずれでも致命的な欠陥を持たない、(b) 作り直しコスト（API 20 パス前後・CSV 取り込み・J-Quants 連携・OAuth・E2E の再構築）に対して得られる利益が小さい、(c) 運用コスト面で現行構成からの明確な改善策がない、ため。
- あるならどこからか: 仮に手を入れるなら優先順位は (1) バックエンドの層責務の明確化と重複クエリの共通化（`search_filters.rs` / `facets.rs` / `bulk_helpers.rs` 周りの整理）、(2) ゼロコスト観点の小改善（`async_trait` の削減、`Arc<Secrets>` の見直し等の検証）、(3) テストの実行時間・安定性の改善、の順。いずれも作り直しではなく既存コード上のリファクタで到達できる。
- 変えないもの: Web フレームワーク（axum 維持）、DB（Neon Postgres 維持）、認証方式（Google OAuth + DB セッション維持）、デプロイ先（Fly.io + Vercel 維持）、フロント基盤（React 19 + Vite + TanStack Query + Tailwind 4 維持）。

## 2. 現行アーキテクチャの棚卸し（強み / 弱み）

調査範囲: `CLAUDE.md`、`backend/CLAUDE.md`、`backend/.claude/rules/00-backend.md`、`frontend/CLAUDE.md`、`frontend/.claude/rules/00-frontend.md`、`docs/agent-roles.md`、`.claude/skills/deploy/SKILL.md`、`backend/Cargo.toml`、`docs/openapi.json`（先頭およびパス数）、`backend/src/` 構成、`frontend/package.json`、ワークフロー・`fly.toml` の要点。

### 2.1 全体像

- Backend: Rust + axum 0.8 + sqlx 0.9（postgres / chrono / uuid / rust_decimal）+ tokio full + reqwest + oauth2 + utoipa（OpenAPI 生成）。`handlers / services / models / extractors / middleware / state / errors` の層構成。
- Frontend: React 19 + React Compiler + TypeScript（strict）+ Vite 7/8 系 + Tailwind 4 + TanStack Query 5 + react-router-dom 7。Atomic Design + features 分割。API 型は `docs/openapi.json` 正本 → `src/generated/api.ts` 生成。
- DB: Neon Postgres。マイグレーションは `backend/migrations/`（現時点で 5 ファイル: 初期スキーマ + 検索系インデックス追加中心）。
- 認証: Google OAuth2（`oauth2` クレート）+ DB セッション + Cookie（HttpOnly 必須、本番 Secure、SameSite 使い分け）。
- デプロイ・CI: Fly.io（backend、リージョン nrt、256MB shared、常時 1 台維持）+ Vercel（frontend）+ GitHub Actions（backend: `backend/**` 変更時のみデプロイ、frontend: PR 単位 CI + プレビュー）。`main` マージで自動デプロイ。
- API 規模: `docs/openapi.json` は約 4147 行、パス数は約 20（`/api/v1/...` 系: 認証・アカウント、資産残高、配当、国内株、投信、市場データ、J-Quants 系等）。「全部読む必要はない」規模だが、小規模〜中規模の個人アプリとしては十分育っている。

### 2.2 強み

- **層構成が素直**: handlers（HTTP）/ services（ビジネスロジック）/ models（型・バリデーション）/ extractors（認証・検証済み JSON 等）の分離があり、どこに何を書くかの迷いが少ない。`errors.rs` の統一エラー型 + `IntoResponse` も整備済み。
- **SQL 戦略が一貫**: 動的フィルタ付き一覧検索は `sqlx::QueryBuilder<Postgres>` + `push_filters` 共通ヘルパー（`search_filters.rs`、`facets.rs`）に寄せており、文字列連結ではなくパラメータバインディングを使う方針がルール化されている。バルク系は `bulk_helpers.rs` に集約されつつある。
- **型安全性の意識が高い**: `rust_decimal`（金額）、`chrono`、`uuid` を sqlx の features として有効化し、金額を float で扱わない方針。frontend は TS strict + React Compiler + openapi-typescript 生成型で契約同期。
- **運用が個人規模に適合**: Fly 最小 VM + Neon + Vercel + 自動デプロイで、手運用がほぼ不要。レート制限（governor）、CORS 明示、Fly Secrets 運用、Sentry 導入など、公開アプリとしての最低限が揃っている。
- **テストの多層化**: `cargo test`（単体 + `backend/tests/db_integration.rs` + testcontainers）/ vitest + RTL / Playwright E2E（auth/CSV/検索/エラーパス/a11y）と、各層にテストが存在する。

### 2.3 弱み・気になる点（作り直し理由にはならないが改善余地）

- **services の肥大・重複の香り**: `services/` に `asset_balance.rs` / `dividend.rs` / `domestic_stock.rs` / `mutualfund.rs` / `stock.rs` と並列ドメインが並び、各々に `push_filters` + `QueryBuilder` 組み立て + ファセット取得が存在する。`search_filters.rs` / `facets.rs` で共通化が始まっているが、完全かは未検証（本 RFC ではコード精読をしていない）。
- **`async_trait` の使用**: `services/csv_domain.rs` および `handlers/csv_import.rs` のテスト用トレイトで `async_trait` を使用。Rust 1.75+ の RPITIT（`async fn` in trait）で代替可能な可能性があり、ゼロコスト観点では要検証（詳細は §4.1）。
- **`Arc<Secrets>` など共有状態の粒度**: `AppState { pool: PgPool, secrets: Arc<Secrets>, client: reqwest::Client }` は実務上一般的だが、`Secrets` 全体を `Arc` で回す必要性や、`String` フィールドの clone 頻度は未計測。
- **sqlx マクロ vs QueryBuilder の二系統**: `bulk_helpers.rs` のコメントにある通り、`sqlx::query!` はリテラル必須のため動的 SQL と相性が悪く、QueryBuilder 系と使い分けが発生する。コンパイル時検証の恩恵が動的クエリでは薄れる構造的制約がある。
- **フロントの二重テスト基盤の重さ**: vitest（`NODE_OPTIONS=8GB` 指定あり）+ Playwright E2E 多数。個人規模にしては E2E が充実している反面、実行時間・メモリ・フレーキー管理のコストがある。
- **依存の重さ**: backend は axum-extra / tower-http / governor / sentry / oauth2 / reqwest / futures / csv / utoipa 等、個人規模には十分だが「最小」ではない。ただし削れるものはほぼない（後述）。

## 3. 制約の解釈（ゼロコスト抽象化・運用コストゼロを具体的にどう適用したか）

### 3.1 制約 1: Rust かつゼロコスト抽象化を設計原則として守る

本 RFC における解釈:

1. **静的ディスパッチ優先**: `dyn Trait` は「本当に型が実行時まで定まらない境界」（エラーの箱化、プラグイン的拡張点、テストダブル等）に限定し、それ以外は generics + monomorphization（`impl Fn(&mut QueryBuilder<Postgres>)` のような引数位置 impl Trait を含む）で済ませる。現行コードの `push_filters: impl Fn(...)` 形式はこの原則に適合している。
2. **不要な確保・clone の回避**: リクエスト毎の `String`/`Vec` の余分な clone、特に認証・バリデーション・クエリ組み立て経路での複製を疑う。ただし「読んだだけで clone を断罪しない」——計測（ベンチ・プロファイル）なしに最適化提案はしない。
3. **型で不変条件を保証**: 金額は `rust_decimal`、ID は `Uuid`、時刻は `chrono` 型、日付・区分は enum/newtype で表現し、バリデーションは `validator` + 抽出器（`ValidatedJson` 等）の境界で一回だけ行う。以降の層では「検証済み」前提で二重チェックしない。
4. **async との折り合い（正直な評価）**: async（tokio）は executor・タスク・waker 等のランタイムコストを伴い、厳密な意味での「ゼロコスト」ではない。しかし本アプリは「同時アクセス少数・I/O 待ち中心（Neon Postgres、J-Quants API、OAuth）」であり、スループットより **I/O 多重化とコードの直線性** が支配的。少数の同時接続でもブロッキング I/O でスレッドを占有する同期モデルより、tokio + `PgPool` の非同期プールの方が資源効率が良い。よって **async 否定は非現実的** であり、ゼロコスト原則は「async の枠内で無駄を払わない」（不要な `spawn`、`clone`、`Arc`、`Box`、過剰なバッファリングの排除）に適用するのが正しい。本 RFC はこの立場を取る。

適用結果の先取り: 現行の axum / sqlx / tokio / reqwest 選択は、上記解釈のもとで **妥当**。フレームワークを変えてもゼロコスト性は本質的に改善しない（§4.1 参照）。

### 3.2 制約 2: 運用コストを実質ゼロに近づける

本 RFC における解釈:

1. **無料枠内が正**: 個人利用・同時アクセスごく少数なら、計算・DB・ホスティング・CI のいずれも無料枠（または数百円/月程度の最小枠）に収めることを成功条件とする。性能・冗長性より **「放置できること」「請求が立たないこと」** を優先する。
2. **現行維持が第一候補**: 現行（Fly 最小 VM + Neon + Vercel + GitHub Actions 無料枠）が既に無料枠近辺に収まっていると見られるため、安易な移行提案はしない。移行コスト（DNS・シークレット・マイグレーション経路・CI 書き換え・枯れた運用知見の破棄）は「見えない運用コスト」である。
3. **機能削減は最終手段**: 無料枠に収めるための機能削減は、削る機能と影響を明記した上でしか提案しない。本検討では削減提案に至らなかった（§4.5 参照）。
4. **コールドスタート等の微コストは受容**: `auto_stop_machines = 'stop'` + `min_machines_running = 1` のように、コールドスタート回避とコストのバランスは現行で既に取られている。これを崩す提案はしない。

## 4. 各観点の検討

### 4.1 観点 1: バックエンドのアーキテクチャ（axum + sqlx 生SQL/QueryBuilder 中心 vs 他の選択肢）

**現行**: axum 0.8 + sqlx（`query!` マクロと `QueryBuilder` の併用）+ `handlers / services / models` 層。

**代替候補と評価**:

| 候補 | 評価 |
|---|---|
| Actix-Web / Rocket 等への乗換 | 否定的。axum は tokio エコシステムとの親和性・メンテナンス状況・型安全性（extractor による境界保証）のバランスが良く、個人規模では乗換利益がない。パフォーマンス差も本アプリのボトルネック（DB・外部 API 待ち）に対して誤差。 |
| SeaORM / Diesel 等の ORM 導入 | 否定的。動的フィルタ・ファセット・バルク UPSERT 中心のクエリ形状に対して、ORM は抽象化の漏れ（結局生 SQL に落ちる箇所）と学習・運用コストを増やす。現行の QueryBuilder + 共通ヘルパーは ORM が得意とする「CRUD の定型化」より本アプリの実態に合っている。 |
| sqlx 維持 + 共通化の推進 | 肯定的。`search_filters.rs` / `facets.rs` / `bulk_helpers.rs` への寄せ方を徹底し、各ドメイン service の `push_filters` と一覧取得の重複を減らす。これは作り直しではなく段階リファクタで可能。 |
| レイヤー構成の変更（例: repository 層の新設、CQRS 等） | 否定的（現時点）。`handlers / services / models` は妥当で、層を増やすと個人規模には過剰。問題があるとすれば層の数ではなく services 内のドメイン間重複であり、層追加ではなく共通化で解決すべき。 |

**ゼロコスト観点の査定（現行コードの実査に基づく）**:

- 動的ディスパッチ: アプリ本体での `dyn` 使用は `bin/import_csv.rs` の `Box<dyn Error>`（CLI バイナリのエントリのみ）と tracing テスト内の `dyn Debug` 程度で、ホットパスに `dyn Trait` は見当たらない。`push_filters: impl FnOnce/Fn(&mut QueryBuilder)` は静的ディスパッチであり適合的。→ **良好**。
- `async_trait`: `services/csv_domain.rs` と `handlers/csv_import.rs`（テスト内）で使用。少なくとも新規コードでは RPITIT（`async fn` in trait）への置換可否を検討する価値がある。ただし `mockall` との組み合わせでは `async_trait` が必要な場合があり、機械的な除去は不可。→ **小改善候補（要検証）**。
- ヒープ確保・clone: `Secrets { database_url: String, ... }` を `Arc<Secrets>` で共有する形は一般的だが、`Secrets` 全体を Arc で包む必然性（各ハンドラが実際に触るのは一部）や、リクエスト毎の clone 箇所の有無は本 RFC では未計測。→ **計測なしに断定しない。将来のプロファイル対象**。
- 型による保証: `rust_decimal` / `Uuid` / `chrono` の使用、`validator` による境界バリデーション、パラメータバインディングの徹底は原則に適合。→ **良好**。
- async の折り合い: §3.1 の通り、tokio + 非同期プールは本アプリの I/O 待ち特性に適合しており、ランタイムコストは受容すべきトレードオフ。`tokio::spawn` の乱用や不要なバックグラウンド化が見られない限り問題なし（配当キャッシュの `AtomicBool` 二重起動防止など、既に節度ある実装）。→ **妥当**。

**結論（観点 1）**: 枠組みは変えない。やるなら services 内の重複排除と `async_trait` の要否検証に留める。

### 4.2 観点 2: フロントエンド（React + Vite + TanStack Query + Tailwind）

**現行**: React 19（Compiler 有効）+ TS strict + Vite + Tailwind 4 + TanStack Query 5 + react-router-dom 7。Atomic Design + features 分割。fetch ベースの共通 API クライアント + TanStack Query によるサーバー状態管理。契約は openapi-typescript 生成型。

**評価**:

- React 継続は妥当。個人の証券管理 UI（表・検索・CSV 取り込み・配当表示）はフォームと一覧中心で、React のエコシステム（RTL、Playwright、TanStack Query）の蓄積が大きい。Svelte / Solid / HTMX 等への乗換は、学習・再実装コストに対して利益がほぼない。
- Vite 継続は妥当。ビルド高速で Vercel との相性も良い。Rspack 等への乗換は測定可能な課題がない限り不要。
- TanStack Query 継続は妥当。キャッシュ・再フェッチ・楽観的更新が自前実装不要になり、CSV 取り込み後の再取得のような処理と相性が良い。素の fetch + useState に戻す理由はない。
- Tailwind 4 継続は妥当。`@theme` / `@layer components`（`.panel-card` 等）/ `cn()` の運用ルールが既に文書化されており、作り直して得られるものがない。
- React Compiler 前提（`useMemo` / `useCallback` 原則不要）は現代的で、ゼロコスト的な発想（不要な抽象のコストを払わない）とも方向が一致する。
- 気になる点: `package.json` の `typescript-next`（7.0.1-rc）併用や `NODE_OPTIONS=8GB` のメモリ指定は、ビルド・テスト環境の重さを示唆する。作り直しではなく、依存の棚卸し（rc 版の追従要否）とテスト分割で対応すべき範囲。

**結論（観点 2）**: 変える点なし。強いて言えば依存の rc 追従とテスト実行の軽量化の検討のみ。

### 4.3 観点 3: 認証・セッション管理（Google OAuth + DB セッション + Cookie）

**現行**: Google OAuth2（`oauth2` クレート）+ DB セッション + Cookie（HttpOnly 必須、本番 Secure、SameSite はクロスオリジン `None` / ローカル `Lax` の使い分け）。CORS は origin 明示 + 必要時のみ credentials。ルールは `backend/.claude/rules/00-backend.md` に明文化。

**代替候補と評価**:

| 候補 | 評価 |
|---|---|
| JWT（stateless）に乗換 | 否定的。個人規模では失効管理・リフレッシュ・鍵ローテーションの複雑さが増すだけで利益が薄い。現行の DB セッションは「ログアウト・退会時の即時無効化」が素直で、同時接続少数なら DB 負荷も問題にならない。 |
| Auth0 / Clerk 等の外部 IdP サービス | 否定的。Google OAuth 直接連携で足りており、外部サービスは無料枠・ベンダーロック・月額の不確実性を増やす。運用コストゼロ制約に逆行する。 |
| パスキー / メールリンク等の追加 | 否定的（現時点）。利用者が本人のみ（個人用）の前提では認証手段を増やす意味が薄い。 |
| 現行維持 + ルールの遵守徹底 | 肯定的。Cookie 属性・CORS・シークレット管理（Fly Secrets / `.env` 分離）・ログへの機密情報非出力が既にルール化されている。これを守り続けることが最良の投資。 |

**結論（観点 3）**: 現行方式は妥当。変更不要。

### 4.4 観点 4: データベース（Neon Postgres 前提、スキーマ・マイグレーション運用）

**現行**: Neon Postgres + sqlx マイグレーション（`migrations/`、5 ファイル）。インデックス追加系のマイグレーションが中心で、検索性能への意識がある。API 契約正本（openapi.json）と生成型の同期ルールもある。

**評価**:

- Neon 前提は妥当。個人規模・同時接続少数では、マネージド Postgres の無料枠 + 自動スリープ + ブランチ機能は運用コストゼロ制約に適合する。自前 Postgres（Fly Postgres / VPS 上の 直接運用）はバックアップ・アップデート・監視の手間が増えるだけで利益がない。SQLite（単一ファイル）化は「安い」が、現行の Postgres 固有 SQL・sqlx 運用・Neon ブランチ運用を捨てるコストに見合わない。
- スキーマ設計: 金額 `DECIMAL`、ID `UUID`、時刻 `timestamptz` 等の型選定は堅実。検索系インデックスの追加履歴があり、N+1 や全表走査への警戒はあると見られる（精読はしていないため断定しない）。
- マイグレーション運用: sqlx 標準 + `make sqlx-prepare`（オフライン準備）の運用があり、CI との整合も取れている。デプロイ前チェックリストに「マイグレーションが必要な場合は先に実行」が含まれている。改善余地としては、マイグレーション適用の自動化度合い（デプロイ時自動実行か手動かの明文化）と、破壊的変更時の手順の文書化くらいであり、仕組みの作り直しは不要。

**結論（観点 4）**: Neon 維持。やるならマイグレーション適用手順の明文化程度。

### 4.5 観点 5: デプロイ・CI/CD（Fly.io / Vercel / GitHub Actions、無料枠との関係）

**現行**: backend は Fly.io（nrt、256MB shared、常時 1 台、auto stop/start）+ `backend/**` 変更時のみ自動デプロイ。frontend は Vercel（main で自動、PR でプレビュー）。GitHub Actions 無料枠内で CI（backend: test & build、frontend: unit test & build、他に DB 統合・セキュリティ監査・flaky-test 系ワークフロー）。

**評価**:

- Fly.io 継続は妥当。最小 VM + 常時 1 台（コールドスタート回避）+ 自動停止の組み合わせは、個人規模のコスト最適に近い。代替（Render / Railway / Cloud Run / 自宅 VPS / オール Vercel 化）はいずれも「移行コスト ＞ 期待節約」か「無料枠の不確実性増」のどちらかになる。特に backend を Vercel Functions 等に寄せる案は、Rust ビルド・長時間リクエスト（CSV 取り込み）・WebSocket 将来性・既存 Dockerfile 資産との相性が悪く、非推奨。
- Vercel 継続は妥当。個人・小トラフィックの静的ホスティング + プレビュー環境として無料枠に収まりやすく、運用手間がほぼゼロ。Cloudflare Pages 等への乗換は、 DNS・プレビュー・環境変数運用の再構築コストに見合う節約がない。
- GitHub Actions 継続は妥当。パスフィルタ（backend/frontend 変更検出）+ concurrency キャンセルで無駄な実行を抑えており、無料枠意識がある。改善余地はキャッシュ活用と flaky 対策の継続程度。
- 無料枠のために機能を削る提案: **なし**。現行構成で無料枠近辺に収まっていると見られ、削減すべき機能は特定されなかった。仮にコスト超過が観測された場合の削減候補の考え方のみ記す: (a) Sentry（エラー監視）を外す → 障害検知が遅れる、(b) E2E の CI 実行頻度を下げる → 検出力低下、(c) Fly 常時 1 台を 0 台化 → コールドスタート悪化。いずれも本 RFC 時点では推奨しない。

**結論（観点 5）**: 現行維持。無料枠対応のための機能削減は不要。

### 4.6 観点 6: テスト戦略（cargo test / vitest / Playwright E2E）

**現行**: backend は `cargo test`（単体 `#[cfg(test)]` + `backend/tests/` 統合 + testcontainers Postgres + wiremock + tokio-test + mockall）。frontend は vitest + RTL（`__tests__/`、ユーザー視点・`getByRole` 優先）+ Playwright E2E（auth / CSV CRUD / 検索 / エラー系 / a11y / スクリーンショット）。

**評価**:

- 多層構成自体は妥当。CSV 取り込み（parse / validate / import の backend 集約）や J-Quants 連携（外部 API 契約差分が起きやすい）のような「壊れると痛い」箇所に、単体・統合・E2E の各網があるのは健全。
- 懸念はコスト（時間・メモリ・フレーキー）: frontend の `NODE_OPTIONS=8GB`、testcontainers を使う backend 統合テスト、多数の Playwright スペックは、CI 時間と不安定性の源泉になりうる。ただしこれは「戦略の誤り」ではなく「運用チューニング」の問題であり、作り直し理由にはならない。
- 改善方向（作り直さずにできる）: (a) E2E の CI 実行範囲の整理（PR 毎の全量実行 vs main のみ全量等。現行の CI 設定の精査が必要）、(b) flaky-test ワークフローの活用継続、(c) backend 統合テストの並列・分割、(d) J-Quants 系の契約テスト（`jquants-contract-maintenance` skill との連携）の維持。

**結論（観点 6）**: 戦略は妥当。やるなら実行時間・安定性のチューニングに留める。

### 4.7 観点 7: 総合判断（作り直す価値があるか、優先順位付き移行ステップ）

**総合判断: 作り直す価値はない。段階的な部分改善のみ推奨。**

理由の再掲:

1. 7 観点のいずれにも、作り直しを正当化する致命的欠陥がない（フレームワーク・DB・認証・デプロイ・テスト戦略はいずれも「妥当」）。
2. 制約の両立: 現行スタックはゼロコスト原則（静的ディスパッチ中心、型による保証、ホットパスでの `dyn` 回避）と運用コストゼロ（最小 VM + マネージド DB + 自動デプロイ）を既に満たしている。作り直しで両制約を「より良く」満たせる見込みがない。
3. コスト対効果: API 約 20 パス・CSV パイプライン・J-Quants 連携・OAuth・E2E の再構築コストは、個人開発の数ヶ月分に相当しうる一方、得られる利益（わずかな性能向上や好みの構造）は小さい。

**仮に作り直すとした場合の優先順位付き移行ステップ**（推奨はしないが、要請に応えて道筋のみ示す。すべて段階的で、一気切替はしない）:

1. **P1（やる価値あり・作り直し不要）**: services 内の重複排除（`push_filters` / ファセット / ページネーションの共通化徹底）。影響小・効果中。一気に変えずドメイン毎に寄せる。
2. **P2（検証の上で）**: `async_trait` の RPITIT 置換可否の検証（`csv_domain.rs` 等）。`mockall` との両立を確認してから。効果小。
3. **P3（計測の上で）**: リクエスト経路の clone/`Arc`/`String` 処理のプロファイルと微最適化。計測なしに実施しない。
4. **P4（運用）**: テスト実行時間・E2E 安定性のチューニング、マイグレーション手順の明文化、frontend rc 依存の棚卸し。
5. **P5（やらない）**: フレームワーク乗換、ORM 導入、DB 乗換、認証方式変更、デプロイ先移行、フロント基盤乗換。これらは本 RFC では不採用。

## 5. 採用する場合の移行ステップ（優先順位付き）

§4.7 の P1〜P4 を実行計画として再掲する。いずれも既存ブランチ運用（`1 ブランチ = 1 タスク`、PR・レビュー経由、main マージで自動デプロイ）に従い、**1 ステップずつ** 進める。P5 は実施しない。

1. services 共通化（P1）: `search_filters.rs` / `facets.rs` / `bulk_helpers.rs` の責務を確定し、各ドメイン service から重複を寄せる。受入: 既存テスト全緑 + 行数・重複の減少（目安であり厳密な数値目標は設けない）。
2. `async_trait` 検証（P2）: 置換可否を spike で確認し、可能なら最小範囲で適用。mockall との両立が崩れる場合は中止する。
3. プロファイル駆動の微最適化（P3）: ベンチ・計測の上で hot path のみ手を入れる。計測なしの「思い込み最適化」は禁止。
4. 運用チューニング（P4）: CI 時間・E2E 安定性・依存棚卸し。デプロイ構成自体は変えない。

## 6. 採用しない場合の理由 / 部分的にだけ取り入れる場合の線引き

- **フル作り直しを採用しない理由**: §4.7 の通り。致命的欠陥なし、制約は現行で充足、再構築コストが見合わない。
- **部分的に取り入れる線引き**: P1〜P4 は「作り直し」ではなく「現行コード上の改善」として取り入れてよい。線引きの基準は以下:
  - 取り入れる: API・DB・認証の互換性を保ち、1 PR でレビュー可能な範囲の変更（共通化、微最適化、CI 調整、文書化）。
  - 取り入れない: 互換性を壊す変更（フレームワーク・ORM・DB・認証・デプロイ先の乗換、一括移行）。これらを行うには、本 RFC とは別の新規 RFC と統合エージェント（Claude Code）の承認が必要。
- **機能削減の線引き**: 無料枠超過が実測されない限り削減しない。超過が観測されたら、§4.5 の候補 (a)〜(c) を「影響を明記した上で」個別に検討する。

## 7. 残る疑問点・要確認事項（統合エージェント = Claude Code への質問）

本 RFC はコード精読ではなく構成・依存・ルール文書の調査に基づくため、以下の点は未確認である。統合エージェント（Claude Code = 設計・統合担当）への確認事項として残す:

1. `services/` 各ドメインの重複度は実際どれくらいか（P1 の工数見積もりのため、コード精読による裏付けがほしい）。
2. `async_trait`（`csv_domain.rs` 等）を RPITIT に置換できるか、`mockall` との両立条件は何か。
3. Fly.io / Neon / Vercel の直近の実コスト・無料枠使用率はどうか（本 RFC は「収まっていると見られる」の推定であり、実測値の確認がほしい）。
4. E2E・統合テストの CI 実行時間と flaky 発生率はどうか（P4 の優先度付けのため）。
5. `typescript-next`（rc）併用と `NODE_OPTIONS=8GB` は一時的なものか、恒常的なものか（frontend の依存棚卸しの要否判断のため）。
6. 本 RFC の結論（フル作り直し不要・P1〜P4 のみ）に設計上の異議があるか。特に J-Quants 連携の将来拡張（V1→V2 移行等、関連 skill 参照）との整合に問題がないか。
