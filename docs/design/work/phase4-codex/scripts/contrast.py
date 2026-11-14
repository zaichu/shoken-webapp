from pathlib import Path
import re

root = Path(__file__).resolve().parents[1]
tokens = (root / "tokens.css").read_text()
pairs = [("text", "page"), ("text", "surface"), ("text-muted", "surface"), ("accent", "surface"), ("positive", "surface"), ("negative", "surface"), ("success", "positive-soft"), ("warning", "warning-soft"), ("error", "negative-soft"), ("info", "info-soft"),  ("on-error", "error")]
names = sorted({name for pair in pairs for name in pair})

def colors(theme):
    block = tokens.split(':root[data-theme="dark"]')[1] if theme == "dark" else tokens.split(':root[data-theme="dark"]')[0]
    return {name: re.search(rf"--{re.escape(name)}:\s*(#[0-9a-fA-F]{{6}})", block).group(1) for name in names}

def luminance(hex_color):
    channels = [int(hex_color[i:i + 2], 16) / 255 for i in (1, 3, 5)]
    linear = [c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4 for c in channels]
    return sum(v * w for v, w in zip(linear, (0.2126, 0.7152, 0.0722)))

def ratio(foreground, background):
    light, dark = sorted((luminance(foreground), luminance(background)), reverse=True)
    return (light + 0.05) / (dark + 0.05)

rows = []
for theme in ("light", "dark"):
    palette = colors(theme)
    results = [(fg, bg, ratio(palette[fg], palette[bg])) for fg, bg in pairs]
    failing = [(fg, bg, value) for fg, bg, value in results if value < 4.5]
    if failing:
        raise SystemExit(f"{theme}: {len(failing)} combinations below 4.5:1; lowest={min(failing, key=lambda row: row[2])}")
    for fg, bg, value in results:
        rows.append(f"| {theme} | `--{fg}` `{palette[fg]}` / `--{bg}` `{palette[bg]}` | {value:.2f}:1 | |")

path = root / "foundations.md"
source = path.read_text()
start = source.index("| テーマ | 文字色 / 背景色 | 比率 | 用途 |")
end = source.index("\n\n", start)
table = "| テーマ | 文字色 / 背景色 | 比率 | 用途 |\n|---|---|---:|---|\n" + "\n".join(rows)
path.write_text(source[:start] + table + source[end:])
print(f"{len(rows)} text/background combinations checked; all >= 4.5:1")
