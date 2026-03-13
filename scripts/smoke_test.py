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

# (名前, URL, 確認セレクタ)
# - テキスト文字列: page.locator("text=...") で確認
# - CSS セレクタ（'['で始まる）: 動的描画要素。wait_for_selector で描画完了を待ってから確認する
#   - assetbalance: [data-testid="assetbalance-workspace"] は isAuthenticated 時のみ描画
#   - receipts: [data-testid="receipt-card"] は !authLoading && !dbLoading 後に描画
#     （auth-only ではないが、認証 + データロードが完了したことを確認できる）
PAGES = [
    ("home",         f"{BASE_URL}/",            "証券Webへようこそ"),
    ("search",       f"{BASE_URL}/search",       "銘柄検索"),
    ("assetbalance", f"{BASE_URL}/assetbalance", '[data-testid="assetbalance-workspace"]'),
    ("receipts",     f"{BASE_URL}/receipts",     '[data-testid="receipt-card"]'),
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
            # CSS セレクタの場合は動的描画の完了を明示的に待つ
            # （networkidle 直後では認証判定・遅延 import が未完了な場合がある）
            if check_selector.startswith("["):
                try:
                    page.wait_for_selector(check_selector, state="visible", timeout=10000)
                except Exception:
                    pass  # タイムアウト時は count() で false を検出して失敗として記録
            save(page, name)
            # コンテンツ確認
            # - CSS セレクタ（'['で始まる）は locator() で検索 + role="alert" の不在も確認
            #   （workspace/card は API エラー時でも描画されるため、エラーアラートの不在が必要）
            # - テキスト文字列は text= locator で検索
            if check_selector.startswith("["):
                found = page.locator(check_selector).count() > 0
                has_api_error = page.locator('[role="alert"]').count() > 0
                label = f"{check_selector} が存在する"
                if found and not has_api_error:
                    results[name] = {"url": url, "ok": True, "note": label}
                elif found and has_api_error:
                    results[name] = {"url": url, "ok": False, "note": f"{label}（API エラーアラートが表示されている）"}
                    print(f"  失敗: {name} で API エラーアラートが検出されました")
                else:
                    results[name] = {"url": url, "ok": False, "note": f"{label}（未検出: 認証切れまたはエラーの可能性）"}
                    print(f"  警告: {label} を確認できませんでした")
            else:
                found = page.locator(f"text={check_selector}").count() > 0
                label = f"'{check_selector}' が表示されている"
                if found:
                    results[name] = {"url": url, "ok": True, "note": label}
                else:
                    results[name] = {"url": url, "ok": False, "note": f"{label}（未検出: 認証切れまたはエラーの可能性）"}
                    print(f"  警告: {label} を確認できませんでした")

        # 銘柄検索操作のスモーク
        # /stock/{query} はローカル DB を参照する。
        # フロントエンドの catch ブロックが全 Axios エラーを同じメッセージで包むため
        # UI テキストでは 404 と 5xx を区別できない。
        # ネットワーク応答のステータスコードで判定する:
        #   404 → stock テーブル未 seed（ローカル開発での想定範囲内）→ warning (ok: True)
        #   5xx → バックエンド異常 → ok: False
        print("→ 銘柄検索: '7974' で検索")
        page.goto(f"{BASE_URL}/search", wait_until="networkidle", timeout=20000)
        search_input = page.locator("input[placeholder*='銘柄']").first

        stock_response_status: list[int] = []

        def _on_response(response):
            if "/stock/" in response.url:
                stock_response_status.append(response.status)
        page.on("response", _on_response)

        if search_input.count() > 0:
            search_input.fill("7974")
            page.locator("button:has-text('検索')").click()
            # 検索ボタンがクリック可能に戻るまで待つ（loading 解除 = API 応答完了）
            page.locator("button:has-text('検索')").wait_for(state="visible", timeout=15000)
            page.wait_for_load_state("networkidle", timeout=10000)
            save(page, "search-7974")
            has_table = page.locator("table").count() > 0
            status = stock_response_status[-1] if stock_response_status else None
            if has_table:
                results["search-op"] = {"ok": True, "note": f"検索結果テーブルが表示された (HTTP {status})"}
            elif status == 404:
                # 404: stock テーブル未 seed のローカル環境では想定範囲内
                results["search-op"] = {"ok": True, "note": "銘柄未検出（stock テーブル未 seed の可能性。UI は正常応答）"}
                print("  警告: 銘柄コード '7974' が見つかりません（stock テーブルに seed データが必要かもしれません）")
            elif status is not None and status >= 500:
                results["search-op"] = {"ok": False, "note": f"バックエンドエラー (HTTP {status}) が返された"}
                print(f"  失敗: /stock/7974 が HTTP {status} を返しました")
            else:
                has_error = page.locator("text=エラー").count() > 0
                if has_error:
                    results["search-op"] = {"ok": False, "note": f"エラー表示（API ステータス: {status}）"}
                    print(f"  失敗: 検索でエラーが表示されました（HTTP {status}）")
                else:
                    results["search-op"] = {"ok": False, "note": "検索後に結果もエラーも表示されない（UI が無応答の可能性）"}
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
