// 数字と記号の書体比較(#2042)の撮影用。Amount([data-amount])の font-family だけを候補に替える。
// 通常の撮影(--numerals なし)では読み込まれない。設計は docs/design/numerals.md。

export const UNICODE_RANGE = 'U+0025, U+002B-002C, U+002E, U+0030-0039, U+00A5, U+2212';

export const NUMERAL_CANDIDATES = [
  {
    key: 'inter',
    source: 'Inter',
    family: 'Shoken Numerals A',
    faces: [{ url: '/fonts/numerals/inter.woff2', weight: '400 700' }],
  },
  {
    key: 'roboto',
    source: 'Roboto',
    family: 'Shoken Numerals B',
    faces: [{ url: '/fonts/numerals/roboto.woff2', weight: '400 700' }],
  },
  {
    key: 'manrope',
    source: 'Manrope',
    family: 'Shoken Numerals C',
    faces: [{ url: '/fonts/numerals/manrope.woff2', weight: '400 700' }],
  },
  {
    key: 'figtree',
    source: 'Figtree',
    family: 'Shoken Numerals D',
    faces: [{ url: '/fonts/numerals/figtree.woff2', weight: '400 700' }],
  },
  {
    key: 'barlow',
    source: 'Barlow',
    family: 'Shoken Numerals E',
    faces: [
      { url: '/fonts/numerals/barlow-400.woff2', weight: '400' },
      { url: '/fonts/numerals/barlow-700.woff2', weight: '700' },
    ],
  },
];

export function parseNumerals(value) {
  if (value === 'all') {
    return NUMERAL_CANDIDATES;
  }
  const keys = value.split(',').map((key) => key.trim());
  const candidates = keys.map((key) => NUMERAL_CANDIDATES.find((candidate) => candidate.key === key));
  if (candidates.some((candidate) => !candidate)) {
    throw new Error(`--numerals は ${NUMERAL_CANDIDATES.map((c) => c.key).join(',')} か all です: ${value}`);
  }
  return candidates;
}

// ページ内で実行する。CSP(style-src 'self')のもとで <style> を足せないため、
// FontFace と構築可能なスタイルシートで当てる。負号は表示の文字ノードだけ U+2212 にする
// (色の判定は Rust 側の元の文字列で済んでいるので変わらない)
function applyCandidate({ family, faces, unicodeRange }) {
  for (const face of faces) {
    document.fonts.add(
      new FontFace(family, `url(${face.url}) format('woff2')`, {
        weight: face.weight,
        style: 'normal',
        display: 'block',
        unicodeRange,
      }),
    );
  }
  const sheet = new CSSStyleSheet();
  sheet.replaceSync(`[data-amount] { font-family: "${family}", var(--font-sans); }`);
  document.adoptedStyleSheets = [...document.adoptedStyleSheets, sheet];

  const replaceMinus = () => {
    for (const amount of document.querySelectorAll('[data-amount]')) {
      const walker = document.createTreeWalker(amount, NodeFilter.SHOW_TEXT);
      for (let node = walker.nextNode(); node; node = walker.nextNode()) {
        if (node.data.includes('-')) {
          node.data = node.data.replaceAll('-', '−');
        }
      }
    }
  };
  const start = () => {
    replaceMinus();
    new MutationObserver(replaceMinus).observe(document.body, {
      subtree: true,
      childList: true,
      characterData: true,
    });
  };
  if (document.body) {
    start();
  } else {
    document.addEventListener('DOMContentLoaded', start, { once: true });
  }
}

export async function installCandidate(context, candidate) {
  await context.addInitScript(applyCandidate, {
    family: candidate.family,
    faces: candidate.faces,
    unicodeRange: UNICODE_RANGE,
  });
}

// 候補の書体が読み込まれ、実際に Amount の描画に使われたことを確かめる
export async function assertCandidateUsed(page, candidate) {
  const faces = await page.evaluate(async (family) => {
    await document.fonts.ready;
    await Promise.all([
      document.fonts.load(`400 16px "${family}"`, '0123456789'),
      document.fonts.load(`700 16px "${family}"`, '0123456789'),
    ]);
    return [...document.fonts]
      .filter((face) => face.family.replaceAll('"', '') === family)
      .map((face) => ({ weight: face.weight, status: face.status }));
  }, candidate.family);
  if (faces.length !== candidate.faces.length || faces.some((face) => face.status !== 'loaded')) {
    throw new Error(`${candidate.key}: 書体が読み込めていません ${JSON.stringify(faces)}`);
  }
  const used = await platformFontsOfFirstAmount(page);
  if (used && !used.some((font) => font.familyName === candidate.family)) {
    throw new Error(`${candidate.key}: Amount の描画に候補が使われていません ${JSON.stringify(used)}`);
  }
  return used;
}

async function platformFontsOfFirstAmount(page) {
  const session = await page.context().newCDPSession(page);
  try {
    await session.send('DOM.enable');
    await session.send('CSS.enable');
    const { root } = await session.send('DOM.getDocument', { depth: -1 });
    const { nodeIds } = await session.send('DOM.querySelectorAll', {
      nodeId: root.nodeId,
      selector: '[data-amount]',
    });
    for (const nodeId of nodeIds) {
      const { fonts } = await session.send('CSS.getPlatformFontsForNode', { nodeId });
      if (fonts.length > 0) {
        return fonts;
      }
    }
    return null;
  } finally {
    await session.detach();
  }
}

