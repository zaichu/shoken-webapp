#!/usr/bin/env bash
# test-worker-*.sh 共通ヘルパー。backend/ に cd した後で source して使う。
# wrangler dev の起動(setsid)・/health 待機・プロセスグループ停止と、
# docker compose 経由の psql・テストセッション生成をまとめる。

WORKER_PORT="${WORKER_PORT:-8787}"
COMPOSE_FILE="${COMPOSE_FILE:-docker-compose.yml}"
WRANGLER_PID=""

fail() { echo "FAIL: $*" >&2; exit 1; }

psql() {
  docker compose -f "$COMPOSE_FILE" exec -T postgres \
    psql -tA -U user -d shoken_db -c "$1"
}

# "ユーザーID トークン SHA-256ハッシュ" を1行で出す
# (トークンは Cookie の UUID、DB には SHA-256(正準形文字列)を入れる。services/auth.rs と同じ)
generate_test_session() {
  python3 -c '
import hashlib, uuid
u, t = str(uuid.uuid4()), str(uuid.uuid4())
print(u, t, hashlib.sha256(t.encode()).hexdigest())'
}

# wrangler dev を新セッションで起動。追加の --var などは引数で渡す
start_wrangler() {
  setsid npx wrangler dev --env dev --port "$WORKER_PORT" "$@" >"$WRANGLER_LOG" 2>&1 &
  WRANGLER_PID=$!
}

# /health が応答するまで最大180秒待つ。失敗時は wrangler ログを出して非0を返す
wait_for_worker() {
  echo "wrangler dev 起動待ち..."
  for _ in $(seq 1 180); do
    if curl -sf "http://127.0.0.1:$WORKER_PORT/health" >/dev/null 2>&1; then
      return 0
    fi
    sleep 1
  done
  cat "$WRANGLER_LOG" >&2 || true
  return 1
}

# setsid で新セッション化しているため、プロセスグループごと(workerd まで)止める
stop_wrangler() {
  [[ -n "$WRANGLER_PID" ]] && kill -- -"$WRANGLER_PID" 2>/dev/null || true
}
