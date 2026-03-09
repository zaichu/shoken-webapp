# タスク名

shoken-webapp 再設計提案

## 目的

現行実装の課題を整理し、一から再設計した場合の方針を記録する。
実装の優先度付けや、将来の大規模リファクタの判断材料とする。

---

## 現行の課題（観察）

### DB スキーマ
- **金額フィールドに `DOUBLE PRECISION`** を使用している。浮動小数点誤差が税額・損益計算に混入するリスクがある。
- `dividends` / `domestic_stocks` / `mutualfunds` が独立テーブルで、共通フィールド（`user_id`, `trade_date`, `security_code`, `taxes` 等）が分散している。
- UUID がランダム生成（v4）のため、挿入順序とディスク上の順序が一致せずページ分割が起きやすい。
- マイグレーション番号が `0001〜0015` と `20250327...` で命名規則が混在していた（現在は `0001〜0017` の4桁連番に統一済み）。

### バックエンド
- `dividend` / `domestic_stock` / `mutualfund` / `asset_balance` の 4 ドメインに CRUD ハンドラー・サービス・モデルが平行して存在し、機能追加のたびに 4 ファイルを同時に変更する必要がある。
- CSV パースロジックがドメイン別サービスに分散し、テストが書きにくい。
- セッションが DB テーブルに保存されており、スケールアウト時にセッション共有が必要になる。
- `DOUBLE PRECISION` の金額を Rust 側で `f64` として扱っており、丸め誤差が計算結果に伝播する（主要取引モデルは `rust_decimal::Decimal` 移行済み。`jquants_dividend_cache` は未移行）。
- J-Quants API の依存が services 層に直接書かれており、モック化・テストがしづらい。

### フロントエンド
- Atomic Design（atoms/molecules/organisms/templates）を 146 ファイルに適用しているが、規模に対してオーバースペックな階層で、コンポーネントの置き場に迷いが生じやすい。
- `receipt` ページが肥大化（`Receipts.tsx` 周辺）しており、タブ・フィルタ・CSV 取込が 1 ページに混在している。
- 型は OpenAPI codegen で自動生成しているが、フロントエンド側の API 呼び出しコードが手書きの axios で重複しやすい。

### インフラ・DX
- フロントエンド（Vercel）とバックエンド（Fly.io）が別々にデプロイされ、CORS の管理が必要。
- ローカル開発で DB・バックエンド・フロントエンドを個別に起動する手順が複雑（`scripts/start-local.sh` で緩和はしているが）。
- E2E テストが認証情報のローカル保存に依存しており、CI での再現性が低い。

---

## 再設計の方針

### 1. DB スキーマ

#### 金額を `NUMERIC` に統一（対応済み・一部残存）
実装では `NUMERIC(18,6)` ではなく無制限精度の `NUMERIC` を採用した（trailing zeros 問題を避けるため）。

```sql
-- Before
shares DOUBLE PRECISION NOT NULL,
taxes DOUBLE PRECISION NOT NULL,

-- After（実装済み: 0016_alter_money_columns_to_numeric.sql）
shares NUMERIC NOT NULL,
taxes  NUMERIC NOT NULL,
```
Rust 側では `rust_decimal::Decimal` で受け取り、計算に使用する。
**残作業**: `jquants_dividend_cache.dividend_per_share` は `DOUBLE PRECISION` のまま未移行。

#### UUID v7（時系列 UUID）を採用
```sql
-- Before: gen_random_uuid()（v4 ランダム）
-- After: generate_ulid() または pg_uuidv7 拡張
id UUID PRIMARY KEY DEFAULT gen_ulid()
```
挿入順序とインデックスの物理順序が一致し、インデックスのページ分割を抑制できる。

#### マイグレーション命名規則の統一（対応済み）
現在は `0001〜0017` の4桁連番形式に統一されている。新規追加は `0018_` から連番で続ける。

#### transactions テーブルへの統合（検討）
```sql
CREATE TABLE transactions (
    id         UUID    PRIMARY KEY,
    user_id    UUID    NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    type       TEXT    NOT NULL CHECK (type IN ('dividend', 'domestic_stock', 'mutualfund')),
    trade_date DATE    NOT NULL,
    -- 共通フィールド
    security_code VARCHAR(10),
    security_name VARCHAR(200),
    account       VARCHAR(100) NOT NULL,
    taxes         NUMERIC(18, 6) NOT NULL DEFAULT 0,
    -- タイプ別フィールドは JSONB か別テーブルで拡張
    extra         JSONB NOT NULL DEFAULT '{}'
);
```
**注意**: JSONB 化は型安全性を失うため、ドメイン別テーブルを維持しつつ共通インデックス・ビューで横断検索する方式も有力。

---

### 2. バックエンド

#### `CsvDomain` トレイトで 4 ドメインを統一
```rust
#[async_trait]
pub trait CsvDomain: Send + Sync {
    type Row: Send;
    type CreateRequest: Send;

    fn domain_name() -> &'static str;
    fn table_name() -> &'static str;
    fn parse_row(record: &StringRecord, row_num: usize) -> Result<Self::Row, CsvRowError>;
    async fn bulk_create(pool: &PgPool, user_id: Uuid, rows: &[Self::Row])
        -> Result<BulkCreateResponse, ApiError>;
}

// ハンドラーは型パラメータで統一
pub async fn upload_csv<D: CsvDomain>(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    multipart: Multipart,
) -> Result<impl IntoResponse, ApiError> { ... }
```
これにより新しい CSV ドメインは `CsvDomain` を実装するだけで CRUD が揃う。

