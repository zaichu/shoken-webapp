# CI/CD 高速化の検討

- 作成日: 2026-09-05
- 対象ブランチ: `docs/ci-cd-speedup-proposal`
- 制約: ワークフローファイルを含め一切変更しない（本書は提案のみ）。運用コストゼロ（GitHub Actions 無料枠内）。
- 計測方法: `gh run list` / `gh run view --json jobs` / `gh run view --log` の実データに基づく。推測による効果主張はしない。

## 1. 概要

### 現状の実行時間サマリー（実測）

| ワークフロー | 代表的な実行時間 (wall) | 主な内訳 |
|---|---|---|
| Backend CI / Deploy（backend 変更あり・push to main） | 約 12 分 23 秒 | Test & Build 約 8 分 49 秒 + Validate 約 21 秒 + Deploy 約 3 分 2 秒 |
| Backend CI / Deploy（backend 変更あり・PR） | 約 9 分 40 秒 | Test & Build 約 9 分 10 秒 + Validate 約 13 秒（Deploy なし） |
| Backend CI / Deploy（backend 変更なし・PR） | 約 9〜12 秒 | 変更検出後に後続ジョブは skip（検出ロジックは正常に機能） |
| Frontend CI（frontend 変更あり・PR） | 約 2 分 34 秒 | Unit Test & Build 約 2 分 23 秒と E2E 約 1 分 16 秒が並列 |
| Frontend CI（frontend 変更なし・PR） | 約 8 秒 | 変更検出後に後続ジョブは skip（検出ロジックは正常に機能） |
| Backend DB Integration（定期・日次） | 約 1 分 51 秒 | キャッシュ復元約 63 秒 + テスト約 34 秒 |
| Security Audit（PR・依存変更なし） | 約 17〜24 秒 | 変更検出のみで監査ジョブは skip |

### 期待できる短縮効果の見積もり（概要）

| 優先順位 | 改善案 | 期待効果（推定・要検証） |
|---|---|---|
| P0 | Docker build に Buildx + GHA キャッシュ (`type=gha`) を導入 | backend フル実行で約 2〜3 分短縮（現行 docker build 約 4〜5 分が対象） |
| P1 | Rust キャッシュの見直し（`Swatinem/rust-cache` 移行またはキー設計整理） | backend フル実行で約 1〜2 分短縮（復元 36〜63 秒 + 保存 35〜107 秒が対象） |
| P2 | `validate-frontend-types` の並列化（`needs` 解除） | wall 約 20 秒短縮（効果は小さいがコスト・リスクも小さい） |
| P3 | Playwright ブラウザインストールの条件付きスキップ等こまかい改善の束 | 合計で約 10〜30 秒短縮 |

効果の数値は「現行の実測値を上限とする差分推定」であり、導入後に同一条件で再計測して確定させること（詳細は §4 の各案の「効果測定方法」）。

### 無料枠の状況（概算）

- 直近 100 実行（2026-08-12〜2026-09-05・約 24 日間）の wall 合計は約 209 分。
- ジョブ並列分を加味した課金分はその約 1.2〜1.5 倍と見積もると、月換算で約 300〜400 分程度。無料枠（Linux ランナー 2,000 分/月）に対して十分余裕がある。
- よって本検討の主目的は「コスト削減」ではなく「開発者待機時間（特に backend フル実行の約 10〜12 分）の短縮」と位置づける。並列化など課金分を増やす案はこの観点で慎重に扱う（§5）。

## 2. 実測データ

計測日時: 2026-09-05。`gh run list --limit 20/60/100` と `gh run view <id> --json jobs`、`--log` を使用。

### 2.1 Backend CI / Deploy — フル実行（push to main、run `33930195659`）

wall: 2026-09-04T23:37:19Z → 23:49:42Z（約 12 分 23 秒）。

**Test & Build（約 8 分 49 秒）**

| ステップ | 所要時間 |
|---|---|
| Setup Rust toolchain | 約 10 秒 |
| Cache Cargo registry & build（復元） | 約 36 秒 |
| Lint (clippy) | 約 32 秒 |
| Run tests (`cargo test`) | 約 1 分 48 秒 |
| Generate OpenAPI schema | 約 16 秒 |
| Check OpenAPI schema is up to date | 約 0 秒 |
| Build Docker image（`docker build`、キャッシュなし） | 約 4 分 46 秒 |
| Post Cache Cargo registry & build（保存） | 約 35 秒 |

