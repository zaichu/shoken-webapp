import { renderHook, act } from '@testing-library/react';
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { useHoldingsStorage } from '../useHoldingsStorage';
import { HoldingsData } from '@/lib/interfaces/holdings';

// localStorageのモック
const localStorageMock = {
  getItem: vi.fn((key: string) => {
    return localStorageMock._storage[key] || null;
  }),
  setItem: vi.fn((key: string, value: string) => {
    localStorageMock._storage[key] = value;
  }),
  removeItem: vi.fn((key: string) => {
    delete localStorageMock._storage[key];
  }),
  clear: vi.fn(() => {
    localStorageMock._storage = {};
  }),
  _storage: {} as Record<string, string>,
};

Object.defineProperty(window, 'localStorage', {
  value: localStorageMock,
});

const mockHoldingsData: HoldingsData = {
  security_code: '7203',
  security_name: 'トヨタ自動車',
  shares: 100,
  executing_shares: 0,
  average_purchase_price: 2500.0,
  total_purchase_amount: 250000,
  current_price: 2600.0,
  daily_change: 50.0,
  market_value: 260000,
  profit_loss_rate: 4.0,
};

describe('useHoldingsStorage', () => {
  beforeEach(() => {
    localStorageMock.clear();
    localStorageMock._storage = {};
    vi.clearAllMocks();
    
    // localStorageMockを再設定
    localStorageMock.setItem = vi.fn((key: string, value: string) => {
      localStorageMock._storage[key] = value;
    });
    localStorageMock.getItem = vi.fn((key: string) => {
      return localStorageMock._storage[key] || null;
    });
    localStorageMock.removeItem = vi.fn((key: string) => {
      delete localStorageMock._storage[key];
    });
  });

  it('初期状態では空の配列を返す', () => {
    const { result } = renderHook(() => useHoldingsStorage());

    expect(result.current.holdings).toEqual([]);
    expect(result.current.lastUpdated).toBeNull();
  });

  it('保有株データを保存できる', () => {
    const { result } = renderHook(() => useHoldingsStorage());
    const holdings = [mockHoldingsData];

    act(() => {
      result.current.saveHoldings(holdings);
    });

    expect(result.current.holdings).toEqual(holdings);
    expect(result.current.lastUpdated).not.toBeNull();
    expect(localStorageMock.setItem).toHaveBeenCalledWith(
      'holdings-data',
      JSON.stringify(holdings)
    );
  });

  it('保有株データをクリアできる', () => {
    const { result } = renderHook(() => useHoldingsStorage());
    const holdings = [mockHoldingsData];

    // まずデータを保存
    act(() => {
      result.current.saveHoldings(holdings);
    });

    expect(result.current.holdings).toEqual(holdings);

    // データをクリア
    act(() => {
      result.current.clearHoldings();
    });

    expect(result.current.holdings).toEqual([]);
    expect(result.current.lastUpdated).toBeNull();
  });

  it('銘柄コードで保有株を検索できる', () => {
    const { result } = renderHook(() => useHoldingsStorage());
    const holdings = [mockHoldingsData];

    act(() => {
      result.current.saveHoldings(holdings);
    });

    const foundHolding = result.current.getHoldingByCode('7203');
    expect(foundHolding).toEqual(mockHoldingsData);

    const notFoundHolding = result.current.getHoldingByCode('9999');
    expect(notFoundHolding).toBeUndefined();
  });

  it('既存の保有株データを読み込める', () => {
    const holdings = [mockHoldingsData];
    const lastUpdated = new Date().toISOString();
    
    localStorageMock.setItem('holdings-data', JSON.stringify(holdings));
    localStorageMock.setItem('holdings-last-updated', JSON.stringify(lastUpdated));

    const { result } = renderHook(() => useHoldingsStorage());

    expect(result.current.holdings).toEqual(holdings);
    expect(result.current.lastUpdated).toBe(lastUpdated);
  });

  it('複数の保有株を管理できる', () => {
    const { result } = renderHook(() => useHoldingsStorage());
    const holdings = [
      mockHoldingsData,
      {
        ...mockHoldingsData,
        security_code: '6758',
        security_name: 'ソニーグループ',
      },
    ];

    act(() => {
      result.current.saveHoldings(holdings);
    });

    expect(result.current.holdings).toHaveLength(2);
    expect(result.current.getHoldingByCode('7203')).toEqual(mockHoldingsData);
    expect(result.current.getHoldingByCode('6758')).toBeDefined();
  });
});
