"""opencode担当分の撮影: 完成形 *-mobile(390) / *-pc(1280) x light/dark + 状態 *-state-*-mobile/pc x lightのみ。"""
import sys
from pathlib import Path
from playwright.sync_api import sync_playwright

HERE = Path(__file__).resolve().parent
BASE = HERE.parent
SHOTS = BASE / "shots"
SHOTS.mkdir(exist_ok=True)

COMPLETE = [
    ("assetbalance-mobile", 390, 844),
    ("assetbalance-pc", 1280, 900),
    ("composition-mobile", 390, 844),
    ("composition-pc", 1280, 900),
    ("fund-detail-mobile", 390, 844),
    ("fund-detail-pc", 1280, 900),
    ("receipts-mobile", 390, 844),
    ("receipts-pc", 1280, 900),
    ("csv-mobile", 390, 844),
    ("csv-pc", 1280, 900),
]

STATES = sorted(
    (p.stem, 390 if p.stem.endswith("-mobile") else 1280, 844 if p.stem.endswith("-mobile") else 900)
    for p in BASE.glob("*-state-*.html")
    if p.stem.endswith("-mobile") or p.stem.endswith("-pc")
)

with sync_playwright() as p:
    browser = p.chromium.launch()
    for name, w, h in COMPLETE:
        for theme in ("light", "dark"):
            ctx = browser.new_context(
                viewport={"width": w, "height": h},
                color_scheme=theme,
                device_scale_factor=1,
                locale="ja-JP",
            )
            page = ctx.new_page()
            url = (BASE / f"{name}.html").as_uri() + f"?theme={theme}"
            page.goto(url, wait_until="load")
            page.wait_for_timeout(1500)
            out = SHOTS / f"{name}-{theme}.png"
            if name.endswith("-mobile"):
                full = page.evaluate("() => document.documentElement.scrollHeight")
                page.set_viewport_size({"width": w, "height": full})
                page.wait_for_timeout(400)
                page.screenshot(path=str(out), full_page=False)
            else:
                page.screenshot(path=str(out), full_page=True)
            print("saved", out, out.stat().st_size)
            ctx.close()
    for name, w, h in STATES:
        ctx = browser.new_context(
            viewport={"width": w, "height": h},
            color_scheme="light",
            device_scale_factor=1,
            locale="ja-JP",
        )
        page = ctx.new_page()
        url = (BASE / f"{name}.html").as_uri() + "?theme=light"
        page.goto(url, wait_until="load")
        page.wait_for_timeout(1500)
        out = SHOTS / f"{name}.png"
        if name.endswith("-mobile"):
            full = page.evaluate("() => document.documentElement.scrollHeight")
            page.set_viewport_size({"width": w, "height": full})
            page.wait_for_timeout(400)
            page.screenshot(path=str(out), full_page=False)
        else:
            page.screenshot(path=str(out), full_page=True)
        print("saved", out, out.stat().st_size)
        ctx.close()
    browser.close()
print("done")
