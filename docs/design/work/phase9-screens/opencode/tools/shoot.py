"""opencode担当分の撮影: *-mobile(390) / *-pc(1280) x light/dark。"""
import sys
from pathlib import Path
from playwright.sync_api import sync_playwright

HERE = Path(__file__).resolve().parent
BASE = HERE.parent
SHOTS = BASE / "shots"
SHOTS.mkdir(exist_ok=True)

TARGETS = [
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

with sync_playwright() as p:
    browser = p.chromium.launch()
    for name, w, h in TARGETS:
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
    browser.close()
print("done")
