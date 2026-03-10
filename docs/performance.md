# Backend パフォーマンス計測ガイド

## 概要

CSV import 系処理のボトルネックを把握するための計測手順をまとめる。
本番環境への負荷試験は対象外とし、ローカルで再現可能な計測に限定する。

---

## 計測ポイント

### 1. BulkTimer（本番ログ計測）

`backend/src/services/shared.rs` の `BulkTimer` が全 import 経路に組み込まれており、
実際のリクエストで以下のログが出力される。

```
[dividend.bulk_create] リクエスト受信: 1000件
[dividend.bulk_create] 完了: inserted=987, skipped=13, 処理時間=234.56ms
```

計測対象ドメイン: `dividend` / `domestic_stock` / `mutualfund` / `asset_balance`

ローカルで計測する手順:
1. `cd backend && make run`（環境変数は `.env` を参照）
2. フロントエンドから CSV をアップロードする、または `curl` で直接 POST する
3. バックエンドのログで `処理時間=` を確認する

### 2. CSV パース単体タイミングテスト

`csv_parse.rs` に `#[ignore]` タグ付きのタイミングテストを追加済み。
`--nocapture` で elapsed time を標準出力に出力する。

```bash
cd backend
cargo test --lib -- timing_csv_parse --ignored --nocapture
```

---

## ベースライン（ローカル計測）

環境: WSL2 / Rust release build（`cargo test` はデバッグビルド）

| 対象 | 件数/回数 | 処理時間（デバッグビルド） | 備考 |
|---|---|---|---|
| `decode_bytes` (UTF-8) | 1,000 行 × 10 回 | 0.08ms | `timing_csv_parse` テスト |
| `decode_bytes` (Shift-JIS フォールバック) | 1,000 行 × 10 回 | 0.44ms | UTF-8 の約 5.5 倍 |
| `parse_number` | 50,000 回 | 11.61ms | 0.23µs/回 |
| `parse_date` | 30,000 回 | 36.71ms | 1.22µs/回 |
| DB INSERT（bulk_create） | 100 件 | ～20ms | BulkTimer ログより（参考値） |

> 実測値は `cargo test --lib -- timing_csv_parse --ignored --nocapture` で確認すること。

---

## ボトルネック候補（優先度順）

### 高
- **DB INSERT のラウンドトリップ**: `bulk_create` は UNNEST を使ったバルクINSERTを採用済みだが、
  件数が多い場合（数千件）はトランザクション分割が必要になる可能性がある。

### 中
- **Shift-JIS デコード**: `decode_bytes()` は UTF-8 を先に試みてエラー時のみ SHIFT_JIS に
  フォールバックする。SBI CSV は実際に Shift-JIS のため、毎回フォールバックが発生している。
  先に BOM や先頭バイトでエンコーディングを判定する改善余地がある。

### 低
- **CSV 行パース**: `csv::Reader` 自体は高速。カラム名の HashMap ルックアップは
  件数が増えても O(1) のため現状問題なし。
- **JSON シリアライズ（プレビュー）**: `serde_json::to_value` を行数分呼ぶが、
  プレビューは通常 100 件未満のため影響は小さい。

---

## 次のアクション候補

1. **Shift-JIS 判定の前倒し**: `decode_bytes()` で BOM または先頭バイトを見て
   エンコーディングを決定し、不要な UTF-8 デコード試行を省く。
2. **大量データのストレステスト**: 10,000 行超の CSV で BulkTimer を観測し、
   DB 側（インデックス競合、VACUUM 頻度）の影響を確認する。
3. **release ビルドでの計測**: `cargo test --release -- timing_csv_parse --ignored --nocapture`
   でデバッグ/リリースの差を確認する。
