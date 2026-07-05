import { beforeEach, describe, expect, it, vi } from 'vitest';
import { apiClient } from '@/lib/api/client';
import { fetchDividendPerShareBatch } from '../dividendPerShareApi';

vi.mock('@/lib/api/client', () => ({
  apiClient: {
    post: vi.fn(),
  },
}));

describe('fetchDividendPerShareBatch', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('security_codes を POST して items を返す', async () => {
    const mockItems = [
      {
        security_code: '7203',
        dividend_per_share: 100,
        status: 'ok' as const,
        fetched_at: '2024-01-01',
        is_stale: false,
      },
    ];
    (apiClient.post as ReturnType<typeof vi.fn>).mockResolvedValue({ items: mockItems });

    const result = await fetchDividendPerShareBatch(['7203']);

    expect(apiClient.post).toHaveBeenCalledWith('/api/v1/dividend-per-share-estimates', {
      security_codes: ['7203'],
    }, { withCredentials: true });
    expect(result).toEqual(mockItems);
  });

  it('空配列でも POST して空配列を返す', async () => {
    (apiClient.post as ReturnType<typeof vi.fn>).mockResolvedValue({ items: [] });

    const result = await fetchDividendPerShareBatch([]);

    expect(apiClient.post).toHaveBeenCalledWith('/api/v1/dividend-per-share-estimates', {
      security_codes: [],
    }, { withCredentials: true });
    expect(result).toEqual([]);
  });

  it('複数銘柄コードを正しいペイロードで送信する', async () => {
    const securityCodes = ['7203', '6758'];
    const mockItems = securityCodes.map((securityCode, index) => ({
      security_code: securityCode,
      dividend_per_share: 100 + index,
      status: 'ok' as const,
      fetched_at: '2024-01-01',
      is_stale: false,
    }));
    (apiClient.post as ReturnType<typeof vi.fn>).mockResolvedValue({ items: mockItems });

    const result = await fetchDividendPerShareBatch(securityCodes);

    expect(apiClient.post).toHaveBeenCalledWith('/api/v1/dividend-per-share-estimates', {
      security_codes: securityCodes,
    }, { withCredentials: true });
    expect(result).toEqual(mockItems);
  });
});
