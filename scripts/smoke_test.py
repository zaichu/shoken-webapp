"""
ローカル WebApp スモークテスト
usage:
  # サーバーが起動済みの場合
  python3 scripts/smoke_test.py

  # サーバー起動から行う場合（with_server.py 経由）
  python3 .claude/skills/webapp-testing/scripts/with_server.py \
    --server "bash scripts/start-local.sh" --port 8080 \
    -- python3 scripts/smoke_test.py

出力先: output/smoke/
"""

import json
import os
import sys
from pathlib import Path
from playwright.sync_api import sync_playwright, expect

BASE_URL = "http://127.0.0.1:8080"
AUTH_STATE = Path(__file__).parent.parent / "frontend" / ".auth" / "storage-state.json"
OUTPUT_DIR = Path(__file__).parent.parent / "output" / "smoke"

PAGES = [
    ("home",          f"{BASE_URL}/"),
    ("search",        f"{BASE_URL}/search"),
    ("assetbalance",  f"{BASE_URL}/assetbalance"),
    ("receipts",      f"{BASE_URL}/receipts"),
]


def save(page, name: str) -> None:
    path = OUTPUT_DIR / f"{name}.png"
    page.screenshot(path=str(path), full_page=True)
    print(f"  保存: {path.relative_to(Path.cwd())}")


def run_smoke(use_auth: bool) -> dict:
    results = {}
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        ctx_opts = {"storage_state": str(AUTH_STATE)} if use_auth else {}
        ctx = browser.new_context(viewport={"width": 1920, "height": 1080}, **ctx_opts)
        page = ctx.new_page()

        # 各主要ページのロードとスクリーンショット
        for name, url in PAGES:
            print(f"→ {name}: {url}")
            page.goto(url, wait_until="networkidle", timeout=20000)
            save(page, name)
            results[name] = {"url": url, "title": page.title(), "ok": True}

        # 銘柄検索操作のスモーク
        print("→ 銘柄検索: '任天堂' を検索")
        page.goto(f"{BASE_URL}/search", wait_until="networkidle", timeout=20000)
        search_input = page.locator("input[placeholder*='銘柄']").first
        if search_input.count() > 0:
            search_input.fill("任天堂")
            page.locator("button:has-text('検索')").click()
            # 「読み込み中」ボタンが消えるまで待つ（外部 API 応答待ち）
            page.locator("button:has-text('読み込み中')").wait_for(state="hidden", timeout=15000)
            page.wait_for_load_state("networkidle", timeout=10000)
            save(page, "search-nintendo")
            # 検索結果テーブルまたは EmptyState の存在を確認
            has_results = page.locator("table").count() > 0
            results["search-op"] = {"ok": True, "note": f"検索操作成功 (結果テーブル: {has_results})"}
        else:
            results["search-op"] = {"ok": False, "note": "検索入力欄が見つからない"}
            print("  警告: 検索入力欄が見つかりませんでした")

        # コンソールエラー確認
        errors = []
        page.on("console", lambda msg: errors.append(msg.text) if msg.type == "error" else None)
        page.goto(f"{BASE_URL}/", wait_until="networkidle", timeout=20000)
        results["console_errors"] = errors

        browser.close()
    return results


def main():
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

    use_auth = AUTH_STATE.exists()
    print(f"認証状態: {'あり' if use_auth else 'なし（未ログイン）'}")
    print(f"出力先: {OUTPUT_DIR}")
    print()

    try:
        results = run_smoke(use_auth)
    except Exception as e:
        print(f"エラー: {e}", file=sys.stderr)
        sys.exit(1)

    # 結果サマリー
    report_path = OUTPUT_DIR / "report.json"
    with open(report_path, "w", encoding="utf-8") as f:
        json.dump(results, f, ensure_ascii=False, indent=2)

    print()
    print("=== スモークテスト結果 ===")
    failed = []
    for key, val in results.items():
        if key == "console_errors":
            if val:
                print(f"  コンソールエラー ({len(val)}件): {val[:3]}")
            continue
        status = "✓" if val.get("ok") else "✗"
        note = val.get("note", val.get("title", ""))
        print(f"  {status} {key}: {note}")
        if not val.get("ok"):
            failed.append(key)

    print(f"\nレポート: {report_path}")
    if failed:
        print(f"失敗: {failed}", file=sys.stderr)
        sys.exit(1)
    else:
        print("全項目 OK")


if __name__ == "__main__":
    main()
