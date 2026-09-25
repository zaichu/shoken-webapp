---
name: jquants-contract-maintenance
description: J-Quants API の応答差分、deserialize エラー、OpenAPI と Leptos DTO の不整合、配当表示の不具合を調査・修正するための手順。
allowed-tools: Read, Grep, Glob, Bash
---

# J-Quants 契約の保守

## 確認順

1. J-Quants の実際の応答と `backend/src/models/market_data/providers/jquants.rs` を比較する。
2. `backend/src/services/market_data/providers/jquants.rs` の変換と `backend/src/handlers/dividend_per_share.rs` の API 応答を確認する。
3. API surface を変えた場合は `backend/src/openapi.rs` と `docs/openapi.json` を同期する。
4. `frontend/src/dto.rs`、`dividend_per_share.rs`、`dividend_info.rs` と利用画面を確認する。
5. 関連テストと契約テストを実行する。

## 検証

```bash
bash .claude/skills/jquants-contract-maintenance/scripts/check_jquants_contract.sh
```

追加の確認先は [file-map.md](references/file-map.md) と [symptoms-and-fixes.md](references/symptoms-and-fixes.md) を参照する。
