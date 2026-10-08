#!/usr/bin/env python3
"""Google OAuth エンドポイントのモック。

wrangler dev の --var GOOGLE_TOKEN_URL / GOOGLE_TOKENINFO_URL で差し替えた
エンドポイントとして使う。test-worker-oauth-login.sh から呼ぶ。

Usage: mock_google_oauth.py <port> <claims_file> <log_file>
  /token    : POST されたフォームを log_file に追記し、固定の id_token JSON を返す
  /tokeninfo: claims_file の内容をそのまま JSON として返す(実行前に書き換え可能)
"""

import json
import sys
import time
from http.server import BaseHTTPRequestHandler, HTTPServer
from urllib.parse import urlparse

PORT = int(sys.argv[1])
CLAIMS_FILE = sys.argv[2]
LOG_FILE = sys.argv[3]


class Handler(BaseHTTPRequestHandler):
    def _send(self, code: int, body: dict) -> None:
        payload = json.dumps(body).encode()
        self.send_response(code)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(payload)))
        self.end_headers()
        self.wfile.write(payload)

    def do_POST(self) -> None:  # noqa: N802
        if urlparse(self.path).path != "/token":
            self._send(404, {"error": "not found"})
            return
        length = int(self.headers.get("Content-Length") or 0)
        body = self.rfile.read(length).decode()
        with open(LOG_FILE, "a") as f:
            f.write(f"POST /token {body}\n")
        self._send(
            200,
            {
                "access_token": "mock-access-token",
                "id_token": "mock-id-token",
                "token_type": "Bearer",
                "expires_in": 3600,
            },
        )

    def do_GET(self) -> None:  # noqa: N802
        if urlparse(self.path).path != "/tokeninfo":
            self._send(404, {"error": "not found"})
            return
        try:
            with open(CLAIMS_FILE) as f:
                claims = json.load(f)
        except OSError:
            self._send(500, {"error": "claims file not ready"})
            return
        with open(LOG_FILE, "a") as f:
            f.write(f"GET {self.path}\n")
        self._send(200, claims)

    def log_message(self, fmt: str, *args: object) -> None:
        pass


if __name__ == "__main__":
    HTTPServer(("127.0.0.1", PORT), Handler).serve_forever()
