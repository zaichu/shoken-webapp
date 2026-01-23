# フロントエンド開発ルール

## 技術スタック

- **React**: 19.x (React Compiler 有効)
- **TypeScript**: 5.x (strict mode)
- **Vite**: 7.x
- **Bootstrap**: 5.x
- **TanStack Query**: サーバー状態管理
- **Axios**: HTTPクライアント
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

- Bootstrap 5 をベースに使用
- グローバルスタイルは `src/styles/` に配置
- コンポーネント固有スタイルは CSS Modules を検討

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

## API通信

### Axios ラッパー

`lib/api/` に共通クライアントを配置:

- エラーハンドリング統一
- 認証ヘッダー自動付与
- リトライロジック

### TanStack Query

- サーバー状態管理に使用
- キャッシュ・再フェッチを自動管理

## CSV処理

- PapaParse を使用
- エンコーディング自動検出対応
- `lib/csv/` にユーティリティ配置

## J-Quants API

- 日本株データの取得に使用
- 開発時は Vite プロキシ経由
- `features/jquants/` に関連コードを配置
