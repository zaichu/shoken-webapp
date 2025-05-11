import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import React from 'react';
import { useStockSearch } from '../useStockSearch';
import { fetchStockData } from '../../api';
import { StockData } from '../../types';

vi.mock('../../api', () => ({
  fetchStockData: vi.fn(),
}));

// QueryClientProviderのラッパーを作成
const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });

  return ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client= { queryClient } >
    { children }
    </QueryClientProvider>
  );
};

describe('useStockSearch', () => {
  const mockStockData: StockData = {
    date: '2024-01-01',
    code: '1234',
    name: 'テスト株式',
    market_category: '東証プライム',
    industry_code_33: '10',
    industry_category_33: 'テクノロジー',
    industry_code_17: '01',
    industry_category_17: '情報・通信業',
    size_code: 'L',
    size_category: '大型株',
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('初期状態が正しく設定される', () => {
    const { result } = renderHook(() => useStockSearch(), {
      wrapper: createWrapper(),
    });

    expect(result.current.stockCode).toBe('');
    expect(result.current.stockData).toBeUndefined();
    expect(result.current.error).toBeNull();
    expect(result.current.isLoading).toBe(false);
    expect(result.current.isError).toBe(false);
  });

  it('setStockCodeで株式コードを設定できる', () => {
    const { result } = renderHook(() => useStockSearch(), {
      wrapper: createWrapper(),
    });

    act(() => {
      result.current.setStockCode('1234');
    });

    expect(result.current.stockCode).toBe('1234');
  });

  it('handleSearchで検索を実行できる', async () => {
    (fetchStockData as ReturnType<typeof vi.fn>).mockResolvedValue(mockStockData);

    const { result } = renderHook(() => useStockSearch(), {
      wrapper: createWrapper(),
    });

    // 株式コードを設定
    act(() => {
      result.current.setStockCode('1234');
    });

    // formイベントのモック
    const mockEvent = {
      preventDefault: vi.fn(),
    } as unknown as React.FormEvent;

    // 検索を実行
    await act(async () => {
      result.current.handleSearch(mockEvent);
    });

    expect(mockEvent.preventDefault).toHaveBeenCalled();
    expect(fetchStockData).toHaveBeenCalledWith('1234');
  });

  it('resetSearchで状態をリセットできる', () => {
    const { result } = renderHook(() => useStockSearch(), {
      wrapper: createWrapper(),
    });

    // 値を設定
    act(() => {
      result.current.setStockCode('1234');
    });

    // formイベントのモック
    const mockEvent = {
      preventDefault: vi.fn(),
    } as unknown as React.FormEvent;

    // 検索を実行
    act(() => {
      result.current.handleSearch(mockEvent);
    });

    // リセット
    act(() => {
      result.current.resetSearch();
    });

    expect(result.current.stockCode).toBe('');
    expect(result.current.stockData).toBeUndefined();
  });

  it('searchQueryが変更されたときにデータを取得する', async () => {
    (fetchStockData as ReturnType<typeof vi.fn>).mockResolvedValue(mockStockData);

    const { result } = renderHook(() => useStockSearch(), {
      wrapper: createWrapper(),
    });

    // 株式コードを設定
    act(() => {
      result.current.setStockCode('1234');
    });

    // 検索を実行
    const mockEvent = {
      preventDefault: vi.fn(),
    } as unknown as React.FormEvent;

    await act(async () => {
      result.current.handleSearch(mockEvent);
    });

    // データが取得されるまで待つ
    await act(async () => {
      await new Promise(resolve => setTimeout(resolve, 100));
    });

    expect(fetchStockData).toHaveBeenCalledWith('1234');
    expect(result.current.stockData).toEqual(mockStockData);
  });

  it('エラーが発生した場合、エラー状態を設定する', async () => {
    const mockError = new Error('API Error');
    (fetchStockData as ReturnType<typeof vi.fn>).mockRejectedValue(mockError);

    const { result } = renderHook(() => useStockSearch(), {
      wrapper: createWrapper(),
    });

    // 株式コードを設定
    act(() => {
      result.current.setStockCode('1234');
    });

    // 検索を実行
    const mockEvent = {
      preventDefault: vi.fn(),
    } as unknown as React.FormEvent;

    await act(async () => {
      result.current.handleSearch(mockEvent);
    });

    // エラーが設定されるまで待つ
    await act(async () => {
      await new Promise(resolve => setTimeout(resolve, 100));
    });

    expect(result.current.error).toEqual(mockError);
    expect(result.current.isError).toBe(true);
  });
});
