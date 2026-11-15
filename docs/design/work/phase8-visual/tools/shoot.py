"""Phase 8 visual: shoot each idea (PC 1440 / mobile 390 x light/dark, full page)."""
import sys
from pathlib import Path
from playwright.sync_api import sync_playwright

HERE = Path(__file__).resolve().parent
VISUAL = HERE.parent
SHOTS = VISUAL / "shots"
SHOTS.mkdir(exist_ok=True)

IDEAS = ["idea1-minimal", "idea2-finance", "idea3-cards", "idea4-simple", "idea5-current-color"]
VIEWPORTS = {"pc": (1440, 900), "mobile": (390, 844)}

with sync_playwright() as p:
    browser = p.chromium.launch()
    for idea in IDEAS:
        for vp_name, (w, h) in VIEWPORTS.items():
            for theme in ("light", "dark"):
                ctx = browser.new_context(
                    viewport={"width": w, "height": h},
                    color_scheme=theme,
                    device_scale_factor=1,
                    locale="ja-JP",
                )
                page = ctx.new_page()
                url = (VISUAL / f"{idea}.html").as_uri() + f"?theme={theme}"
                page.goto(url, wait_until="load")
                page.wait_for_timeout(1500)
                out = SHOTS / f"{idea}-{vp_name}-{theme}.png"
                page.screenshot(path=str(out), full_page=True)
                print("saved", out, out.stat().st_size)
                ctx.close()
    browser.close()
print("done")
