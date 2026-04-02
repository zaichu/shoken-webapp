import { describe, it, expect, vi, beforeEach, afterEach, Mock } from 'vitest';
import { act, renderHook, waitFor } from '@testing-library/react';
import { useJQuantsDividendBatch } from '../useJQuantsDividendBatch';
import * as dividendHook from '../useJQuantsDividend';
import { jquantsApiClient } from '../../api/client';
import { parseNumber } from '@/lib/utils/formatters';

vi.mock('../../api/client', () => ({
  jquantsApiClient: {
    getStatements: vi.fn(),
  },
}));

vi.mock('@/lib/utils/formatters', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/lib/utils/formatters')>();
  return {
    ...actual,
    parseNumber: vi.fn(),
  };
});

vi.mock('../useJQuantsDividend', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../useJQuantsDividend')>();
  return {
    ...actual,
    extractDividendFromSummary: vi.fn(actual.extractDividendFromSummary),
  };
});

const flushAsyncUpdates = async () => {
  await act(async () => {
    await Promise.resolve();
    await Promise.resolve();
  });
};

describe('useJQuantsDividendBatch', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    (parseNumber as Mock).mockImplementation((val) => Number(val) || 0);
    (dividendHook.extractDividendFromSummary as Mock).mockImplementation((summary) => {
      if (summary.NxFDivAnn && summary.NxFDivAnn !== '') return summary.NxFDivAnn;
      if (summary.FDivAnn && summary.FDivAnn !== '') return summary.FDivAnn;
      if (summary.DivAnn && summary.DivAnn !== '') return summary.DivAnn;
      return null;
    });
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('同時実行数を3件以下に制限する', async () => {
    const codes = ['1605', '2933', '3627', '4912', '7974', '8591'];
    let inFlight = 0;
    let maxInFlight = 0;
    const resolvers: Array<() => void> = [];

    (jquantsApiClient.getStatements as Mock).mockImplementation((code: string) => {
      inFlight += 1;
      maxInFlight = Math.max(maxInFlight, inFlight);
      return new Promise((resolve) => {
        resolvers.push(() => {
          inFlight -= 1;
          resolve({
            data: [{ DiscDate: '2025-01-01', NxFDivAnn: '10.0', Code: code }],
          });
        });
      });
    });

    const { result } = renderHook(() => useJQuantsDividendBatch(codes, true));

    await waitFor(() => {
      expect(jquantsApiClient.getStatements).toHaveBeenCalledTimes(3);
    });

    resolvers.splice(0, 3).forEach((resolve) => resolve());

    await waitFor(() => {
      expect(jquantsApiClient.getStatements).toHaveBeenCalledTimes(6);
    });

    resolvers.splice(0).forEach((resolve) => resolve());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(maxInFlight).toBeLessThanOrEqual(3);
    expect(result.current.dividendPerShareMap.size).toBe(6);
  });

  it('value=null の銘柄がある場合は 60 秒待機後に再試行し、回復後に map へ追加する', async () => {
    vi.useFakeTimers();
    const codes = ['1605', '2933'];

    const extractionCount: Record<string, number> = {};

    (jquantsApiClient.getStatements as Mock).mockImplementation((code: string) =>
      Promise.resolve({
        data: [{ DiscDate: '2025-01-01', NxFDivAnn: code === '1605' ? '30.0' : '15.5', Code: code }],
      })
    );

    (dividendHook.extractDividendFromSummary as Mock).mockImplementation((summary) => {
      extractionCount[summary.Code] = (extractionCount[summary.Code] ?? 0) + 1;
      if (summary.Code === '2933' && extractionCount[summary.Code] === 1) {
        return null;
      }
      return summary.NxFDivAnn;
    });

    const { result } = renderHook(() => useJQuantsDividendBatch(codes, true));

    await flushAsyncUpdates();

    expect(jquantsApiClient.getStatements).toHaveBeenCalledTimes(2);
    expect(result.current.loading).toBe(false);
    expect(result.current.dividendPerShareMap.get('1605')).toBe(30);
    expect(result.current.dividendPerShareMap.has('2933')).toBe(false);

    await act(async () => {
      vi.advanceTimersByTime(60_001);
    });
    await flushAsyncUpdates();

    expect(jquantsApiClient.getStatements).toHaveBeenCalledTimes(4);
    expect(result.current.loading).toBe(false);
    expect(result.current.dividendPerShareMap.get('2933')).toBe(15.5);
  });

  it('抽出処理の失敗が続く銘柄は MAX_RETRIES 超過後に再試行を停止する', async () => {
    vi.useFakeTimers();
    const codes = ['1605', '2933'];

    (jquantsApiClient.getStatements as Mock).mockImplementation((code: string) =>
      Promise.resolve({
        data: [{ DiscDate: '2025-01-01', NxFDivAnn: code === '1605' ? '30.0' : '15.5', Code: code }],
      })
    );

    (dividendHook.extractDividendFromSummary as Mock).mockImplementation((summary) => {
      if (summary.Code === '2933') {
        throw new Error('forced failure');
      }
      return summary.NxFDivAnn;
    });

    const { result } = renderHook(() => useJQuantsDividendBatch(codes, true));

    await flushAsyncUpdates();

    expect(jquantsApiClient.getStatements).toHaveBeenCalledTimes(2);
    expect(result.current.loading).toBe(false);
    expect(result.current.dividendPerShareMap.get('1605')).toBe(30);
    expect(result.current.dividendPerShareMap.has('2933')).toBe(false);

    for (let i = 0; i < 3; i += 1) {
      await act(async () => {
        vi.advanceTimersByTime(60_001);
      });
      await flushAsyncUpdates();
    }

    expect(jquantsApiClient.getStatements).toHaveBeenCalledTimes(8);
    expect(result.current.loading).toBe(false);
    expect(result.current.dividendPerShareMap.get('1605')).toBe(30);
    expect(result.current.dividendPerShareMap.has('2933')).toBe(false);

    await act(async () => {
      vi.advanceTimersByTime(60_001);
    });
    await flushAsyncUpdates();

    expect(jquantsApiClient.getStatements).toHaveBeenCalledTimes(8);
  });
});