**Validate Frontend Types（約 21 秒）**: Test & Build の完了を約 9 分待ってから実行。実作業は `setup-frontend`（npm ci、約 7 秒）+ 型生成・型チェックのみ。

**Deploy to Fly.io（約 3 分 2 秒）**: すべて `flyctl deploy --remote-only` のステップ内。外部ビルダー側の処理でありリポジトリ側では短縮不能。

### 2.2 Backend CI / Deploy — フル実行（cargo 依存更新 PR、run `33399755997`）

wall: 2026-08-31T13:56:37Z → 14:06:17Z（約 9 分 40 秒）。

| ステップ | 所要時間 |
|---|---|
| Cache 復元 | 約 63 秒 |
| Lint (clippy) | 約 24 秒 |
| Run tests | 約 1 分 39 秒 |
| Generate OpenAPI schema | 約 12 秒 |
| Build Docker image | 約 3 分 57 秒 |
| Post Cache 保存 | 約 1 分 47 秒 |

2 件のフル実行で docker build が約 4〜5 分、clippy + test 合計が約 2 分で安定している。

### 2.3 Frontend CI — フル実行（frontend 依存更新 PR、run `33931979412`）

wall: 2026-09-05T00:08:31Z → 00:11:05Z（約 2 分 34 秒）。Unit Test & Build と E2E は並列実行。

**Unit Test & Build（約 2 分 23 秒）**

| ステップ | 所要時間 |
|---|---|
| Setup frontend（`setup-node` + `npm ci`） | 約 7 秒 |
| Lint | 約 10 秒 |
| TypeScript type check | 約 1 秒 |
| Run tests（`npm test`） | 約 1 分 55 秒 |
| Build | 約 6 秒 |

**E2E（約 1 分 16 秒）**

| ステップ | 所要時間 |
|---|---|
| Setup frontend | 約 7 秒 |
| Cache Playwright browsers（復元） | 約 0 秒（ただし後述の通り実は miss） |
| Cache APT packages | 約 15 秒 |
| Install Playwright browsers | 約 9 秒 |
| Run E2E tests | 約 31 秒 |

### 2.4 変更検出（skip 実行）の実測

- Backend CI（backend 変更なし PR、run `33931979428`）: Test & Build 約 6 秒で完了、後続 2 ジョブは skipped。全体約 9 秒。
- Frontend CI（frontend 変更なし PR、run `33942115730`）: changes ジョブ約 4 秒、後続 2 ジョブは skipped。全体約 8 秒。
- 変更検出ロジック（`dorny/paths-filter`）は正しく機能しており、無駄なフル実行は発生していない。

### 2.5 Backend DB Integration（定期実行、run `33918064561`）

約 1 分 51 秒。内訳: キャッシュ復元約 63 秒 + `make test-ignored` 約 34 秒。復元がテスト本体の約 2 倍かかっている。

### 2.6 キャッシュのヒット実態（ログからの裏取り）

`gh run view --log` で `Cache restored from key` / `Cache saved with key` / `Cache Size` を確認した結果:

