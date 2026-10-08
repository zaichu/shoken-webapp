#!/usr/bin/env python3
"""J-Quants API V2 決算サマリーのモック。

wrangler dev の --var JQUANTS_BASE_URL で差し替えたエンドポイントとして使う。
test-worker-jquants-cron.sh から呼ぶ。

Usage: mock_jquants.py <port> <log_file>
  GET /v2/fins/summary?code=XXXX :
    code が 9999 のとき 429 を返し、それ以外は制御した配当フィールドを含む
    fins/summary 形式の JSON を返す。受けたリクエストを log_file に追記する
"""

import json
import sys
from http.server import BaseHTTPRequestHandler, HTTPServer
from urllib.parse import urlparse, parse_qs

PORT = int(sys.argv[1])
LOG_FILE = sys.argv[2]


class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        parsed = urlparse(self.path)
        with open(LOG_FILE, "a") as f:
            f.write(f"GET {self.path} api_key={self.headers.get('x-api-key')}\n")
        if parsed.path != "/v2/fins/summary":
            self.send_response(404)
            self.end_headers()
            return
        code = parse_qs(parsed.query).get("code", [""])[0]
        if code == "9999":
            self.send_response(429)
            self.end_headers()
            return
        body = json.dumps({
            "data": [{
                "DiscDate": "2024-05-10", "Code": code, "DocType": "FY",
                "NxFDivAnn": "", "FDivAnn": "45.25", "DivAnn": "40.00",
            }]
        }).encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *args):
        pass


HTTPServer(("127.0.0.1", PORT), Handler).serve_forever()
