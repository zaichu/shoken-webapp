import { describe, it, expect, vi, beforeEach, Mock } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { useDividendBatch } from '../useDividendBatch';
import * as api from '../../api/dividendPerShareApi';
import { DividendPerShareItem } from '../../api/dividendPerShareApi';

vi.mock('../../api/dividendPerShareApi', () => ({
  fetchDividendPerShareBatch: vi.fn(),
}));

const mockFetch = api.fetchDividendPerShareBatch as Mock;

const makeItem = (code: string, status: DividendPerShareItem['status'], div: number | null): DividendPerShareItem => ({
  security_code: code,
  dividend_per_share: div,
  status,
  fetched_at: status === 'ok' || status === 'zero' ? '2025-01-01T00:00:00Z' : null,
  is_stale: false,
});

describe('useDividendBatch', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('zero と error のステータスが区別され、dividendPerShareMap には含まれない', async () => {
    mockFetch.mockResolvedValue([
      makeItem('1234', 'zero', 0),
      makeItem('5678', 'error', null),
    ]);

    const { result } = renderHook(() => useDividendBatch(['1234', '5678'], true));

    await waitFor(() => expect(result.current.loading).toBe(false));

    expect(result.current.dividendStatusMap.get('1234')).toBe('zero');
    expect(result.current.dividendStatusMap.get('5678')).toBe('error');
    expect(result.current.dividendPerShareMap.has('1234')).toBe(false);
    expect(result.current.dividendPerShareMap.has('5678')).toBe(false);
  });

  it('ok の配当は dividendPerShareMap に格納され、ok+zero が fetchedCount に加算される', async () => {
    mockFetch.mockResolvedValue([
      makeItem('7203', 'ok', 50),
      makeItem('6758', 'zero', 0),
    ]);

    const { result } = renderHook(() => useDividendBatch(['7203', '6758'], true));

    await waitFor(() => expect(result.current.loading).toBe(false));

    expect(result.current.dividendPerShareMap.get('7203')).toBe(50);
    expect(result.current.dividendPerShareMap.has('6758')).toBe(false);
    expect(result.current.fetchedCount).toBe(2);
    expect(result.current.totalCount).toBe(2);
  });

  it('pending が残る場合は fetchedCount < totalCount になる（再試行条件）', async () => {
    mockFetch.mockResolvedValue([
      makeItem('1111', 'ok', 100),
      makeItem('2222', 'pending', null),
    ]);

    const { result } = renderHook(() => useDividendBatch(['1111', '2222'], true));

    await waitFor(() => expect(result.current.loading).toBe(false));

    expect(result.current.fetchedCount).toBe(1);
    expect(result.current.totalCount).toBe(2);
    expect(result.current.dividendStatusMap.get('2222')).toBe('pending');
  });
});