- **Rust キャッシュは exact ヒットしていないケースが多い。** 例: run `33399755997`（Cargo.lock を更新する dependabot PR）では exact キー（`Linux-cargo-<Cargo.lock ハッシュ>`）が miss し、restore-keys 経由で別ワークフロー（Backend DB Integration）のキー `Linux-cargo-db-integration-<ハッシュ>` から約 2.4 GB を復元していた（63 秒）。幸いフォールバックが効いたため clippy・test は増分扱いで高速だった。
- run `33930195659`（push to main）でも同様に `Linux-cargo-db-integration-...` からの復元だった。直前（8/31）に `Linux-cargo-1e2dc...` として保存された exact キーが使われなかった理由は、その間に backend 依存更新（#746）がマージされ Cargo.lock が変わったためとみられる。すなわち **Cargo.lock が変わるたびに exact miss → 他ワークフローの古いキャッシュへのフォールバック** という動作になっている。
- **キャッシュサイズが大きい。** Rust キャッシュは約 2.4 GB（`Cache Size: ~2422 MB`）。`deploy-backend` 用と `backend-db-integration` 用で別キーに約 2.4 GB 級が 2 本存在しうる。GHA キャッシュの上限（リポジトリあたり 10 GB）に対する圧迫要因であり、退避（eviction）で exact キーが消える可能性もある。
- **npm キャッシュは exact ヒットしている。** `node-cache-Linux-x64-npm-<ハッシュ>` で復元確認。`npm ci` は約 7 秒で終わっており問題ない。
- **Playwright ブラウザキャッシュは miss していた。** run `33931979412` の E2E で `Cache not found for input keys: Linux-playwright-<ハッシュ>`。キーに `package-lock.json` ハッシュを含むため、依存更新 PR では必ず miss する設計になっている。miss してもブラウザインストールは約 9 秒で終わり、実行末に新規保存されていた（次回以降の同一 lockfile では hit する）。
- **APT パッケージキャッシュも同様に miss → 保存されていた。** キーに lockfile ハッシュを含むため依存更新時は miss するが、所要約 15 秒であり影響は小さい。

## 3. ボトルネックの特定

実測の寄与度が大きい順に記載する。

### B1. Docker build にレイヤーキャッシュがない（約 4〜5 分／backend フル実行）

- `deploy-backend.yml` の `Build Docker image` は素の `docker build -t shoken-backend:ci .` で、Buildx の GHA キャッシュもレジストリキャッシュも使っていない。
- `backend/Dockerfile` は依存コンパイル層の分離（ダミー `main.rs` で先に `cargo build --release`）を実装済みだが、CI で層が再利用されないため毎回ベースイメージ取得 + 全依存再コンパイルが発生し、2 回の実測で約 4 分 46 秒・約 3 分 57 秒を記録した。
- 単独で最大のボトルネック。

### B2. Rust キャッシュの復元・保存が重い（合計約 1.5〜2.5 分／backend フル実行）

- 復元 36〜63 秒 + 保存 35〜107 秒。2.4 GB の `backend/target` を含む一括キャッシュの転送が支配的。
- `actions/cache` の素朴なパス指定（`~/.cargo/registry` + `~/.cargo/git` + `backend/target`）で増分コンパイル成果物ごと運んでいる。
- 2 ワークフロー（deploy-backend / backend-db-integration）が別キー・同一 restore prefix（`${{ runner.os }}-cargo-`）で運用され、フォールバックが偶然の共有に依存している。キー設計が意図的に整理されていない。

### B3. `validate-frontend-types` が直列待ち（約 9 分待って実作業約 20 秒）

- `needs: test-and-build` で直列化されているが、同ジョブはリポジトリを fresh checkout して `npm run generate:types` + `typecheck` するだけで、backend ジョブの成果物（ファイル共有なし）を一切使っていない。`needs` は実質「backend 変更時のみ実行する」ためのゲートとして使われている。
- 並列化しても wall 短縮は約 20 秒にとどまる（backend ジョブ自体がクリティカルパス）。効果は小さい。

### B4. Frontend の `npm test`（約 1 分 55 秒／frontend フル実行）

- Unit Test & Build ジョブ内で最大のステップ。テストコード自体の変更は非対象のため、ワークフロー側でできるのは分割・並列化程度に限られる（§5 で不採用理由を述べる）。

### B5. 細かいもの（各約 10 秒前後）

- E2E の `npx playwright install chromium`（約 9 秒）がキャッシュ有無にかかわらず無条件実行。
- Playwright / APT キャッシュのキーが lockfile ハッシュ依存で、依存更新時に必ず miss（ただし影響は約 10〜15 秒）。
- Backend CI の `pull_request` トリガーに paths フィルタがなく、docs のみの PR でも検出ジョブ（約 10 秒）が起動する。
- `security-audit.yml` に `concurrency` 設定がなく、定期実行と手動実行の重複時に打ち消しが効かない。

## 4. 改善案（優先順位付き）

いずれもワークフローファイルの変更を伴うため、本書では提案のみとする。期待効果は「現行実測値を根拠とする上限目安」であり、推定値の確定には各案の効果測定方法による再計測が必要。

### P0. Docker build に Buildx + GHA キャッシュを導入する

