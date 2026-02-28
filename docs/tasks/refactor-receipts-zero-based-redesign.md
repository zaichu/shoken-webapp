# Receipt / CSV 再設計

## 目的

取引明細の CSV 取り込みを、フロントエンド主導の画面ローカル処理から、
バックエンド主導の import パイプラインへ再設計する。
配当金・国内株式・投資信託を個別実装で増やすのではなく、同じ import モデル上で扱える構成に整理する。

## スコープ

- import を `receipts` 機能ではなく独立した `imports` ドメインとして定義する
- CSV アップロード、パース、正規化、バリデーション、重複判定、保存、結果表示を段階的な処理に分離する
- 配当金・国内株式・投資信託を共通インターフェースで扱える backend API を設計する
- frontend は「ファイル送信」「取り込み結果確認」「再実行」に責務を絞る
- receipt type ごとの分岐や duplicated API client / reducer / tab state を削減する設計へ置き換える
- OpenAPI と generated types を新しい import / receipts API に合わせて再設計する

## 非対象

- 現行 CSV 仕様を完全互換で維持すること
- 移行前後を同時に成立させるための長期的な二重実装
- 取引明細以外の資産管理画面の redesign
- UI の見た目だけを先に変えること

## 受け入れ条件

- [x] CSV 取り込みの責務が frontend から backend import pipeline へ移っている
- [x] import 結果として `inserted/skipped/errors` を receipt type 共通フォーマットで返せる
- [x] frontend から receipt type 固有の upload / mutation 分岐が消えている
- [x] receipt 一覧取得と import 実行が別ユースケースとして整理されている
- [x] OpenAPI / generated types / frontend API client が新設計に追従している

## 変更候補ファイル

- backend/src/routes.rs
- backend/src/openapi.rs
- backend/src/handlers/...
- backend/src/services/...
- backend/src/models/...
- backend/migrations/...
- docs/openapi.json
- frontend/src/features/receipt/api/...
- frontend/src/features/receipt/hooks/...
- frontend/src/features/receipt/parsers/...
- frontend/src/pages/Receipts.tsx
- frontend/src/pages/receiptsReducer.ts
- frontend/src/generated/api.ts

## タスク固有コマンド（任意）

- `cd backend && cargo run --bin generate_openapi`
- `cd frontend && npm run generate:types`
- receipt type ごとに CSV アップロードから結果表示までを手動確認する

## 進捗

- [x] 調査
- [x] 設計（type別 upload endpoint を採用）
- [x] 実装（backend CSV upload API + frontend 切り替え）
- [x] テスト（cargo test / vitest 全通過）
- [ ] PR 作成
- [ ] レビュー対応

## レビュー指摘

- 指摘が出たら追記する

## メモ

- ゼロベースなら、`/dividends/csv` のような type ごとの upload endpoint より `/imports/receipts` のような共通 endpoint を優先したい
- import の結果は同期レスポンス 1 回で閉じるより、`import job` と `import result` を持てる構成の方が拡張しやすい
- frontend の local CSV parse は preview 専用に限定するか、完全に削除するかを早めに決める
- 現在の `receiptApi.ts` と `useReceiptsData.ts` は read / import / delete が密結合なので分割対象
- 現在の `Receipts.tsx` は tab state, file state, query state, mutation state, view rendering を同居させているため、container と type 別 view に分解したい

## 実装タスク案

- [x] 1. 現行フローを分解し、frontend / backend / DB の責務一覧を作る
- [x] 2. receipt type 共通の import result schema を定義する（CsvUploadResponse/CsvRowError）
- [x] 3. broker / receipt type ごとの CSV parser を backend に実装する
- [x] 4. 重複判定と validation を import pipeline に寄せる（bulk_create 再利用）
- [x] 5. `/dividends/csv`, `/domestic-stocks/csv`, `/mutualfunds/csv` エンドポイント追加
- [x] 6. OpenAPI と generated types を再生成する
- [x] 7. frontend の API client に uploadCsv を追加し bulkCreate 分岐を削除する
- [x] 8. `useReceiptsData` の mutation を uploadCsv に切り替える
- [x] 9. reducer に rawFiles 状態を追加する
- [x] 10. `Receipts.tsx` を rawFile ベースの保存フローに更新する
- [x] 11. import 結果 UI を inserted/skipped/errors 詳細で表示する
- [x] 12. 旧 CSV パーサー（parseDividendCsvItem 等）の撤去
