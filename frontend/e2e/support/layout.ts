import type { Page } from '@playwright/test';
import { expect } from './test';

// .app-scroll を最下端までスクロールしたとき、頁枠の本文末尾がフッターの上端を
// 越えないことを確かめる(頁枠が縮むと本文が枠の下へはみ出してフッターに重なる)
export async function expectFooterBelowContent(page: Page) {
  const info = await page.evaluate(() => {
    const scroller = document.querySelector<HTMLElement>('.app-scroll');
    const footer = scroller?.querySelector<HTMLElement>(':scope > footer');
    const frame = scroller?.querySelector<HTMLElement>('.page-frame');
    const body = frame?.lastElementChild as HTMLElement | null;
    if (!scroller || !footer || !frame || !body) return null;
    scroller.scrollTop = scroller.scrollHeight - scroller.clientHeight;
    return {
      footerTop: footer.getBoundingClientRect().top,
      bodyBottom: body.getBoundingClientRect().bottom,
      scrollHeight: scroller.scrollHeight,
      viewportHeight: scroller.clientHeight,
      frameScrollHeight: frame.scrollHeight,
    };
  });
  expect(info, '.app-scroll / footer / .page-frame が見つからない').not.toBeNull();
  expect(
    info!.footerTop,
    `フッターの上端(${info!.footerTop.toFixed(1)})が本文の末尾(${info!.bodyBottom.toFixed(1)})より上にあり、本文と重なっている`,
  ).toBeGreaterThanOrEqual(info!.bodyBottom);
}
