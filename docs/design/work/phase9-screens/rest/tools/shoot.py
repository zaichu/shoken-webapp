"""rest担当分の撮影: 完成形 *-mobile(390) / *-pc(1280) x light/dark + 状態 *-state-* x lightのみ + OS確認。"""
from pathlib import Path
from playwright.sync_api import sync_playwright

HERE = Path(__file__).resolve().parent
BASE = HERE.parent
SHOTS = BASE / "shots"
SHOTS.mkdir(exist_ok=True)

COMPLETE = [
    ("search-mobile", 390, 844),
    ("search-pc", 1280, 900),
    ("settings-mobile", 390, 844),
    ("settings-pc", 1280, 900),
    ("login-mobile", 390, 844),
    ("login-pc", 1280, 900),
    ("loginprompt-mobile", 390, 844),
    ("loginprompt-pc", 1280, 900),
    ("notfound-mobile", 390, 844),
    ("notfound-pc", 1280, 900),
    ("admin-logins-mobile", 390, 844),
    ("admin-logins-pc", 1280, 900),
]

STATES = sorted(
    (p.stem, 390 if p.stem.endswith("-mobile") else 1280, 844 if p.stem.endswith("-mobile") else 900)
    for p in BASE.glob("*-state-*.html")
    if p.stem.endswith("-mobile") or p.stem.endswith("-pc")
)

OSCHECK = [
    ("search-mobile", 390, 844),
    ("settings-pc", 1280, 900),
]


def shoot(page, name, w, h, theme, out):
    url = (BASE / f"{name}.html").as_uri()
    if theme:
        url += f"?theme={theme}"
    page.goto(url, wait_until="load")
    page.wait_for_timeout(1500)
    if name.endswith("-mobile"):
        full = page.evaluate("() => document.documentElement.scrollHeight")
        page.set_viewport_size({"width": w, "height": full})
        page.wait_for_timeout(400)
        page.screenshot(path=str(out), full_page=False)
    else:
        page.screenshot(path=str(out), full_page=True)
    print("saved", out, out.stat().st_size)


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
            shoot(page, name, w, h, theme, SHOTS / f"{name}-{theme}.png")
            ctx.close()
    for name, w, h in STATES:
        ctx = browser.new_context(
            viewport={"width": w, "height": h},
            color_scheme="light",
            device_scale_factor=1,
            locale="ja-JP",
        )
        page = ctx.new_page()
        shoot(page, name, w, h, "light", SHOTS / f"{name}.png")
        ctx.close()
    for name, w, h in OSCHECK:
        for theme in ("light", "dark"):
            ctx = browser.new_context(
                viewport={"width": w, "height": h},
                color_scheme=theme,
                device_scale_factor=1,
                locale="ja-JP",
            )
            page = ctx.new_page()
            shoot(page, name, w, h, None, SHOTS / "os-check" / f"{name}-os-{theme}.png")
            ctx.close()
    browser.close()
print("done")
