# shoken-artifact

React + TypeScript + Vite + Tailwind CSS + shadcn/ui で構築した Claude Artifact サンプル。
保有銘柄サマリーを表示する 3 コンポーネント構成のデモ。

## 開発

```bash
pnpm dev      # 開発サーバー起動（http://localhost:5173）
pnpm lint     # ESLint 実行
pnpm build    # Vite ビルド（dist/）
```

## 単一 HTML への bundle

```bash
bash ../scripts/bundle-artifact.sh
```

`bundle.html`（自己完結型 HTML）が生成される。Claude の Artifact として貼り付けて使う。

## 構成

| コンポーネント | 役割 |
|---|---|
| `PortfolioSummary` | 評価額・取得金額・評価損益の集計カード（4 枚） |
| `StockTable` | 銘柄一覧テーブル（評価損益バッジ付き、横スクロール対応） |
| `FilterBar` | 銘柄コード・銘柄名でのフィルター入力 |

## 技術スタック

- React 19 + TypeScript
- Vite 8
- Tailwind CSS 3.4（shadcn/ui テーマ付き）
- shadcn/ui コンポーネント（40+ 種類をプリインストール済み）
- Parcel + html-inline によるシングル HTML バンドル

> **CSS サイズについて**: Tailwind のスキャン対象に shadcn/ui コンポーネント全体（`src/components/ui/`）が含まれるため、ビルド済み CSS は約 43 kB（gzip 8 kB）になる。これは artifact 基盤として全コンポーネントを同梱するための意図的なトレードオフ。
