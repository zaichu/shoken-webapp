# REST API v1 Design

## Goals

- Keep the backend in Rust and Axum.
- Introduce RESTful, versioned routes under `/api/v1`.
- Keep old routes working during migration.
- Make destructive and import operations explicit resources.
- Hide external provider names such as J-Quants from first-class API resources
  unless provider-specific behavior is intentionally exposed.

## Non-Goals

- Replacing Axum.
- Changing the database schema as part of the first API redesign step.
- Rewriting the frontend in the same PR as backend route additions.
- Changing the session cookie storage model.

## Naming Rules

- Use nouns for resources.
- Use query parameters for search.
- Use `DELETE /collection` for collection deletion, not `/all`.
- Use `PUT /collection` when the operation replaces the user's full collection.
- Model CSV preview as an import validation resource.
- Model CSV save as an import resource.

## Proposed Routes

### Probes

| Method | Path | Description |
|---|---|---|
| GET | `/health` | Liveness check |
| GET | `/ready` | Startup readiness check |

### Session And Account

| Method | Path | Description |
|---|---|---|
| GET | `/api/v1/session` | Get current user session |
| DELETE | `/api/v1/session` | Logout current session |
| POST | `/api/v1/account-deletion-confirmations` | Start account deletion confirmation |
| DELETE | `/api/v1/account` | Delete current account after confirmation |
| GET | `/api/v1/oauth/google/authorize` | Start Google OAuth |
| GET | `/api/v1/oauth/google/callback` | Google OAuth callback |

### Stocks

| Method | Path | Description |
|---|---|---|
| GET | `/api/v1/stocks?query=7203` | Search stocks by code or name |
| POST | `/api/v1/stocks` | Create stock |

### Dividends

| Method | Path | Description |
|---|---|---|
| GET | `/api/v1/dividends` | List dividends |
| DELETE | `/api/v1/dividends` | Delete all dividends for current user |
| POST | `/api/v1/dividend-import-validations` | Validate dividend CSV without DB writes |
| POST | `/api/v1/dividend-imports` | Import dividend CSV |
| POST | `/api/v1/dividend-per-share-estimates` | Batch dividend-per-share lookup |

### Domestic Stock Transactions

| Method | Path | Description |
|---|---|---|
| GET | `/api/v1/domestic-stock-transactions` | List domestic stock transactions |
| DELETE | `/api/v1/domestic-stock-transactions` | Delete all domestic stock transactions |
| POST | `/api/v1/domestic-stock-import-validations` | Validate domestic stock CSV |
| POST | `/api/v1/domestic-stock-imports` | Import domestic stock CSV |

### Mutual Fund Transactions

| Method | Path | Description |
|---|---|---|
| GET | `/api/v1/mutual-fund-transactions` | List mutual fund transactions |
| DELETE | `/api/v1/mutual-fund-transactions` | Delete all mutual fund transactions |
| POST | `/api/v1/mutual-fund-import-validations` | Validate mutual fund CSV |
| POST | `/api/v1/mutual-fund-imports` | Import mutual fund CSV |

### Asset Balances

| Method | Path | Description |
|---|---|---|
| GET | `/api/v1/asset-balances` | List asset balances |
| PUT | `/api/v1/asset-balances` | Replace all asset balances for current user |
| DELETE | `/api/v1/asset-balances` | Delete all asset balances |
| POST | `/api/v1/asset-balance-import-validations` | Validate asset balance CSV |
| POST | `/api/v1/asset-balance-imports` | Import asset balance CSV |

### Market Data

| Method | Path | Description |
|---|---|---|
| GET | `/api/v1/financial-statements?code=7203` | Get financial statement summary |

## Compatibility Map

| Current Route | v1 Route |
|---|---|
| `GET /auth/me` | `GET /api/v1/session` |
| `POST /auth/logout` | `DELETE /api/v1/session` |
| `DELETE /auth/delete-account` | `DELETE /api/v1/account` |
| `GET /auth/google` | `GET /api/v1/oauth/google/authorize` |
| `GET /auth/google/callback` | `GET /api/v1/oauth/google/callback` |
| `GET /stocks/{query}` | `GET /api/v1/stocks?query={query}` |
| `POST /stocks` | `POST /api/v1/stocks` |
| `GET /dividends` | `GET /api/v1/dividends` |
| `DELETE /dividends` | `DELETE /api/v1/dividends` |
| `POST /dividends/csv/preview` | `POST /api/v1/dividend-import-validations` |
| `POST /dividends/csv` | `POST /api/v1/dividend-imports` |
| `GET /domestic-stocks` | `GET /api/v1/domestic-stock-transactions` |
| `DELETE /domestic-stocks` | `DELETE /api/v1/domestic-stock-transactions` |
| `POST /domestic-stocks/csv/preview` | `POST /api/v1/domestic-stock-import-validations` |
| `POST /domestic-stocks/csv` | `POST /api/v1/domestic-stock-imports` |
| `GET /mutualfunds` | `GET /api/v1/mutual-fund-transactions` |
| `DELETE /mutualfunds` | `DELETE /api/v1/mutual-fund-transactions` |
| `POST /mutualfunds/csv/preview` | `POST /api/v1/mutual-fund-import-validations` |
| `POST /mutualfunds/csv` | `POST /api/v1/mutual-fund-imports` |
| `GET /asset-balances` | `GET /api/v1/asset-balances` |
| `POST /asset-balances/bulk` | `PUT /api/v1/asset-balances` |
| `DELETE /asset-balances` | `DELETE /api/v1/asset-balances` |
| `POST /asset-balances/csv/preview` | `POST /api/v1/asset-balance-import-validations` |
| `POST /asset-balances/csv` | `POST /api/v1/asset-balance-imports` |
| `GET /jquants/fins/summary` | `GET /api/v1/financial-statements` |
| `POST /dividends/per-share/batch` | `POST /api/v1/dividend-per-share-estimates` |

## Migration Order

1. Harden current security behavior.
2. Add `/api/v1/session` and OAuth aliases.
3. Add `/api/v1/stocks`.
4. Add read/delete aliases for existing collections.
5. Add import validation/import aliases.
6. Move frontend API client to `/api/v1`.
7. Mark old routes as deprecated.
8. Remove old routes only after a separate deprecation decision.

