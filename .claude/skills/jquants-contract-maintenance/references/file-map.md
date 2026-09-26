# File Map

## Backend

- `backend/src/models/market_data/providers/jquants.rs`: 外部 API の応答モデル
- `backend/src/services/market_data/providers/jquants.rs`: J-Quants の呼び出しと変換
- `backend/src/handlers/dividend_per_share.rs`: 配当推定 API
- `backend/src/openapi.rs`: 公開 API の schema と path
- `backend/src/bin/generate_openapi.rs`: `docs/openapi.json` の生成

## Frontend

- `frontend/src/dto.rs`: DTO と OpenAPI 契約テスト
- `frontend/src/dividend_per_share.rs`: 1株配当 API の利用
- `frontend/src/dividend_info.rs`: 配当の表示
- `frontend/src/pages/asset_balance.rs`: 保有銘柄画面での利用
