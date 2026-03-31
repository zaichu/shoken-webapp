# フロントエンド開発ルール

## 技術スタック

- **React**: 19.x (React Compiler 有効)
- **TypeScript**: 5.x (strict mode)
- **Vite**: 7.x
- **Tailwind CSS**: 4.x
- **TanStack Query**: サーバー状態管理
- **Vitest**: テストフレームワーク

## Atomic Design

コンポーネントは Atomic Design パターンに従う:

```
src/components/
├── atoms/       # Button, InputField など基本要素
├── molecules/   # CSVFileInput など複合コンポーネント
├── organisms/   # Header, SearchForm など複雑なブロック
└── templates/   # ページレイアウト
```

### 配置ルール

- 単一の機能に特化した最小単位 → `atoms/`
- atoms を組み合わせた再利用可能なUI → `molecules/`
- 複数の molecules/atoms からなるセクション → `organisms/`
- ページ全体のレイアウト構造 → `templates/`

## ディレクトリ構成

```
src/
├── components/    # Atomic Design コンポーネント
├── features/      # ドメイン別機能
│   ├── auth/      # 認証
│   ├── jquants/   # J-Quants API連携
│   ├── receipt/   # 取引履歴
│   └── stock/     # 株式検索
├── pages/         # ルートページコンポーネント
├── hooks/         # カスタムフック
├── lib/           # ユーティリティ
│   ├── api/       # APIクライアント
│   ├── csv/       # CSV処理
│   ├── interfaces/
│   ├── types/
│   └── utils/
├── contexts/      # React Context
├── routes/        # ルーティング設定
└── styles/        # グローバルスタイル
```

## TypeScript

### パスエイリアス

`@/*` で `src/*` を参照可能:

```typescript
import { Button } from '@/components/atoms/Button';
```

### 型定義

- インターフェースは `lib/interfaces/` に配置
- 型は `lib/types/` に配置
- コンポーネントの Props は同一ファイル内で定義

## React Compiler

React 19 + React Compiler が有効:

- `useMemo`, `useCallback` は基本的に不要（自動最適化）
- ESLint の `react-compiler` ルールがエラーを検出

## スタイリング

### 基本方針

- **Tailwind CSS** をベースに使用
- グローバルスタイルは `src/styles/tailwind.css` に配置

### クラス結合

`cn()` ユーティリティを使用して統一:

```typescript
import { cn } from '@/lib/utils/classNames';

// 基本パターン
const classes = cn(baseClasses, conditionalClass && 'active', className);
```

### 配置ルール

| 種類 | 配置場所 | 例 |
|------|----------|-----|
| テーマ変数 | `tailwind.css` の `@theme` | `--color-primary` |
| 再利用クラス | `tailwind.css` の `@layer components` | `.panel-card`, `.stat-grid` |
| コンポーネント固有 | 各 `.tsx` ファイル内 | `variantStyles` オブジェクト |
| 動的スタイル | `style` 属性 | 計算された高さ・幅のみ |

### 共通クラス

| クラス名 | 用途 |
|---------|------|
| `.form-input-container` | フォーム入力コンテナ（`w-full max-w-[400px]`） |
| `.stat-grid` | 統計情報グリッド（3カラム） |
| `.action-toolbar` | アクションボタン群 |
| `.status-message` | ローディング・空状態メッセージ |
| `.panel-card` | パネルカード |

### className の順序

外部からの `className` は最後に配置（オーバーライド可能にするため）:

```typescript
const classes = cn(
  baseClasses,      // 1. ベーススタイル
  variantClass,     // 2. バリアント
  sizeClass,        // 3. サイズ
  className         // 4. 外部からの上書き（最後）
);
```

### インラインスタイルの使用

動的計算値のみ許可:

```typescript
// OK: 動的に計算される高さ
style={{ maxHeight: calculatedHeight }}

// NG: 静的な値は Tailwind クラスを使う
style={{ padding: '16px' }} // → className="p-4"
```

## テスト

### 設定

- Vitest + React Testing Library
- テストファイルは `__tests__/` ディレクトリに配置
- メモリ制限: `NODE_OPTIONS='--max-old-space-size=8192'`

### 実行

```bash
npm test           # テスト実行
npm run test:watch # ウォッチモード
```

### 補助コマンド

```bash
vp dev   # 開発サーバー起動
vp lint  # Lint 実行
vp build # ビルド
```

### 方針

- テストファイルは各機能配下の `__tests__/` に配置する
- ユーザー視点で書き、`getByRole` / `getByText` を優先する
- `data-testid` は最終手段にする
- 非同期処理は `waitFor` または `findBy*` を使用する

## API通信

### API クライアント

`lib/api/` に共通クライアント（native fetch ベース）を配置:

- エラーハンドリング統一
- 認証ヘッダー自動付与
- リトライロジック

### TanStack Query

- サーバー状態管理に使用
- キャッシュ・再フェッチを自動管理

## J-Quants API

- 日本株データの取得に使用
- 開発時は Vite プロキシ経由
- `features/jquants/` に関連コードを配置

## セキュリティ

### 環境変数

- フロントエンド公開変数は `VITE_` プレフィックス必須
- 機密情報はフロントエンドに置かず、バックエンド経由で取得する

### 入力・XSS

- 入力値はフロントエンドでも検証するが、必ずサーバー側でも検証する
- React の自動エスケープを前提とし、`dangerouslySetInnerHTML` は必要時のみサニタイズ後に使う

### 依存関係

- 必要に応じて `npm audit` で脆弱性を確認する
