#!/bin/bash
# openapi.json と api.ts の同期チェック
# 使い方: scripts/check-openapi.sh
# バックエンドの API 定義に変更があった場合、openapi.json と api.ts が最新かどうかを確認する

set -euo pipefail

REPO_ROOT=$(git rev-parse --show-toplevel)
cd "$REPO_ROOT"

FAILED=0

echo "🔍 openapi.json を再生成して差分を確認..."
(cd backend && cargo run --bin generate_openapi 2>&1)

if ! git diff --quiet docs/openapi.json; then
  echo ""
  echo "❌ openapi.json が古い状態です。以下を実行してコミットしてください:"
  echo "   (cd backend && cargo run --bin generate_openapi)"
  echo "   git add docs/openapi.json"
  echo ""
  FAILED=1
fi

echo "🔍 api.ts を再生成して差分を確認..."
(cd frontend && npm run generate:types 2>&1)

if ! git diff --quiet frontend/src/generated/api.ts; then
  echo ""
  echo "❌ frontend/src/generated/api.ts が古い状態です。以下を実行してコミットしてください:"
  echo "   (cd frontend && npm run generate:types)"
  echo "   git add frontend/src/generated/api.ts"
  echo ""
  FAILED=1
fi

if [ "$FAILED" -eq 1 ]; then
  echo "上記ファイルをコミットしてから再度 push してください。"
  exit 1
fi

echo "✅ openapi.json と api.ts は最新です"
