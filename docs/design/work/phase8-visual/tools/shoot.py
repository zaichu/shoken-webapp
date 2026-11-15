"""Phase 8 visual: shoot home-mobile / home-pc (mobile 390 / pc 1280+1440 x light/dark)."""
import sys
from pathlib import Path
from playwright.sync_api import sync_playwright

HERE = Path(__file__).resolve().parent
VISUAL = HERE.parent
SHOTS = VISUAL / "shots"
SHOTS.mkdir(exist_ok=True)

TARGETS = [
    ("home-mobile", 390, 844),
    ("home-pc", 1280, 900),
    ("home-pc", 1440, 900),
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
            url = (VISUAL / f"{name}.html").as_uri() + f"?theme={theme}"
            page.goto(url, wait_until="load")
            page.wait_for_timeout(1500)
            tag = f"{w}" if name == "home-pc" else ""
            stem = f"{name}-{tag}-{theme}" if tag else f"{name}-{theme}"
            out = SHOTS / f"{stem}.png"
            if name == "home-mobile":
                # 固定ナビが full_page 合成の途中に写り込まないよう、
                # 同一HTMLのまま高さをページ全体に広げて撮る
                h = page.evaluate("() => document.documentElement.scrollHeight")
                page.set_viewport_size({"width": w, "height": h})
                page.wait_for_timeout(400)
                page.screenshot(path=str(out), full_page=False)
            else:
                page.screenshot(path=str(out), full_page=True)
            print("saved", out, out.stat().st_size)
            ctx.close()
    browser.close()
print("done")