// baseline と比べるための状態。Amount の見た目の指定と主な枠の位置を記録する
export async function collectManifest(page) {
  return page.evaluate(() => {
    const rect = (element) => {
      const box = element.getBoundingClientRect();
      return [box.x, box.y, box.width, box.height].map((value) => Math.round(value * 10) / 10);
    };
    const amounts = [...document.querySelectorAll('[data-amount]')].map((element) => {
      const style = getComputedStyle(element);
      return {
        text: element.textContent.replaceAll('−', '-'),
        color: style.color,
        fontSize: style.fontSize,
        fontWeight: style.fontWeight,
        letterSpacing: style.letterSpacing,
        lineHeight: style.lineHeight,
        fontVariantNumeric: style.fontVariantNumeric,
        fontFamily: style.fontFamily,
        rect: rect(element),
      };
    });
    const containers = [...document.querySelectorAll('[data-testid]')]
      .filter((element) => !element.closest('[data-amount]'))
      .slice(0, 60)
      .map((element) => ({ testid: element.dataset.testid, rect: rect(element) }));
    return { amounts, containers };
  });
}

const STYLE_KEYS = ['text', 'color', 'fontSize', 'fontWeight', 'letterSpacing', 'lineHeight', 'fontVariantNumeric'];

// 変えてはいけない値(文言・色・大きさ・太さ・字間・行の高さ)と、字幅で動いてよい位置を分けて数える
export function compareManifest(baseline, candidate) {
  const result = { amountCount: [baseline.amounts.length, candidate.amounts.length], styleDiffs: [], heightDiffs: 0, positionDiffs: 0 };
  const count = Math.min(baseline.amounts.length, candidate.amounts.length);
  for (let index = 0; index < count; index += 1) {
    const before = baseline.amounts[index];
    const after = candidate.amounts[index];
    for (const key of STYLE_KEYS) {
      if (before[key] !== after[key]) {
        result.styleDiffs.push({ index, key, before: before[key], after: after[key] });
      }
    }
    if (before.rect[3] !== after.rect[3]) {
      result.heightDiffs += 1;
    }
  }
  // 同じ testid が並ぶ(一覧の行など)ため、出現順で突き合わせる
  const keyed = (items) => {
    const seen = new Map();
    return items.map((item) => {
      const index = seen.get(item.testid) ?? 0;
      seen.set(item.testid, index + 1);
      return [`${item.testid}#${index}`, item.rect];
    });
  };
  const containers = new Map(keyed(baseline.containers));
  result.containerDiffs = [];
  for (const [key, rect] of keyed(candidate.containers)) {
    const item = { testid: key, rect };
    const before = containers.get(key);
    if (before && (before[0] !== item.rect[0] || before[1] !== item.rect[1] || before[2] !== item.rect[2])) {
      result.containerDiffs.push({ testid: item.testid, before, after: item.rect });
    }
  }
  return result;
}

const SPECIMEN_GLYPHS = ['0123456789', ', . ¥ % + −'];
const SPECIMEN_AMOUNTS = ['¥28,588,148', '+¥1,547,488  +5.7%', '−¥232,318  −0.8%', '¥3,456.78  0.85%  1,234株  4.5万円'];

export function specimenHtml(candidate) {
  const faces = candidate.faces
    .map(
      (face) => `@font-face { font-family: "${candidate.family}"; src: url(${face.url}) format('woff2');
  font-weight: ${face.weight}; font-display: block; unicode-range: ${UNICODE_RANGE}; }`,
    )
    .join('\n');
  const family = `"${candidate.family}", 'Noto Sans JP', sans-serif`;
  const rows = (weight) => `
    <section>
      <h2>太さ ${weight}</h2>
      ${SPECIMEN_GLYPHS.map((text) => `<p class="glyphs" style="font-weight:${weight}">${text}</p>`).join('')}
      ${SPECIMEN_AMOUNTS.map((text) => `<p class="amount" style="font-weight:${weight}">${text}</p>`).join('')}
    </section>`;
  return `<!doctype html><html lang="ja"><head><meta charset="utf-8"><title>${candidate.source}</title><style>
@font-face { font-family: 'Noto Sans JP'; src: url(/fonts/NotoSansJP-400-700.woff2) format('woff2'); font-weight: 400 700; }
${faces}
:root { color-scheme: light dark; }
body { margin: 0; padding: 32px 40px; font-family: 'Noto Sans JP', sans-serif; background: #f3f5f8; color: #111827; }
@media (prefers-color-scheme: dark) { body { background: #0f172a; color: #e5e7eb; } }
h1 { margin: 0 0 4px; font-size: 20px; }
.note { margin: 0 0 16px; font-size: 13px; opacity: .75; }
h2 { margin: 16px 0 0; font-size: 13px; font-weight: 400; opacity: .75; }
section { display: inline-block; vertical-align: top; width: 590px; }
.glyphs { margin: 0; font-family: ${family}; font-size: 72px; line-height: 1.25; font-variant-numeric: tabular-nums; }
.amount { margin: 4px 0 0; font-family: ${family}; font-size: 24px; font-variant-numeric: tabular-nums; }
</style></head><body>
<h1>${candidate.source}(内部名 ${candidate.family})</h1>
<p class="note">数字と記号 16 字だけを ${candidate.source} にし、日本語は Noto Sans JP のまま。等幅数字(tabular-nums)。負号は U+2212</p>
${rows(400)}${rows(700)}
</body></html>`;
}
