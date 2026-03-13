"""
ローカル WebApp スモークテスト
usage:
  # サーバーが起動済みの場合
  python3 scripts/smoke_test.py

  # サーバー起動から行う場合（with_server.py 経由）
  python3 .claude/skills/webapp-testing/scripts/with_server.py \
    --server "bash scripts/start-local.sh" --port 8080 \
    -- python3 scripts/smoke_test.py

出力先: output/smoke/  （output/ は .gitignore 対象）
前提: frontend/.auth/storage-state.json が存在すること
"""

import json
import sys
from pathlib import Path
from playwright.sync_api import sync_playwright

BASE_URL = "http://127.0.0.1:8080"
AUTH_STATE = Path(__file__).parent.parent / "frontend" / ".auth" / "storage-state.json"
OUTPUT_DIR = Path(__file__).parent.parent / "output" / "smoke"

# (名前, URL, 認証確認セレクタ)
# - テキスト文字列: page.locator("text=...") で確認（ページヘッダー等）
# - CSS セレクタ（'['で始まる）: 認証後のみ描画される要素で確認（より信頼性が高い）
# assetbalance: [data-testid="assetbalance-workspace"] は isAuthenticated 時のみ描画される
# receipts: auth-only 要素が存在しないため、ページヘッダーテキストで代替（制約あり）
PAGES = [
    ("home",         f"{BASE_URL}/",            "証券Webへようこそ"),
    ("search",       f"{BASE_URL}/search",       "銘柄検索"),
    ("assetbalance", f"{BASE_URL}/assetbalance", '[data-testid="assetbalance-workspace"]'),
    ("receipts",     f"{BASE_URL}/receipts",     "取引明細"),
]


def save(page, name: str) -> None:
    path = OUTPUT_DIR / f"{name}.png"
    page.screenshot(path=str(path), full_page=True)
    print(f"  保存: {path.relative_to(Path.cwd())}")


def run_smoke() -> dict:
    results: dict = {}
    console_errors: list[str] = []
    page_errors: list[str] = []

    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        ctx = browser.new_context(
            viewport={"width": 1920, "height": 1080},
            storage_state=str(AUTH_STATE),
        )
        page = ctx.new_page()

        # console 監視は最初に登録する（全ページ巡回中のエラーも拾う）
        # "Failed to load resource" はネットワーク失敗（外部 API 不通など）なので除外し、
        # JS アプリケーション側のエラーのみを対象にする
        def _on_console(msg):
            if msg.type == "error" and "Failed to load resource" not in msg.text:
                console_errors.append(msg.text)
        page.on("console", _on_console)

        # pageerror 監視（uncaught exception を拾う）
        page.on("pageerror", lambda err: page_errors.append(str(err)))

        # 各主要ページのロードとコンテンツ確認
        for name, url, check_selector in PAGES:
            print(f"→ {name}: {url}")
            page.goto(url, wait_until="networkidle", timeout=20000)
            save(page, name)
            # 認証済みコンテンツの確認
            # - CSS セレクタ（'['で始まる）は locator() で検索
            # - テキスト文字列は text= locator で検索
            if check_selector.startswith("["):
                found = page.locator(check_selector).count() > 0
                label = f"{check_selector} が存在する"
            else:
                found = page.locator(f"text={check_selector}").count() > 0
                label = f"'{check_selector}' が表示されている"
            if found:
                results[name] = {"url": url, "ok": True, "note": label}
            else:
                results[name] = {"url": url, "ok": False, "note": f"{label}（未検出: 認証切れまたはエラーの可能性）"}
                print(f"  警告: {label} を確認できませんでした")

        # 銘柄検索操作のスモーク
        # /stock/{query} はローカル DB を検索する。結果が出れば OK、
        # エラー（DB 未 seed など）は回帰として扱い ok: False にする
        print("→ 銘柄検索: '7974' で検索")
        page.goto(f"{BASE_URL}/search", wait_until="networkidle", timeout=20000)
        search_input = page.locator("input[placeholder*='銘柄']").first
        if search_input.count() > 0:
            search_input.fill("7974")
            page.locator("button:has-text('検索')").click()
            # 検索ボタンがクリック可能に戻るまで待つ（loading 解除 = API 応答完了）
            page.locator("button:has-text('検索')").wait_for(state="visible", timeout=15000)
            page.wait_for_load_state("networkidle", timeout=10000)
            save(page, "search-7974")
            has_table = page.locator("table").count() > 0
            has_error = page.locator("text=エラー").count() > 0
            if has_table:
                results["search-op"] = {"ok": True, "note": "検索結果テーブルが表示された"}
            elif has_error:
                # /stock/{query} はローカル DB を参照するため、エラーは DB 未 seed の可能性がある
                results["search-op"] = {"ok": False, "note": "エラー表示（ローカル DB に銘柄データが未登録の可能性）"}
                print("  警告: 検索結果がエラー表示です（'stock' テーブルに seed データが必要かもしれません）")
            else:
                results["search-op"] = {"ok": False, "note": "検索後に結果もエラーも表示されない（未応答の可能性）"}
                print("  警告: 検索後の状態が不明です")
        else:
            results["search-op"] = {"ok": False, "note": "検索入力欄が見つからない"}
            print("  警告: 検索入力欄が見つかりませんでした")

        results["console_errors"] = console_errors
        results["page_errors"] = page_errors
        browser.close()

    return results


def main():
    # auth state 必須チェック（認証必須ページを正しく検証するために必要）
    if not AUTH_STATE.exists():
        print(
            f"エラー: 認証状態ファイルが見つかりません: {AUTH_STATE}\n"
            "先に認証状態を保存してください:\n"
            "  ./scripts/run-ui-e2e.sh --save-auth --skip-csv",
            file=sys.stderr,
        )
        sys.exit(1)

    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    print(f"認証状態: {AUTH_STATE}")
    print(f"出力先: {OUTPUT_DIR}")
    print()

    try:
        results = run_smoke()
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
                print(f"  ✗ console_errors ({len(val)}件): {val[:3]}")
                failed.append("console_errors")
            continue
        if key == "page_errors":
            if val:
                print(f"  ✗ page_errors ({len(val)}件): {val[:3]}")
                failed.append("page_errors")
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
