#!/usr/bin/env bash
# wrangler dev 上の Worker で、配当キャッシュのエンキュー→cron 消化を検証する。
# 前提: docker compose up -d(postgres) と backend/.dev.vars(GOOGLE_*/JQUANTS_API_KEY)
#
# JQUANTS_BASE_URL を --var でモックへ差し替え、
# セッション付き POST -> pending 行投入 -> /cdn-cgi/local/scheduled で消化、
# まで実 Worker で通す。
set -euo pipefail

BACKEND_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$BACKEND_DIR"

WORKER_PORT="${WORKER_PORT:-8787}"
MOCK_PORT="${MOCK_PORT:-8789}"
TMP="$(mktemp -d)"
MOCK_LOG="$TMP/mock.log"
WRANGLER_LOG="$TMP/wrangler.log"
JAR="$TMP/cookies.txt"
BODY="$TMP/body.json"
MOCK_PID=""
WRANGLER_PID=""
COMPOSE_FILE="$BACKEND_DIR/docker-compose.yml"

cleanup() {
  [[ -n "$MOCK_PID" ]] && kill "$MOCK_PID" 2>/dev/null || true
  # setsid で新セッション化しているため、プロセスグループごと(workerd まで)止める
  [[ -n "$WRANGLER_PID" ]] && kill -- -"$WRANGLER_PID" 2>/dev/null || true
}
trap cleanup EXIT

fail() { echo "FAIL: $*" >&2; exit 1; }

psql() {
  docker compose -f "$COMPOSE_FILE" exec -T postgres \
    psql -tA -U user -d shoken_db -c "$1"
}

python3 scripts/mock_jquants.py "$MOCK_PORT" "$MOCK_LOG" &
MOCK_PID=$!
setsid npx wrangler dev --env dev --port "$WORKER_PORT" \
  --var "JQUANTS_BASE_URL:http://127.0.0.1:$MOCK_PORT/v2/fins/summary" \
  >"$WRANGLER_LOG" 2>&1 &
WRANGLER_PID=$!

echo "wrangler dev 起動待ち..."
for _ in $(seq 1 180); do
  if curl -sf "http://127.0.0.1:$WORKER_PORT/health" >/dev/null 2>&1; then break; fi
  sleep 1
done
curl -sf "http://127.0.0.1:$WORKER_PORT/health" >/dev/null \
  || { cat "$WRANGLER_LOG" >&2; fail "Worker が起動しませんでした"; }

# 0. セッションを直接投入(認証済みリクエスト用)。トークンは Cookie の UUID、
#    DB には SHA-256(正準形文字列)を入れる(services/auth.rs と同じ)
read -r USER_ID TOKEN HASH < <(python3 -c '
import hashlib, uuid
u, t = str(uuid.uuid4()), str(uuid.uuid4())
print(u, t, hashlib.sha256(t.encode()).hexdigest())')
psql "DELETE FROM dividend_per_share_cache WHERE security_code IN ('7203','9999')" >/dev/null
# 再実行冪等化: テストユーザーを消すとセッションは ON DELETE CASCADE で連鎖削除される
psql "DELETE FROM users WHERE google_id='cron-test'" >/dev/null
psql "INSERT INTO users (id, google_id, email) VALUES ('$USER_ID', 'cron-test', 'cron-test@example.com')" >/dev/null
psql "INSERT INTO sessions (user_id, token_hash) VALUES ('$USER_ID', decode('$HASH','hex'))" >/dev/null
echo "0. テストユーザー+セッション投入: OK"

# 1. エンキュー経路: 認証済み POST で pending 行が積まれること
STATUS=$(curl -s -b "$JAR" -c "$JAR" -o "$BODY" -w "%{http_code}" \
  -X POST "http://127.0.0.1:$WORKER_PORT/api/v1/dividend-per-share-estimates" \
  -H "content-type: application/json" \
  -H "Cookie: session_token=$TOKEN" \
  -d '{"security_codes":["7203"]}')
[[ "$STATUS" == "200" ]] || { cat "$WRANGLER_LOG" >&2; fail "batch POST が $STATUS: $(cat "$BODY")"; }
grep -q '"status":"pending"' "$BODY" || fail "レスポンスが pending でない: $(cat "$BODY")"
ROW=$(psql "SELECT status FROM dividend_per_share_cache WHERE security_code='7203'")
[[ "$ROW" == "pending" ]] || fail "7203 行が pending でない: $ROW"
echo "1. POST -> 200 + pending 行投入: OK"

# 2. scheduled イベントを手動発火して消化を確認する(12秒スロットのため待ち得る)
curl -sf -X POST "http://127.0.0.1:$WORKER_PORT/cdn-cgi/local/scheduled?cron=*+*+*+*+*" \
  || fail "scheduled の発火に失敗"
echo "2. scheduled 発火: OK(消化をポーリング)"

for _ in $(seq 1 60); do
  ROW=$(psql "SELECT status, dividend_per_share FROM dividend_per_share_cache WHERE security_code='7203'")
  [[ "$ROW" == ok* ]] && break
  sleep 1
done
STATUS_VAL="${ROW%%|*}"
DPS_VAL="${ROW#*|}"
[[ "$STATUS_VAL" == "ok" ]] || { cat "$WRANGLER_LOG" >&2; fail "7203 が消化されない: $ROW"; }
[[ "$DPS_VAL" == "45.25" ]] || fail "dividend_per_share が想定外: $DPS_VAL"
grep -q "GET /v2/fins/summary?code=7203" "$MOCK_LOG" || fail "モックが fins/summary を受けていない"
grep -q "api_key=test-jquants-key" "$MOCK_LOG" || fail "x-api-key が送られていない"
echo "3. cron 消化 -> status=ok, dividend_per_share=45.25: OK"

# 3b. 429 -> error + cooldown(未来 stale_at)行になること
psql "INSERT INTO dividend_per_share_cache (security_code, status, provider, updated_at)
      VALUES ('9999','pending','jquants',NOW())
      ON CONFLICT (security_code) DO UPDATE SET status='pending', stale_at=NULL, updated_at=NOW()" \
  >/dev/null
psql "DELETE FROM market_data_provider_rate_control WHERE provider='jquants'" >/dev/null
curl -sf -X POST "http://127.0.0.1:$WORKER_PORT/cdn-cgi/local/scheduled?cron=*+*+*+*+*" \
  || fail "2回目の scheduled 発火に失敗"
for _ in $(seq 1 30); do
  ROW=$(psql "SELECT status, stale_at > NOW() FROM dividend_per_share_cache WHERE security_code='9999'")
  [[ "$ROW" == "error|t" ]] && break
  sleep 1
done
[[ "$ROW" == "error|t" ]] || { cat "$WRANGLER_LOG" >&2; fail "9999 が 429->cooldown error にならない: $ROW"; }
echo "3b. 429 -> error + cooldown 行: OK"

echo "PASS: エンキュー->cron消化->cooldown が実 Worker 上で完走した"