- 内容: `docker/build-push-action` 等への置き換え、`cache-from: type=gha` / `cache-to: type=gha,mode=max` を設定する。Dockerfile 自体の変更は不要（依存層分離が既にあるため、そのまま層キャッシュが効く）。
- 根拠（実測）: docker build ステップは約 4 分 46 秒・約 3 分 57 秒で、毎回全依存を再コンパイルしている。Cargo.lock 不変時は依存層が再利用され、残るのはアプリクレートのコンパイルとベースイメージ取得のみになる。
- 期待効果（推定）: backend フル実行で約 2〜3 分短縮。正確な値は導入後に同一条件（同一 Cargo.lock）で 2〜3 回計測して確定させる。
- 実装コスト: 小（1 ステップの置き換え + キャッシュスコープ設定）。
- リスク・トレードオフ:
  - GHA キャッシュ使用量が増加（イメージ層で 1〜2 GB 規模の可能性）し、10 GB 上限の圧迫要因になる（B2 と合算で要管理）。`scope` 分離と定期的な eviction 状況の確認が必要。
  - キャッシュ由来の古い層混入リスクは `mode=max` + Cargo.lock 依存のキー設計上は低いが、Dockerfile 変更時は初回のみ遅くなる。
  - 信頼性への影響なし（同一 Dockerfile のビルドであり、テストの skip ではない）。
- 効果測定方法: 導入前後で同一 Cargo.lock の push 実行における `Build Docker image` ステップ時間を比較する。

### P1. Rust キャッシュを見直す（`Swatinem/rust-cache` 移行またはキー設計の整理）

- 内容（案 a・推奨）: `actions/cache` の手書きパス指定をやめ、`Swatinem/rust-cache`（または同等の Rust 特化キャッシュ）に置き換える。増分コンパイルに必要な最小集合だけを扱うため、転送量が大幅に減る設計である。
- 内容（案 b・軽量）: 現行 `actions/cache` を維持し、2 ワークフローでキー・restore-keys を意図的に整理する（例: 共通 prefix の明示、保存キーの統一、`backend/target` の不要物除外）。
- 根拠（実測）: 復元 36〜63 秒 + 保存 35〜107 秒、サイズ約 2.4 GB。exact miss が常態化し他ワークフローのキーへのフォールバックで動いている（§2.6）。
- 期待効果（推定）: 復元 + 保存の合計約 1.5〜2.5 分が数十秒規模に縮小し、backend フル実行・DB Integration ともに約 1〜2 分短縮。導入後に `Cache Size` ログと復元・保存ステップ時間で確定させる。
- 実装コスト: 小〜中（Action 差し替え + 数回の実行でヒット率を確認）。
- リスク・トレードオフ:
  - サードパーティ Action への依存（現行も `dorny/paths-filter` 等を使っており新規リスクは限定的。SHA pin の方針維持が前提）。
  - キャッシュ破損・不整合時は初回フルビルドに戻るだけであり、テスト内容自体は変わらないため信頼性への影響なし。
  - P0 と併用すると GHA キャッシュ総量が増える方向と減る方向の両作用があるため、合計使用量の監視が必要（Claude Code への確認事項）。
- 効果測定方法: 導入前後で `Cache Cargo registry & build`（復元・保存）のステップ時間と `Cache Size` ログを比較する。

### P2. `validate-frontend-types` を `test-and-build` と並列化する

- 内容: `needs: test-and-build` を外し、backend 変更検出を `dorny/paths-filter` の独立ステップ（frontend CI の `changes` ジョブと同型）で行う。`deploy` ジョブの `needs` は維持する。
- 根拠（実測）: 実作業約 13〜21 秒に対し約 9 分の直列待ちが発生している。ジョブ間に成果物の受け渡しはなく（fresh checkout）、順序依存がないことを確認済み。
- 期待効果（実測ベース）: wall 約 20 秒短縮。効果は小さいが実装コスト・リスクも最小。
- リスク・トレードオフ: 並列化により backend 側が失敗した場合でも frontend 型検証分のランナー分（約 20 秒）が消費される。無視できる範囲。
- 効果測定方法: 変更前後で backend 変更 PR の workflow wall を比較する。

### P3. こまかい改善の束（いずれも効果は小さいがコストも小さい）

