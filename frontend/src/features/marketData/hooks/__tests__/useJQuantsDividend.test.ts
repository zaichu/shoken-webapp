import { describe, it, expect, vi, beforeEach, Mock } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { useJQuantsDividend } from '../useJQuantsDividend';
import { jquantsApiClient } from '../../api/client';
import { parseNumber } from '@/lib/utils/formatters';

vi.mock('../../api/client', () => ({
  jquantsApiClient: {
    getSummary: vi.fn(),
  },
}));

vi.mock('@/lib/utils/formatters', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/lib/utils/formatters')>();
  return {
    ...actual,
    parseNumber: vi.fn(),
  };
});

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
    expect(jquantsApiClient.getSummary).not.toHaveBeenCalled();
  });

  it('securityCodeが空の場合、配当情報を取得しない', () => {
    const { result } = renderHook(() => useJQuantsDividend('', true));

    expect(result.current.dividendPerShare).toBeUndefined();
    expect(result.current.loading).toBe(false);
    expect(result.current.error).toBeNull();
    expect(jquantsApiClient.getSummary).not.toHaveBeenCalled();
  });

  it('最新の決算データから配当情報を取得する（開示日でソート）', async () => {
    // 開示日が異なる複数のデータ（古い順）
    const mockResponse = {
      data: [
        { DiscDate: '2024-01-15', NxFDivAnn: '50.00' },
        { DiscDate: '2024-04-15', NxFDivAnn: '60.00' },
        { DiscDate: '2024-07-15', NxFDivAnn: '70.00' },
      ],
    };

    (jquantsApiClient.getSummary as Mock).mockResolvedValue(mockResponse);
    (parseNumber as Mock).mockReturnValue(70);

    const { result } = renderHook(() => useJQuantsDividend('1234', true));

    expect(result.current.loading).toBe(true);
    expect(result.current.dividendPerShare).toBeUndefined();

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(jquantsApiClient.getSummary).toHaveBeenCalledWith('1234');
    // 最新データ（2024-07-15）の70.00が取得されること
    expect(parseNumber).toHaveBeenCalledWith('70.00');
    expect(result.current.dividendPerShare).toBe(70);
    expect(result.current.error).toBeNull();
  });

  it('来期予想がない場合、今期予想を使用する', async () => {
    const mockResponse = {
      data: [
        {
          DiscDate: '2024-04-15',
          NxFDivAnn: '',
          FDivAnn: '45.00',
        },
      ],
    };

    (jquantsApiClient.getSummary as Mock).mockResolvedValue(mockResponse);
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
      data: [
        {
          DiscDate: '2024-04-15',
          NxFDivAnn: '',
          FDivAnn: '',
          DivAnn: '40.00',
        },
      ],
    };

    (jquantsApiClient.getSummary as Mock).mockResolvedValue(mockResponse);
    (parseNumber as Mock).mockReturnValue(40);

    const { result } = renderHook(() => useJQuantsDividend('1234', true));

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.dividendPerShare).toBe(40);
    expect(result.current.error).toBeNull();
  });

  it('最新データに配当情報がない場合、次のデータから取得する', async () => {
    const mockResponse = {
      data: [
        { DiscDate: '2024-01-15', NxFDivAnn: '50.00' },
        { DiscDate: '2024-04-15', NxFDivAnn: '' }, // 最新だが配当情報なし
      ],
    };

    (jquantsApiClient.getSummary as Mock).mockResolvedValue(mockResponse);
    (parseNumber as Mock).mockReturnValue(50);

    const { result } = renderHook(() => useJQuantsDividend('1234', true));

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    // 2番目に新しいデータ（2024-01-15）の50.00が取得されること
    expect(parseNumber).toHaveBeenCalledWith('50.00');
    expect(result.current.dividendPerShare).toBe(50);
    expect(result.current.error).toBeNull();
  });

  it('空の配当情報の場合、0を返す', async () => {
    const mockResponse = {
      data: [
        { DiscDate: '2024-01-15', NxFDivAnn: '' },
        { DiscDate: '2024-04-15', NxFDivAnn: '' },
      ],
    };

    (jquantsApiClient.getSummary as Mock).mockResolvedValue(mockResponse);
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

    (jquantsApiClient.getSummary as Mock).mockRejectedValue(mockError);

    const { result } = renderHook(() => useJQuantsDividend('1234', true));

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.dividendPerShare).toBeUndefined();
    expect(result.current.error).toBe('API Error');
  });

  it('非Errorオブジェクトのエラーの場合、デフォルトメッセージを設定する', async () => {
    (jquantsApiClient.getSummary as Mock).mockRejectedValue('string error');

    const { result } = renderHook(() => useJQuantsDividend('1234', true));

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.dividendPerShare).toBeUndefined();
    expect(result.current.error).toBe('配当情報の取得に失敗しました');
  });

  it('response.dataがundefinedの場合、undefinedを返す', async () => {
    const mockResponse = { data: undefined };

    (jquantsApiClient.getSummary as Mock).mockResolvedValue(mockResponse);

    const { result } = renderHook(() => useJQuantsDividend('1234', true));

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.dividendPerShare).toBeUndefined();
    expect(result.current.error).toBeNull();
  });

  it('response.dataが配列でない場合、undefinedを返す', async () => {
    const mockResponse = { data: { some: 'object' } };

    (jquantsApiClient.getSummary as Mock).mockResolvedValue(mockResponse);

    const { result } = renderHook(() => useJQuantsDividend('1234', true));

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.dividendPerShare).toBeUndefined();
    expect(result.current.error).toBeNull();
  });

  it('DiscDateがないデータを含む場合でもソートが失敗しない', async () => {
    const mockResponse = {
      data: [
        { NxFDivAnn: '40.00' }, // DiscDateなし
        { NxFDivAnn: '50.00' }, // DiscDateなし
      ],
    };

    (jquantsApiClient.getSummary as Mock).mockResolvedValue(mockResponse);
    (parseNumber as Mock).mockReturnValue(40);

    const { result } = renderHook(() => useJQuantsDividend('1234', true));

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.dividendPerShare).toBeDefined();
    expect(result.current.error).toBeNull();
  });

  it('アンマウント時にstateを更新しない', async () => {
    let resolvePromise!: (value: { data: unknown[] }) => void;
    const pendingPromise = new Promise<{ data: unknown[] }>((resolve) => {
      resolvePromise = resolve;
    });

    (jquantsApiClient.getSummary as Mock).mockReturnValue(pendingPromise);

    const { result, unmount } = renderHook(() => useJQuantsDividend('1234', true));

    expect(result.current.loading).toBe(true);

    // アンマウント（isActive = false になる）
    unmount();

    // 非同期処理を解決してもstateは更新されない
    resolvePromise({ data: [{ NxFDivAnn: '50.00' }] });

    // stateが変わらないことを確認（エラー/警告なし）
    await new Promise(resolve => setTimeout(resolve, 50));
    expect(result.current.loading).toBe(true); // unmount後は更新されない
  });

  it('securityCodeが変更された場合、再取得する', async () => {
    const mockResponse = {
      data: [{ NxFDivAnn: '50.00' }],
    };

    (jquantsApiClient.getSummary as Mock).mockResolvedValue(mockResponse);
    (parseNumber as Mock).mockReturnValue(50);

    const { result, rerender } = renderHook(
      ({ code, enabled }) => useJQuantsDividend(code, enabled),
      { initialProps: { code: '1234', enabled: true } }
    );

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    }, { timeout: 5000 });

    expect(jquantsApiClient.getSummary).toHaveBeenCalledWith('1234');
    expect(result.current.dividendPerShare).toBe(50);

    // securityCodeを変更
    rerender({ code: '5678', enabled: true });

    await waitFor(() => {
      expect(jquantsApiClient.getSummary).toHaveBeenCalledWith('5678');
    }, { timeout: 5000 });
  }, 15000);
});
