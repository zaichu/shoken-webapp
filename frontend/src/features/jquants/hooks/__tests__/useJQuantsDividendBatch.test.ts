import { describe, it, expect, vi, beforeEach, Mock } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { useJQuantsDividendBatch } from '../useJQuantsDividendBatch';
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

describe('useJQuantsDividendBatch', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    (parseNumber as Mock).mockImplementation((val) => Number(val) || 0);
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
});
