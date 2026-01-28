import { describe, it, expect, vi, beforeEach, Mock } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { useJQuantsDividend } from '../useJQuantsDividend';
import { jquantsApiClient } from '../../api/client';
import { parseNumber } from '@/lib/utils/formatters';

vi.mock('../../api/client', () => ({
  jquantsApiClient: {
    getStatements: vi.fn(),
  },
}));

vi.mock('@/lib/utils/formatters', () => ({
  parseNumber: vi.fn(),
}));

describe('useJQuantsDividend', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    (parseNumber as Mock).mockImplementation((val) => Number(val) || 0);
  });

  it('enabledがfalseの場合、配当情報を取得しない', () => {
    const { result } = renderHook(() => useJQuantsDividend('1234', false));

    expect(result.current.dividendPerShare).toBeUndefined();
    expect(result.current.loading).toBe(false);
    expect(result.current.error).toBeNull();
    expect(jquantsApiClient.getStatements).not.toHaveBeenCalled();
  });

  it('securityCodeが空の場合、配当情報を取得しない', () => {
    const { result } = renderHook(() => useJQuantsDividend('', true));

    expect(result.current.dividendPerShare).toBeUndefined();
    expect(result.current.loading).toBe(false);
    expect(result.current.error).toBeNull();
    expect(jquantsApiClient.getStatements).not.toHaveBeenCalled();
  });

  it('配当情報を正常に取得できる（来期予想）', async () => {
    // V2 API では fin_summary フィールドを使用
    const mockResponse = {
      fin_summary: [
        { NextYearForecastDividendPerShareAnnual: '' },
        { NextYearForecastDividendPerShareAnnual: '50.00' },
        { NextYearForecastDividendPerShareAnnual: '60.00' },
      ],
    };

    (jquantsApiClient.getStatements as Mock).mockResolvedValue(mockResponse);
    (parseNumber as Mock).mockReturnValue(60);

    const { result } = renderHook(() => useJQuantsDividend('1234', true));

    expect(result.current.loading).toBe(true);
    expect(result.current.dividendPerShare).toBeUndefined();

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(jquantsApiClient.getStatements).toHaveBeenCalledWith('1234');
    expect(result.current.dividendPerShare).toBe(60);
    expect(result.current.error).toBeNull();
  });

  it('来期予想がない場合、今期予想を使用する', async () => {
    const mockResponse = {
      fin_summary: [
        {
          NextYearForecastDividendPerShareAnnual: '',
          ForecastDividendPerShareAnnual: '45.00',
        },
      ],
    };

    (jquantsApiClient.getStatements as Mock).mockResolvedValue(mockResponse);
    (parseNumber as Mock).mockReturnValue(45);

    const { result } = renderHook(() => useJQuantsDividend('1234', true));

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.dividendPerShare).toBe(45);
    expect(result.current.error).toBeNull();
  });

  it('予想がない場合、実績を使用する', async () => {
    const mockResponse = {
      fin_summary: [
        {
          NextYearForecastDividendPerShareAnnual: '',
          ForecastDividendPerShareAnnual: '',
          ResultDividendPerShareAnnual: '40.00',
        },
      ],
    };

    (jquantsApiClient.getStatements as Mock).mockResolvedValue(mockResponse);
    (parseNumber as Mock).mockReturnValue(40);

    const { result } = renderHook(() => useJQuantsDividend('1234', true));

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.dividendPerShare).toBe(40);
    expect(result.current.error).toBeNull();
  });

  it('空の配当情報の場合、0を返す', async () => {
    const mockResponse = {
      fin_summary: [
        { NextYearForecastDividendPerShareAnnual: '' },
        { NextYearForecastDividendPerShareAnnual: '' },
      ],
    };

    (jquantsApiClient.getStatements as Mock).mockResolvedValue(mockResponse);
    (parseNumber as Mock).mockReturnValue(0);

    const { result } = renderHook(() => useJQuantsDividend('1234', true));

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.dividendPerShare).toBe(0);
    expect(result.current.error).toBeNull();
  });

  it('エラーが発生した場合、エラーメッセージを設定する', async () => {
    const mockError = new Error('API Error');

    (jquantsApiClient.getStatements as Mock).mockRejectedValue(mockError);

    const { result } = renderHook(() => useJQuantsDividend('1234', true));

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.dividendPerShare).toBeUndefined();
    expect(result.current.error).toBe('API Error');
  });

  it('非Errorオブジェクトのエラーの場合、デフォルトメッセージを設定する', async () => {
    (jquantsApiClient.getStatements as Mock).mockRejectedValue('string error');

    const { result } = renderHook(() => useJQuantsDividend('1234', true));

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.dividendPerShare).toBeUndefined();
    expect(result.current.error).toBe('配当情報の取得に失敗しました');
  });

  it('securityCodeが変更された場合、再取得する', async () => {
    const mockResponse = {
      fin_summary: [{ NextYearForecastDividendPerShareAnnual: '50.00' }],
    };

    (jquantsApiClient.getStatements as Mock).mockResolvedValue(mockResponse);
    (parseNumber as Mock).mockReturnValue(50);

    const { result, rerender } = renderHook(
      ({ code, enabled }) => useJQuantsDividend(code, enabled),
      { initialProps: { code: '1234', enabled: true } }
    );

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(jquantsApiClient.getStatements).toHaveBeenCalledWith('1234');
    expect(result.current.dividendPerShare).toBe(50);

    // securityCodeを変更
    rerender({ code: '5678', enabled: true });

    await waitFor(() => {
      expect(jquantsApiClient.getStatements).toHaveBeenCalledWith('5678');
    });
  });
});
