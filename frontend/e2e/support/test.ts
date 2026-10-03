import { test as base, expect } from '@playwright/test';
import { loadBackendOrigin } from '../../scripts/backend-origin.cjs';

const backendOrigin = loadBackendOrigin();

// モックの外れた API 通信を仕組みで止める。context 側に先に登録するので、
// spec 側のモック(page.route は context.route より優先、同レベルは後勝ち)が上書きできる。
// /api/ は全て abort し(proxy 先の無い trunk serve と同じく接続失敗として扱う)、
// 本番 backend への通信は URL を記録してテストを失敗させる。
export const test = base.extend<{ _apiGuard: void }>({
  _apiGuard: [
    async ({ context }, use) => {
      const prodHits: string[] = [];
      await context.route(/\/api\//, (route) => route.abort());
      await context.route(`${backendOrigin}/**`, (route) => {
        prodHits.push(route.request().url());
        return route.abort();
      });
      await use();
      expect(prodHits).toEqual([]);
    },
    { auto: true },
  ],
});

export { expect };

// route.fulfill 用の共通レスポンス。extra は追加フィールド(total 等)の上書き用
export function json(body: unknown, status = 200) {
  return { status, contentType: 'application/json', body: JSON.stringify(body) };
}

export function paginated(data: unknown[], extra: Record<string, unknown> = {}) {
  return { data, total: data.length, page: 1, per_page: Math.max(data.length, 1), ...extra };
}
