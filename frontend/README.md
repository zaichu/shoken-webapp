# Shoken Webapp (React)

**Shoken Webapp** は、株式情報の検索と取引履歴管理をサポートするReactベースのウェブアプリケーションです。

## デモ

以下のリンクからデモアプリを確認できます。

[デモサイトリンク](https://zaichu.github.io/shoken-webapp/)

## 主な機能
- CSVファイルからの取引データのインポート
- 実現損益の計算と表示
- 銘柄情報の検索と表示
- 各種証券情報サイトへのリンク生成

## プロジェクト構造

このプロジェクトでは、**Atomic Design**パターンを採用しています。UIコンポーネントは以下の5つのレベルに分類されています：

1. **Atoms**: 基本的なUIコンポーネント（Button, InputField, SelectFieldなど）
2. **Molecules**: 複数のAtomsを組み合わせたコンポーネント（CSVFileInput, StockInfoLinksなど）
3. **Organisms**: より複雑な機能ブロック（Header, Footer, SearchFormなど）
4. **Templates**: ページレイアウトの定義（ErrorPage, ReceiptTemplateなど）
5. **Pages**: 実際のページコンポーネント（Home, Search, Receiptsなど）

詳細は `/src/components/README.md` を参照してください。

## 使用技術

- **React**: フロントエンド UI ライブラリ
- **TypeScript**: 型安全なコーディング
- **React Query**: サーバー状態管理
- **React Router**: クライアントサイドルーティング
- **Bootstrap**: レスポンシブデザインを簡素化するためのCSSフレームワーク
- **Vite**: 高速な開発環境とビルドツール
- **Vitest**: ユニットテスト
- **Testing Library**: コンポーネントテスト

## 開発

```bash
# 依存関係のインストール
npm install

# 開発サーバーの起動
npm run dev

# ビルド
npm run build

# テスト
npm test
```

## デプロイ
このプロジェクトはGitHub Pagesでホストすることができます。
```bash
npm run build
```
ビルド後、`dist`ディレクトリの内容をGitHub Pagesにデプロイします。
