import { act, renderHook } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { useReceiptPageState } from '../useReceiptPageState';
import { type FilterConfig } from '@/lib/utils/searchUtils';

interface Item {
  id: number;
  code: string;
  name: string;
}

const testData: Item[] = [
  { id: 1, code: '1234', name: 'テスト商品1' },
  { id: 2, code: '5678', name: 'サンプル品' },
  { id: 3, code: '9012', name: 'テスト商品2' },
];

const filterConfig: FilterConfig<Item> = {
  stringFields: [item => item.code],
  partialStringFields: [item => item.name],
};

describe('useReceiptPageState', () => {
  it('初期状態ではsearchQueryが空文字でfilteredDataが全件になる', () => {
    const { result } = renderHook(() =>
      useReceiptPageState(testData, filterConfig)
    );

    expect(result.current.searchQuery).toBe('');
    expect(result.current.filteredData).toEqual(testData);
  });

  it('setSearchQueryでフィルタが適用される', () => {
    const { result } = renderHook(() =>
      useReceiptPageState(testData, filterConfig)
    );

    act(() => {
      result.current.setSearchQuery('テスト');
    });

    expect(result.current.searchQuery).toBe('テスト');
    expect(result.current.filteredData).toEqual([testData[0], testData[2]]);
  });

  it('searchQueryをクリアすると全件に戻る', () => {
    const { result } = renderHook(() =>
      useReceiptPageState(testData, filterConfig)
    );

    act(() => {
      result.current.setSearchQuery('テスト');
    });

    expect(result.current.filteredData).toEqual([testData[0], testData[2]]);

    act(() => {
      result.current.setSearchQuery('');
    });

    expect(result.current.searchQuery).toBe('');
    expect(result.current.filteredData).toEqual(testData);
  });

  it('データが変わるとfilteredDataも再計算される', () => {
    const nextData: Item[] = [
      ...testData,
      { id: 4, code: '3456', name: '新しいテスト商品' },
    ];

    const { result, rerender } = renderHook(
      ({ data }: { data: Item[] }) => useReceiptPageState(data, filterConfig),
      { initialProps: { data: testData } }
    );

    act(() => {
      result.current.setSearchQuery('テスト');
    });

    expect(result.current.filteredData).toEqual([testData[0], testData[2]]);

    rerender({ data: nextData });

    expect(result.current.filteredData).toEqual([
      testData[0],
      testData[2],
      nextData[3],
    ]);
  });
});
