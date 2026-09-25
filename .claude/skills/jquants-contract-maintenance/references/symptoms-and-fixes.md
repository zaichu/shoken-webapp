# 症状と確認先

## Deserialize エラー

`backend/src/models/market_data/providers/jquants.rs` と `backend/src/services/market_data/providers/jquants.rs` で、field 名と optional / required の扱いを実応答と比較する。

## API は成功するが表示が崩れる

`frontend-leptos/src/dto.rs`、`dividend_per_share.rs`、`dividend_info.rs` を確認する。API shape を変更した場合は OpenAPI と DTO 契約テストも更新する。

## 確認順

1. 実応答と backend model / service を比較する。
2. handler と OpenAPI を揃える。
3. Leptos DTO と表示を揃える。
4. 関連テストと `scripts/check-openapi.sh` を実行する。
