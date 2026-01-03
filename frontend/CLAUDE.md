# CLAUDE.md

日本語で必ず回答してください。
このファイルは Claude Code (claude.ai/code) がこのリポジトリで作業する際のガイダンスを提供します。

## コマンド

### 開発用コマンド
- `npm run dev` - ポート8080で開発サーバーを起動（自動でブラウザ起動）
- `npm run build` - `dist/`ディレクトリにプロダクションビルド
- `npm run lint` - TypeScriptとReactルールでESLintを実行
- `npm test` - メモリ最適化したVitestテスト実行
- `npm test:watch` - メモリ最適化したVitestテストをウォッチモードで実行

### Makeコマンド
- `make dev` - 開発サーバー起動
- `make build` - プロダクションビルド
- `make install` - 依存関係インストール
- `make clean` - node_modulesとdistを削除
- `make all` - クリーン、インストール、ビルドを順次実行

## アーキテクチャ概要

### 技術スタック
- **React 19.2.3** with TypeScript 5.9.3 and Vite 7.3.0
- **React Compiler 19.1.0** 自動メモ化による最適化（babel-plugin-react-compiler）
- **TanStack React Query 5.90.16** サーバー状態管理
- **React Router DOM 7.11.0** ルーティング
- **Bootstrap 5.3.8** スタイリング
- **Vitest 4.0.16 + Testing Library 16.3.1** テスト

### プロジェクト構造
**Atomic Design**手法に従った日本株取引Webアプリケーション：

- `src/components/atoms/` - 基本的なUI要素（Button, InputField, Table）
- `src/components/molecules/` - 複合コンポーネント（CSVFileInput, ErrorBoundary）
- `src/components/organisms/` - 複雑なUIブロック（Header, SearchForm, ReceiptTable）
- `src/components/templates/` - ページレイアウト（Layout, ReceiptTemplate）
- `src/pages/` - ルートレベルコンポーネント（Home, Search, Receipts, AssetBalance）
- `src/features/` - ドメインロジック（認証, jquants API, 取引履歴処理, 株式検索）
- `src/lib/` - コアユーティリティ（API クライアント, CSV処理, インターフェース, ユーティリティ）
- `src/hooks/` - カスタムReactフック
- `src/contexts/` - Reactコンテキスト

### ドメイン機能
このアプリが扱う機能：
- **CSVインポート**: エンコーディング検出付き取引データ
- **株式検索**: 銘柄コード/名前による日本株検索
- **取引履歴管理**: 取引分析（国内株式、投資信託、配当）
- **税計算**: 日本の税率内蔵（20.315%）
- **J-Quants API**: `/api/jquants` プロキシ経由の金融データ連携

### 主要アーキテクチャパターン
- **Atomic Design**: 単一責任での厳密なコンポーネント階層
- **機能ベース組織**: ビジネス機能でグループ化されたドメインロジック
- **カスタムHTTPクライアント**: リトライロジックとエラーハンドリング付きAxiosラッパー
- **型安全なCSV処理**: インポートパイプライン全体での強い型付け
- **レスポンシブテーブル**: グループ化とサマリー行付き自動リサイズ

### 設定注意事項
- **ベースパス**: `/` (Vercel ルート)
- **Vercel設定**: `vercel.json`でSPAルーティング対応（すべてのルートを`index.html`にリライト）
- **パスエイリアス**: `@/*` は `src/*` にマップ
- **API プロキシ**: 開発サーバーがJ-Quants APIをプロキシ
- **テスト**: jsdom環境、Vitest 4の forks pool使用
- **ビルド**: Terser圧縮でのベンダーチャンク分割
- **React Compiler**: Vite設定で`babel-plugin-react-compiler`を使用、ESLintで`eslint-plugin-react-compiler`を有効化

### 開発ガイドライン
- コンポーネントはAtomic Design分類に従う必要がある
- ビジネスロジックはカスタムフックまたは機能モジュールに配置
- すべてのCSV処理で型安全性を維持
- サーバー状態管理にはReact Queryを使用
- コメントとUIテキストは日本語で統一
- **React Compiler使用**: `useMemo`、`useCallback`は不要（自動最適化される）
- パフォーマンス最適化が必要な場合のみ、React Compilerのルールに従ってコードを記述

### テスト注意事項
- メモリ最適化でテスト実行（`NODE_OPTIONS='--max-old-space-size=8192'`）
- 安定性のための単一フォーク設定
- テストとフックのタイムアウト10秒
- DOMテスト用のjsdom環境使用

### API連携
- 開発時の `/api/jquants` プロキシ経由でのJ-Quants API連携
- リトライロジックとエラーハンドリング付きカスタムHTTPクライアント
- バックエンドプロキシ経由での認証管理