#### ディレクトリを機能ドメイン単位に再編
```
backend/src/
├── domain/
│   ├── dividend/       # models + service + handler + routes
│   ├── domestic_stock/
│   ├── mutualfund/
│   ├── asset_balance/
│   └── auth/
├── infra/
│   ├── db.rs
│   ├── session.rs      # セッション管理（Redis or JWT）
│   └── jquants.rs      # J-Quants HTTP クライアント（トレイト経由でモック化可能）
├── web/
│   ├── middleware.rs
│   ├── errors.rs
│   └── state.rs
└── main.rs
```

#### セッションを Redis へ移行（またはステートレス JWT）
DB セッションは水平スケール時にボトルネックになる。
- 短期: `axum-sessions` + Redis（`fred` クレート）
- 長期: PKCE + JWT（Refresh Token ローテーション）

#### 金額計算に `rust_decimal` を使用
```rust
use rust_decimal::Decimal;

pub struct Dividend {
    pub taxes: Decimal,
    pub net_amount_received: Decimal,
}
```

#### J-Quants クライアントをトレイトで抽象化
```rust
#[async_trait]
pub trait JQuantsClient: Send + Sync {
    async fn get_dividend_per_share(&self, code: &str) -> Result<Vec<DividendPerShare>>;
}

pub struct HttpJQuantsClient { ... }
pub struct MockJQuantsClient { ... }  // テスト用
```

---

### 3. フロントエンド

#### Atomic Design → Feature-first 構造
```
frontend/src/
├── features/
│   ├── dividend/       # コンポーネント・hooks・型を同一ディレクトリに
│   ├── domestic-stock/
│   ├── mutualfund/
│   ├── asset-balance/
│   ├── stock-search/
│   └── auth/
├── shared/
│   ├── components/     # Button, Table など本当に汎用なもの
│   ├── hooks/
│   └── lib/
└── pages/              # ルーティング用エントリポイントのみ
```
「このコンポーネントは atoms か molecules か」の議論がなくなる。

#### TanStack Router（ファイルベースルーティング）を採用
```
src/routes/
├── __root.tsx
├── index.tsx           # /
├── search.tsx          # /search
├── receipts/
│   ├── index.tsx       # /receipts
│   ├── dividend.tsx    # /receipts/dividend
│   ├── domestic-stock.tsx
│   └── mutualfund.tsx
└── asset-balance.tsx
```
型安全なナビゲーション、コード分割が自動化される。

#### orval で API クライアントを完全自動生成
現行の手書き axios → openapi-typescript + orval で React Query hooks まで生成。
```typescript
// 自動生成される例
const { data } = useGetDividends({ params: { user_id } });
const { mutate: uploadCsv } = usePostDividendsCsv();
```

#### 数値を `Decimal.js` で処理
フロントエンドでも金額計算に浮動小数点を使わない。

---

### 4. インフラ・DX

#### モノリポで Vercel + Fly.io を廃止し、単一デプロイへ（検討）
- **Next.js App Router** に移行し、API Routes をバックエンドとして使う（Rust 廃止は大きなコスト）
- または **Axum + Vite SSR（Vike）** で同一サーバーから配信し CORS を不要にする
- 現実的な妥協案: Fly.io で両方ホスト（フロントを nginx で配信）

#### Docker Compose でフル環境を1コマンド起動
```yaml
services:
  db:      { image: postgres:16 }
  redis:   { image: redis:7 }
  backend: { build: ./backend }
  frontend: { build: ./frontend }
```

#### E2E テストを CI で完全再現可能に
- 認証情報をシード（テスト用 Google OAuth モック or ローカル OAuth サーバー）
- Playwright の storage state を CI 環境変数から注入

#### OpenTelemetry を最初から組み込む
```rust
// トレーシング・メトリクス・ログを統一
tracing_opentelemetry::layer()
```
Fly.io の Metrics → Grafana Cloud に流す。

---

## 優先度マトリクス

| 課題 | インパクト | コスト | 優先度 |
|---|---|---|---|
| 金額を NUMERIC / Decimal に統一 | 高（計算バグ防止） | 中 | **P0** ✓ 主要テーブル完了（#184）、jquants_dividend_cache は未移行 |
| `CsvDomain` トレイト統一 | 高（保守性） | 中 | **P1** |
| Feature-first 構造への移行 | 中（DX） | 高 | P2 |
| UUID v7 | 低（パフォーマンス） | 低 | P3 |
| セッション → Redis/JWT | 中（スケール） | 中 | P2 |
| TanStack Router 移行 | 中（型安全） | 中 | P2 |
| orval codegen | 中（DX） | 低 | P1 |
| Docker Compose 統合 | 中（DX） | 低 | P1 |
| OpenTelemetry | 中（運用） | 中 | P2 |
| モノリポデプロイ | 低（CORS排除） | 高 | P3 |

---

## メモ

- 各 P1 以降の項目は個別ブランチで実施する
