#!/usr/bin/env bash
# wrangler dev 上の Worker で CSV 系(multipart + Shift_JIS + bulk insert)を検証する。
# 前提: docker compose up -d(postgres) と backend/.dev.vars
#
# セッション付き multipart POST -> validate(200)->import(201)->DB 行確認、
# Shift_JIS 固有文字のデコード確認、CPU 見積もり用の壁時計計測まで通す。
set -euo pipefail

BACKEND_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$BACKEND_DIR"

WORKER_PORT="${WORKER_PORT:-8787}"
ROWS="${ROWS:-500}"
TMP="$(mktemp -d)"
WRANGLER_LOG="$TMP/wrangler.log"
JAR="$TMP/cookies.txt"
BODY="$TMP/body.json"
CSV_FILE="$TMP/dividends.csv"
WRANGLER_PID=""
COMPOSE_FILE="$BACKEND_DIR/docker-compose.yml"

cleanup() {
  # setsid で新セッション化しているため、プロセスグループごと(workerd まで)止める
  [[ -n "$WRANGLER_PID" ]] && kill -- -"$WRANGLER_PID" 2>/dev/null || true
}
trap cleanup EXIT

fail() { echo "FAIL: $*" >&2; exit 1; }

psql() {
  docker compose -f "$COMPOSE_FILE" exec -T postgres \
    psql -tA -U user -d shoken_db -c "$1"
}

# Shift_JIS の配当 CSV(入金日,商品,口座,銘柄コード,銘柄,受取通貨,…)を生成。
# 全角固有文字(ＵＦＪ・㈱)が Shift_JIS 経路で正しく復元されるかを同時に検証する
python3 - "$CSV_FILE" "$ROWS" <<'PYEOF'
import sys
path, n = sys.argv[1], int(sys.argv[2])
header = "入金日,商品,口座,銘柄コード,銘柄,受取通貨,単価[円/現地通貨],数量[株/口],配当・分配金合計（税引前）[円/現地通貨],税額合計[円/現地通貨],受取金額[円/現地通貨]"
names = ["三菱ＵＦＪフィナンシャル・グループ", "オリックス", "㈱テスト証券", "トヨタ自動車"]
lines = [header]
for i in range(n):
    day = 1 + (i % 28)
    name = names[i % len(names)]
    code = str(7000 + (i % 500))
    lines.append(
        f'"2025/03/{day:02d}","国内株式","特定・一般","{code}","{name}","円",'
        f'"1,000.50","100","10,050","2,040","8,010"'
    )
data = "\n".join(lines) + "\n"
with open(path, "wb") as f:
    # 証券会社の CSV 実体は CP932(Windows-31J)。㈱のような拡張文字を含めるため cp932 で出力する
    f.write(data.encode("cp932"))
PYEOF

setsid npx wrangler dev --env dev --port "$WORKER_PORT" >"$WRANGLER_LOG" 2>&1 &
WRANGLER_PID=$!

echo "wrangler dev 起動待ち..."
for _ in $(seq 1 180); do
  if curl -sf "http://127.0.0.1:$WORKER_PORT/health" >/dev/null 2>&1; then break; fi
  sleep 1
done
curl -sf "http://127.0.0.1:$WORKER_PORT/health" >/dev/null \
  || { cat "$WRANGLER_LOG" >&2; fail "Worker が起動しませんでした"; }

# 0. セッションを直接投入(トークンは Cookie の UUID、DB には SHA-256 ハッシュ)
read -r USER_ID TOKEN HASH < <(python3 -c '
import hashlib, uuid
u, t = str(uuid.uuid4()), str(uuid.uuid4())
print(u, t, hashlib.sha256(t.encode()).hexdigest())')
psql "DELETE FROM dividends" >/dev/null
psql "DELETE FROM users WHERE google_id='csv-test'" >/dev/null
psql "INSERT INTO users (id, google_id, email) VALUES ('$USER_ID', 'csv-test', 'csv-test@example.com')" >/dev/null
psql "INSERT INTO sessions (user_id, token_hash) VALUES ('$USER_ID', decode('$HASH','hex'))" >/dev/null
echo "0. テストユーザー+セッション投入: OK"

# 1. validate: multipart + Shift_JIS デコード + 行 transform(純 CPU 経路)
T_VALIDATE=$(curl -s -b "$JAR" -c "$JAR" -o "$BODY" -w "%{time_total}" \
  -X POST "http://127.0.0.1:$WORKER_PORT/api/v1/dividend-import-validations" \
  -H "Cookie: session_token=$TOKEN" \
  -F "file=@$CSV_FILE;type=text/csv;filename=dividends.csv")
grep -q "\"total_rows\":$ROWS" "$BODY" || { cat "$WRANGLER_LOG" >&2; fail "validate 応答が不正: $(head -c 300 "$BODY")"; }
grep -q "\"valid_rows\":$ROWS" "$BODY" || fail "valid_rows != $ROWS: $(head -c 300 "$BODY")"
echo "1. validate($ROWS 行, Shift_JIS) -> 200 + 全行 valid: OK (${T_VALIDATE}s)"

# 2. import: bulk insert まで(DB 書き込みあり)
T_IMPORT=$(curl -s -b "$JAR" -c "$JAR" -o "$BODY" -w "%{time_total}" \
  -X POST "http://127.0.0.1:$WORKER_PORT/api/v1/dividend-imports" \
  -H "Cookie: session_token=$TOKEN" \
  -F "file=@$CSV_FILE;type=text/csv;filename=dividends.csv")
grep -q "\"inserted\":$ROWS" "$BODY" || { cat "$WRANGLER_LOG" >&2; fail "import 応答が不正: $(head -c 300 "$BODY")"; }
COUNT=$(psql "SELECT COUNT(*) FROM dividends WHERE user_id='$USER_ID'")
[[ "$COUNT" == "$ROWS" ]] || fail "DB 行数が不一致: $COUNT != $ROWS"
echo "2. import($ROWS 行) -> 201 + inserted=$ROWS + DB 一致: OK (${T_IMPORT}s)"

# 3. Shift_JIS 固有文字の復元確認(ＵＦＪ・㈱が壊れていないこと)
NAME=$(psql "SELECT security_name FROM dividends WHERE user_id='$USER_ID' AND security_name LIKE '%ＵＦＪ%' LIMIT 1")
[[ "$NAME" == "三菱ＵＦＪフィナンシャル・グループ" ]] || fail "Shift_JIS デコード結果が不正: $NAME"
NAME2=$(psql "SELECT security_name FROM dividends WHERE user_id='$USER_ID' AND security_name LIKE '㈱%' LIMIT 1")
[[ "$NAME2" == "㈱テスト証券" ]] || fail "㈱ の復元に失敗: $NAME2"
echo "3. Shift_JIS 固有文字(ＵＦＪ/㈱)の復元: OK"

# 4. エラー経路: .csv 以外の拡張子は 400
STATUS=$(curl -s -o "$BODY" -w "%{http_code}" \
  -X POST "http://127.0.0.1:$WORKER_PORT/api/v1/dividend-imports" \
  -H "Cookie: session_token=$TOKEN" \
  -F "file=@$CSV_FILE;type=text/plain;filename=dividends.txt")
[[ "$STATUS" == "400" ]] || fail "拡張子違反が 400 でない: $STATUS"
echo "4. 拡張子違反 -> 400 CSV_ERROR: OK"

echo "PASS: multipart + Shift_JIS + bulk insert が実 Worker 上で完走した"
echo "計測: validate=${T_VALIDATE}s import=${T_IMPORT}s (${ROWS} 行)"
