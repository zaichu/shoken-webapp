import { renderHook, act } from '@testing-library/react';
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { useAssetBalanceStorage } from '../useAssetBalanceStorage';
import { AssetBalanceData } from '@/lib/interfaces/assetBalance';

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

const mockAssetBalanceData: AssetBalanceData = {
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

const mockAssetBalanceData2: AssetBalanceData = {
  security_code: '6758',
  security_name: 'ソニーグループ',
  shares: 50,
  executing_shares: 10,
  average_purchase_price: 15000.0,
  total_purchase_amount: 750000,
  current_price: 16000.0,
  daily_change: 100.0,
  market_value: 800000,
  profit_loss_rate: 6.67,
};

describe('useAssetBalanceStorage', () => {
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
    const { result } = renderHook(() => useAssetBalanceStorage());

    expect(result.current.assetBalanceStorageData).toEqual([]);
    expect(result.current.lastUpdated).toBeNull();
  });

  it('保有株データを保存できる', () => {
    const { result } = renderHook(() => useAssetBalanceStorage());
    const assetBalance = [mockAssetBalanceData];

    act(() => {
      result.current.saveAssetBalance(assetBalance);
    });

    expect(result.current.assetBalanceStorageData).toEqual(assetBalance);
    expect(result.current.lastUpdated).not.toBeNull();

    // setItemの呼び出し回数を確認
    expect(localStorageMock.setItem).toHaveBeenCalledTimes(2);

    // 1回目の呼び出し（asset-balance-data）
    expect(localStorageMock.setItem).toHaveBeenNthCalledWith(
      1,
      'asset-balance-data',
      JSON.stringify(assetBalance)
    );

    // 2回目の呼び出し（asset-balance-last-updated）
    expect(localStorageMock.setItem).toHaveBeenNthCalledWith(
      2,
      'asset-balance-last-updated',
      expect.stringMatching(/^".+Z"$/) // JSON.stringifyされた文字列なので、クォートで囲まれる
    );
  });

  it('保有株データをクリアできる', () => {
    const { result } = renderHook(() => useAssetBalanceStorage());
    const assetBalance = [mockAssetBalanceData];

    // まずデータを保存
    act(() => {
      result.current.saveAssetBalance(assetBalance);
    });

    expect(result.current.assetBalanceStorageData).toEqual(assetBalance);

    // モックをクリアして新しい呼び出しを追跡
    vi.clearAllMocks();

    // データをクリア
    act(() => {
      result.current.clearAssetBalance();
    });

    expect(result.current.assetBalanceStorageData).toEqual([]);
    expect(result.current.lastUpdated).toBeNull();

    // setItemが2回呼ばれることを確認（空配列とnullを設定）
    expect(localStorageMock.setItem).toHaveBeenCalledTimes(2);

    // 1回目の呼び出し（asset-balance-data に空配列）
    expect(localStorageMock.setItem).toHaveBeenNthCalledWith(
      1,
      'asset-balance-data',
      JSON.stringify([])
    );

    // 2回目の呼び出し（asset-balance-last-updated に null）
    expect(localStorageMock.setItem).toHaveBeenNthCalledWith(
      2,
      'asset-balance-last-updated',
      JSON.stringify(null)
    );
  });

  it('銘柄コードで保有株を検索できる', () => {
    const { result } = renderHook(() => useAssetBalanceStorage());
    const assetBalance = [mockAssetBalanceData, mockAssetBalanceData2];

    act(() => {
      result.current.saveAssetBalance(assetBalance);
    });

    const foundAssetBalance = result.current.getAssetBalanceByCode('7203');
    expect(foundAssetBalance).toEqual(mockAssetBalanceData);

    const foundAssetBalance2 = result.current.getAssetBalanceByCode('6758');
    expect(foundAssetBalance2).toEqual(mockAssetBalanceData2);

    const notFoundAssetBalance = result.current.getAssetBalanceByCode('9999');
    expect(notFoundAssetBalance).toBeUndefined();
  });

  it('既存の保有株データを読み込める', () => {
    const assetBalance = [mockAssetBalanceData];
    const lastUpdated = new Date().toISOString();

    // モックストレージに事前にデータを設定
    localStorageMock._storage['asset-balance-data'] = JSON.stringify(assetBalance);
    localStorageMock._storage['asset-balance-last-updated'] = JSON.stringify(lastUpdated);

    const { result } = renderHook(() => useAssetBalanceStorage());

    expect(result.current.assetBalanceStorageData).toEqual(assetBalance);
    expect(result.current.lastUpdated).toBe(lastUpdated);
  });

  it('複数の保有株を管理できる', () => {
    const { result } = renderHook(() => useAssetBalanceStorage());
    const assetBalance = [mockAssetBalanceData, mockAssetBalanceData2];

    act(() => {
      result.current.saveAssetBalance(assetBalance);
    });

    expect(result.current.assetBalanceStorageData).toHaveLength(2);
    expect(result.current.getAssetBalanceByCode('7203')).toEqual(mockAssetBalanceData);
    expect(result.current.getAssetBalanceByCode('6758')).toEqual(mockAssetBalanceData2);
  });

  it('保有株データを更新できる', () => {
    const { result } = renderHook(() => useAssetBalanceStorage());
    const initialAssetBalance = [mockAssetBalanceData];
    const updatedAssetBalance = [
      {
        ...mockAssetBalanceData,
        shares: 200,
        market_value: 520000,
      },
    ];

    // 初期データを保存
    act(() => {
      result.current.saveAssetBalance(initialAssetBalance);
    });

    const firstUpdated = result.current.lastUpdated;
    expect(result.current.assetBalanceStorageData).toEqual(initialAssetBalance);

    // データを更新
    setTimeout(() => {
      act(() => {
        result.current.saveAssetBalance(updatedAssetBalance);
      });

      expect(result.current.assetBalanceStorageData).toEqual(updatedAssetBalance);
      expect(result.current.lastUpdated).not.toBe(firstUpdated);
      expect(result.current.lastUpdated).not.toBeNull();
    }, 100);
  });

  it('空の配列を保存できる', () => {
    const { result } = renderHook(() => useAssetBalanceStorage());
    const assetBalance = [mockAssetBalanceData];

    // まずデータを保存
    act(() => {
      result.current.saveAssetBalance(assetBalance);
    });

    expect(result.current.assetBalanceStorageData).toEqual(assetBalance);

    // 空の配列を保存
    act(() => {
      result.current.saveAssetBalance([]);
    });

    expect(result.current.assetBalanceStorageData).toEqual([]);
    expect(result.current.lastUpdated).not.toBeNull();
  });

  it('執行中の株式がある銘柄を正しく処理できる', () => {
    const { result } = renderHook(() => useAssetBalanceStorage());
    const assetBalanceWithExecuting = [
      {
        ...mockAssetBalanceData,
        executing_shares: 20,
      },
    ];

    act(() => {
      result.current.saveAssetBalance(assetBalanceWithExecuting);
    });

    const foundAssetBalance = result.current.getAssetBalanceByCode('7203');
    expect(foundAssetBalance?.executing_shares).toBe(20);
  });

  it('インデックスシグネチャによる動的アクセスをテスト', () => {
    const { result } = renderHook(() => useAssetBalanceStorage());
    const assetBalanceWithExtra = [
      {
        ...mockAssetBalanceData,
        custom_field: 'カスタム値',
      },
    ];

    act(() => {
      result.current.saveAssetBalance(assetBalanceWithExtra);
    });

    const foundAssetBalance = result.current.getAssetBalanceByCode('7203');
    expect(foundAssetBalance?.['custom_field']).toBe('カスタム値');
  });

  it('lastUpdatedがISO文字列形式であることを確認', () => {
    const { result } = renderHook(() => useAssetBalanceStorage());
    const assetBalance = [mockAssetBalanceData];

    act(() => {
      result.current.saveAssetBalance(assetBalance);
    });

    const lastUpdated = result.current.lastUpdated;
    expect(lastUpdated).not.toBeNull();
    if (lastUpdated) {
      const date = new Date(lastUpdated);
      expect(date.toISOString()).toBe(lastUpdated);
    }
  });
});
