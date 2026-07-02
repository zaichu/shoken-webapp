import { describe, it, expect, vi } from 'vitest';
import { fetchAllPages } from '../pagination';

interface Row {
  id: number;
}

function makePage(page: number, total: number, perPage: number): { data: Row[]; total: number; page: number; per_page: number } {
  const start = (page - 1) * perPage;
  const end = Math.min(start + perPage, total);
  const data = Array.from({ length: Math.max(end - start, 0) }, (_, i) => ({ id: start + i + 1 }));
  return { data, total, page, per_page: perPage };
}

describe('fetchAllPages', () => {
  it('total が 1 ページ分に収まる場合は 1 回のみ取得する', async () => {
    const fetchPage = vi.fn((page: number) => Promise.resolve(makePage(page, 3, 1000)));

    const rows = await fetchAllPages(fetchPage);

    expect(rows).toHaveLength(3);
    expect(fetchPage).toHaveBeenCalledTimes(1);
  });

  it('total が 1 ページを超える場合は次ページを取得し全件を返す', async () => {
    const fetchPage = vi.fn((page: number) => Promise.resolve(makePage(page, 2500, 1000)));

    const rows = await fetchAllPages(fetchPage);

    expect(rows).toHaveLength(2500);
    expect(rows[0]).toEqual({ id: 1 });
    expect(rows[2499]).toEqual({ id: 2500 });
    expect(fetchPage).toHaveBeenCalledTimes(3);
    expect(fetchPage).toHaveBeenNthCalledWith(1, 1);
    expect(fetchPage).toHaveBeenNthCalledWith(2, 2);
    expect(fetchPage).toHaveBeenNthCalledWith(3, 3);
  });

  it('total が 0 件の場合は空配列を返す', async () => {
    const fetchPage = vi.fn((page: number) => Promise.resolve(makePage(page, 0, 1000)));

    const rows = await fetchAllPages(fetchPage);

    expect(rows).toEqual([]);
    expect(fetchPage).toHaveBeenCalledTimes(1);
  });
});
