# Shoken Webapp (React)

**Shoken Webapp** は、日本株取引の検索と取引履歴管理をサポートするReactベースのウェブアプリケーションです。

## デモ

以下のリンクからデモアプリを確認できます。

[https://shoken-webapp.vercel.app](https://shoken-webapp.vercel.app)

## 主な機能
- **CSVインポート**: エンコーディング自動検出付き取引データのインポート
- **株式検索**: 銘柄コード/名前による日本株検索
- **取引履歴管理**: 取引分析（国内株式、投資信託、配当）
- **税計算**: 日本の税率内蔵（20.315%）
- **J-Quants API連携**: バックエンド経由の金融データ取得（`VITE_SHOKEN_WEBAPI_API_URL` で接続先を指定）
- **実現損益の計算と表示**
- **各種証券情報サイトへのリンク生成**

## プロジェクト構造

このプロジェクトでは、**Atomic Design**パターンと**機能ベース組織**を採用しています。

### UIコンポーネント（Atomic Design）
1. **Atoms**: 基本的なUIコンポーネント（Button, InputField, Tableなど）
2. **Molecules**: 複数のAtomsを組み合わせたコンポーネント（CSVFileInput, ErrorBoundaryなど）
3. **Organisms**: より複雑な機能ブロック（Header, SearchForm, ReceiptTableなど）
4. **Templates**: ページレイアウトの定義（Layout, ReceiptTemplateなど）
5. **Pages**: 実際のページコンポーネント（Home, Search, Receipts, AssetBalanceなど）

### ドメイン機能とユーティリティ
- **`src/features/`**: ドメインロジック（認証、jquants API、取引履歴処理、株式検索）
- **`src/lib/`**: コアユーティリティ（API クライアント、CSV処理、インターフェース、ユーティリティ）
- **`src/hooks/`**: カスタムReactフック
- **`src/contexts/`**: Reactコンテキスト

詳細は `/src/components/README.md` を参照してください。

## 使用技術

- **React 19.2.3**: フロントエンド UI ライブラリ
- **TypeScript 5.9.3**: 型安全なコーディング
- **React Compiler 19.1.0**: 自動メモ化による最適化（babel-plugin-react-compiler）
- **TanStack React Query 5.90.16**: サーバー状態管理
- **React Router DOM 7.11.0**: クライアントサイドルーティング
- **Tailwind CSS 4.1.18**: ユーティリティファーストのCSSフレームワーク
- **Vite 7.3.0**: 高速な開発環境とビルドツール
- **Vitest 4.0.16 + Testing Library 16.3.1**: ユニットテストとコンポーネントテスト

### 主要アーキテクチャパターン
- **Atomic Design**: 単一責任での厳密なコンポーネント階層
- **機能ベース組織**: ビジネス機能でグループ化されたドメインロジック
- **カスタムHTTPクライアント**: リトライロジックとエラーハンドリング付きAxiosラッパー
- **型安全なCSV処理**: インポートパイプライン全体での強い型付け
- **レスポンシブテーブル**: グループ化とサマリー行付き自動リサイズ

## 開発

### NPMコマンド
```bash
# 依存関係のインストール
npm install

# 開発サーバーの起動（ポート8080で自動ブラウザ起動）
npm run dev

# プロダクションビルド（dist/ディレクトリに出力）
npm run build

# ESLintによるコード検証（TypeScript + React + React Compilerルール）
npm run lint

# テスト実行（メモリ最適化済み）
npm test

# テストをウォッチモードで実行
npm test:watch
```

### Makeコマンド
```bash
# 開発サーバー起動
make dev

# プロダクションビルド
make build

# 依存関係インストール
make install

# node_modulesとdistを削除
make clean

# クリーン、インストール、ビルドを順次実行
make all
```

## 開発ガイドライン

- **コンポーネント設計**: Atomic Design分類に従ってコンポーネントを配置
- **ビジネスロジック**: カスタムフックまたは`src/features/`に配置
- **型安全性**: すべてのCSV処理とAPI通信で型安全性を維持
- **状態管理**: サーバー状態はReact Query、ローカル状態はReactコンテキストまたはフックを使用
- **コメントとUI**: 日本語で統一
- **React Compiler**: `useMemo`、`useCallback`は不要（自動最適化される）
  - パフォーマンス最適化が必要な場合のみ、React Compilerのルールに従ってコードを記述
  - ESLintで`eslint-plugin-react-compiler`が有効化されており、違反を検出

## 設定注意事項

- **ベースパス**: GitHub Pagesデプロイ用に `/shoken-webapp/` を設定
- **パスエイリアス**: `@/*` は `src/*` にマップ
- **API 接続先**: `VITE_SHOKEN_WEBAPI_API_URL` でバックエンドURLを指定（J-Quants APIはバックエンド経由）
- **テスト環境**: jsdom環境での単一フォーク設定、10秒タイムアウト
- **ビルド最適化**: Terser圧縮でのベンダーチャンク分割

## デプロイ

このプロジェクトはVercelでホストされています。

```bash
npm run build
```

ビルド後、`dist`ディレクトリの内容がVercelに自動デプロイされます。`vercel.json`の設定により、SPAのクライアントサイドルーティングが正しく動作します。

## ライセンス

このプロジェクトは個人プロジェクトです。
