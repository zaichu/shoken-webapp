#!/usr/bin/env python3
"""J-Quants API V2 のモック(決算サマリー + 上場銘柄一覧)。

wrangler dev の --var JQUANTS_BASE_URL / JQUANTS_MASTER_BASE_URL で差し替えた
エンドポイントとして使う。test-worker-jquants-cron.sh から呼ぶ。

Usage: mock_jquants.py <port> <log_file>
  GET /v2/fins/summary?code=XXXX :
    code が 9999 のとき 429 を返し、それ以外は制御した配当フィールドを含む
    fins/summary 形式の JSON を返す
  GET /v2/equities/master :
    末尾 5 の優先株と、それと4桁が衝突する普通株を含む3件を返す
    (末尾 0 フィルタと重複除外で普通株2件だけが stock に入るはず)
  いずれも受けたリクエストを log_file に追記する
"""

import json
import sys
from http.server import BaseHTTPRequestHandler, HTTPServer
from urllib.parse import urlparse, parse_qs

PORT = int(sys.argv[1])
LOG_FILE = sys.argv[2]


def _equities_master_body() -> bytes:
    def row(code: str, name: str) -> dict:
        return {
            "Date": "2026-10-15", "Code": code, "CoName": name,
            "CoNameEn": name, "S17": "6", "S17Nm": "情報通信・サービスその他",
            "S33": "3750", "S33Nm": "情報・通信業", "ScaleCat": "TOPIX Small 1",
            "Mkt": "0111", "MktNm": "プライム", "Mrgn": "2", "MrgnNm": "貸借",
            "ProdCat": "011",
        }

    return json.dumps({
        "data": [
            row("13010", "極洋"),
            row("25930", "伊藤園"),
            # 末尾 5 の優先株式: フィルタで除外される(残るのは普通株の "2593")
            row("25935", "伊藤園 優先株式"),
        ]
    }).encode()


class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        parsed = urlparse(self.path)
        with open(LOG_FILE, "a") as f:
            f.write(f"GET {self.path} api_key={self.headers.get('x-api-key')}\n")
        if parsed.path == "/v2/equities/master":
            body = _equities_master_body()
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
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