1. **Playwright ブラウザインストールの条件付きスキップ**: `actions/cache` ステップに `id` を付与し、`if: steps.<id>.outputs.cache-hit != 'true'` で `npx playwright install chromium` をスキップする。根拠: 約 9 秒の固定コストが毎回発生していること（§2.3）。注意: Playwright のバージョン更新時はキャッシュキーが lockfile ハッシュ依存のため自動的に miss → インストールが走る設計であり、陳腐化リスクは低い。
2. **Backend CI の PR トリガーに paths フィルタ検討**: docs のみの PR で検出ジョブ（約 10 秒 + ランナー起動）が走っている。ただし paths フィルタの抜けは「CI 未実行のままマージ」のリスクを生むため、対象は `docs/**` `*.md` 等の無害なパスに限定し、信頼性との両論併記で判断する（§5 関連）。
3. **`security-audit.yml` に `concurrency` を追加**: 定期実行と手動実行の重複時に旧実行を打ち消す。効果は待機時間の削減ではなくリソース浪費の防止。監査自体の信頼性への影響なし。

## 5. 採用しない案とその理由

- **テストの skip・削減（cargo test / npm test / E2E の省略）**: 信頼性を直接落とすため不採用。高速化と信頼性のトレードオフ以前に、本タスクの制約で禁止されている。
- **Docker build ステップの削除**: 約 4〜5 分の削減になるが、本番デプロイ物（Fly.io は別途 remote build するものの、同一 Dockerfile の検証）の事前検証を失う。デプロイ失敗の検出が main push 時まで遅延し、信頼性が落ちるため不採用。
- **`cargo test` / clippy の別ジョブ並列化**: wall は clippy 分（約 30 秒）縮む可能性があるが、各ジョブが約 2.4 GB のキャッシュ復元（約 40〜60 秒）を個別に行い、課金分は純増する。無料枠の観点で逆効果のため不採用。P1 でキャッシュが軽量化できた後に再検討する余地はある。
- **セルフホストランナー・有料ビルド高速化サービス**: 運用コストゼロ制約に反するため不採用。
- **Fly.io デプロイ（約 3 分）の短縮**: `--remote-only` の外部ビルダー処理が支配的で、リポジトリ側のワークフロー変更では短縮不能。デプロイ戦略自体の見直しは設計判断が必要なため、本書では提案しない。
- **Frontend `npm test` の shard 分割**: テストコード変更なしに `vitest --shard` 等で並列化は可能だが、ジョブ分割で `npm ci` + ランナー起動が重複し課金分が増加する。wall 約 1 分に対する setup 重複約 15〜20 秒×分割数を考えると費用対効果が薄く、まず P0・P1 を優先すべきとして見送る。

## 6. Claude Code（統合エージェント）への確認事項

1. **GHA キャッシュ 10 GB 上限の管理方針**: Rust 約 2.4 GB×2 系統に加え、P0（Docker 層キャッシュ 1〜2 GB 規模の見込み）を追加すると上限に近づく可能性がある。キャッシュ総量の定期的確認の方法（設定画面の目視か、API での監視か）と、上限接近時の優先度（何を evict させるか）を決めたい。
2. **サードパーティ Action 追加の可否**: P1 で `Swatinem/rust-cache`、P0 で `docker/build-push-action` を使う場合、既存の SHA pin 方針（現行も `dorny/paths-filter` 等で採用）を踏襲することでよいか。
3. **`validate-frontend-types` 並列化（P2）の可否**: `deploy` ジョブの `needs: [test-and-build, validate-frontend-types]` は維持しつつ、検証 2 ジョブ間だけ並列化する設計でよいか。
4. **効果測定の受け入れ基準**: 各案の導入後に「同一条件で N 回計測しステップ時間を比較する」ことを受け入れ条件に含めてよいか（N=2〜3 を想定）。
5. **無料枠の監視**: 月間使用量（目安: 現行約 300〜400 分/月）が急増した際の検知方法を用意するか（例: 定期的な `gh` での棚卸しを運用に組み込むか）。
6. **Backend CI の PR トリガーへの paths フィルタ追加の可否**: 微効果の一方で「CI 未実行のままマージ」のリスクをどう評価するか。対象パスを `docs/**` 等に限定する案の採否。
