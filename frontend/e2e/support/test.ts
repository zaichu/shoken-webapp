import { test as base, expect, type BrowserContext, type Page } from '@playwright/test';
import { createHash } from 'node:crypto';
import { loadBackendOrigin, loadProductionCsp } from '../../scripts/backend-origin.cjs';

const backendOrigin = loadBackendOrigin();
const productionCsp = loadProductionCsp();

// モックの外れた API 通信を仕組みで止める。context 側に先に登録するので、
// spec 側のモック(page.route は context.route より優先、同レベルは後勝ち)が上書きできる。
// /api/ は全て abort し(proxy 先の無い trunk serve と同じく接続失敗として扱う)、
// 本番 backend への通信は URL を記録してテストを失敗させる。
//
// _cspGuard: document レスポンスに本番と同じ CSP を付ける。trunk serve は _headers を
// 読まないため route で差し込む(serve-dist では同じ値の付け直し)。CSP 違反の
// console メッセージがあればテストを失敗させる。
// _apiGuard より先に登録して、後から登録される /api/ abort・spec モックが必ず優先される形にする
export const test = base.extend<{ _cspGuard: void; _apiGuard: void }>({
  _cspGuard: [
    async ({ context, page }, use) => {
      await context.route('**/*', async (route) => {
        if (route.request().resourceType() !== 'document') {
          return route.continue();
        }
        const response = await route.fetch();
        const body = await response.text();
        // trunk serve が差し込む wasm bootstrap は nonce 付きインライン script なので、
        // 拾った nonce を script-src に足す。dist 配信では外部化済みで nonce は出ない
        const nonces = [
          ...new Set(
            [...body.matchAll(/<script[^>]*\snonce="([^"]+)"/g)].map(
              (match) => `'nonce-${match[1]}'`,
            ),
          ),
        ];
        // trunk serve (0.22) のインライン bootstrap は nonce が付かないため、
        // その script の sha256 を script-src に足す。本番の _headers は変えない
        // (本番では prepare-dist.mjs で外部化済み)。
        const inlineHashes = [
          ...new Set(
            [...body.matchAll(/<script\b(?![^>]*\bsrc=)(?![^>]*\bnonce=)[^>]*>([\s\S]*?)<\/script>/g)]
              .map((match) => match[1])
              .filter((content) => content.trim().length > 0)
              .map(
                (content) =>
                  `'sha256-${createHash('sha256').update(content).digest('base64')}'`,
              ),
          ),
        ];
        const tokens = [...nonces, ...inlineHashes];
        const csp = tokens.length
          ? productionCsp.replace(
              "script-src 'self'",
              `script-src 'self' ${tokens.join(' ')}`,
            )
          : productionCsp;
        await route.fulfill({
          response,
          headers: {
            ...response.headers(),
            'content-security-policy': csp,
          },
          body,
        });
      });
      const violations: string[] = [];
      page.on('console', (message) => {
        if (/Content Security Policy|Refused to (?:apply|load|execute)/.test(message.text())) {
          violations.push(message.text());
        }
      });
      await use();
      expect(violations).toEqual([]);
    },
    { auto: true },
  ],
  _apiGuard: [
    async ({ context, _cspGuard }, use) => {
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

export const MOCK_USER = {
  id: '00000000-0000-0000-0000-000000000002',
  email: 'test@example.com',
  name: 'テストユーザー',
};

export async function mockSession(
  target: Page | BrowserContext,
  user: unknown = MOCK_USER,
  status = 200,
) {
  await target.route(/\/api\/v1\/session$/, (route) => route.fulfill(json(user, status)));
}
