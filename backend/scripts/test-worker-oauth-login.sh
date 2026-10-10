#!/usr/bin/env bash
# wrangler dev 上の Worker で、制御した Google 応答によるログイン一巡を検証する。
# 前提: docker compose up -d(postgres) と backend/.dev.vars(GOOGLE_CLIENT_ID/SECRET)
#
# GOOGLE_TOKEN_URL / GOOGLE_TOKENINFO_URL を --var でモックへ差し替え、
# authorize -> callback -> セッション発行 -> /api/v1/session まで実 Worker で通す。
set -euo pipefail

BACKEND_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$BACKEND_DIR"
source scripts/test-worker-common.sh

MOCK_PORT="${MOCK_PORT:-8788}"
TMP="$(mktemp -d)"
JAR="$TMP/cookies.txt"
HEADERS="$TMP/headers.txt"
CLAIMS="$TMP/claims.json"
MOCK_LOG="$TMP/mock.log"
WRANGLER_LOG="$TMP/wrangler.log"
MOCK_PID=""

cleanup() {
  [[ -n "$MOCK_PID" ]] && kill "$MOCK_PID" 2>/dev/null || true
  stop_wrangler
}
trap cleanup EXIT

CLIENT_ID="$(grep -oP '^GOOGLE_CLIENT_ID=\K.*' .dev.vars | tr -d '"')" \
  || fail ".dev.vars に GOOGLE_CLIENT_ID がありません"
python3 scripts/mock_google_oauth.py "$MOCK_PORT" "$CLAIMS" "$MOCK_LOG" &
MOCK_PID=$!
start_wrangler \
  --var "GOOGLE_TOKEN_URL:http://127.0.0.1:$MOCK_PORT/token" \
  --var "GOOGLE_TOKENINFO_URL:http://127.0.0.1:$MOCK_PORT/tokeninfo"
wait_for_worker || fail "Worker が起動しませんでした"

# 1. authorize: 303 + state/nonce/PKCE Cookie が返ること
STATUS=$(curl -s -c "$JAR" -o /dev/null -w "%{http_code}" \
  "http://127.0.0.1:$WORKER_PORT/api/v1/oauth/google/authorize")
[[ "$STATUS" == "303" ]] || fail "authorize が $STATUS"
for name in oauth_state oauth_pkce_verifier oauth_nonce; do
  grep -q "$name" "$JAR" || fail "$name Cookie がありません"
done
STATE=$(awk -F'\t' '$6=="oauth_state"{print $7}' "$JAR")
NONCE=$(awk -F'\t' '$6=="oauth_nonce"{print $7}' "$JAR")
echo "1. authorize -> 303 + state/nonce/PKCE Cookie: OK"

# 2. モックの tokeninfo 応答を nonce 一致の制御クレームに設定
python3 - "$CLAIMS" "$CLIENT_ID" "$NONCE" <<'EOF'
import json, sys, time
json.dump({
    "iss": "https://accounts.google.com",
    "aud": sys.argv[2],
    "sub": "worker-test-sub-001",
    "email": "worker-test@example.com",
    "name": "Worker Test",
    "exp": str(int(time.time()) + 3600),
    "iat": str(int(time.time())),
    "nonce": sys.argv[3],
}, open(sys.argv[1], "w"))
EOF

# 3. callback: state 一致 + モック応答 -> セッション発行 + 303
STATUS=$(curl -s -b "$JAR" -c "$JAR" -D "$HEADERS" -o /dev/null -w "%{http_code}" \
  "http://127.0.0.1:$WORKER_PORT/api/v1/oauth/google/callback?code=mock-auth-code&state=$STATE")
[[ "$STATUS" == "303" ]] || { cat "$WRANGLER_LOG" >&2; fail "callback が $STATUS"; }
grep -qi "login=success" "$HEADERS" || fail "callback の Location が login=success でない"
grep -q "session_token" "$JAR" || fail "session_token Cookie が発行されていない"
grep -q "code=mock-auth-code" "$MOCK_LOG" || fail "モックが token 交換を受けていない"
grep -q "code_verifier=" "$MOCK_LOG" || fail "PKCE verifier が送られていない"
grep -q "GET /tokeninfo" "$MOCK_LOG" || fail "モックが tokeninfo を受けていない"
echo "3. callback -> セッション発行 + session_token Cookie + 303(login=success): OK"

# 4. 発行されたセッションで /api/v1/session が通ること
BODY=$(curl -s -b "$JAR" -w $'\n%{http_code}' \
  "http://127.0.0.1:$WORKER_PORT/api/v1/session")
[[ "$(tail -n1 <<<"$BODY")" == "200" ]] || fail "/api/v1/session が 200 でない: $BODY"
grep -q "worker-test@example.com" <<<"$BODY" || fail "ユーザー情報が想定外: $BODY"
echo "4. 発行セッションで /api/v1/session -> 200 + ユーザー情報: OK"

echo "PASS: 制御した Google 応答でログイン一巡が Worker 上で完走した"
