import { renderHook } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { useReceiptBaseData, useReceiptCalculations } from '../useReceiptData';
import { type FilterConfig } from '@/lib/utils/searchUtils';

type Item = { id: number; name: string };

const identity = (items: Item[]) => items;
const filterConfig: FilterConfig<Item> = {
  fields: ['name'],
};

describe('useReceiptBaseData', () => {
  it('previewDataがある場合はpreviewDataを優先する', () => {
    const data: Item[] = [{ id: 1, name: 'メインデータ' }];
    const previewData: Item[] = [{ id: 2, name: 'プレビューデータ' }];

    const { result } = renderHook(() =>
      useReceiptBaseData(data, previewData, identity, filterConfig)
    );

    expect(result.current.sortedData).toEqual(previewData);
  });

  it('previewDataが空配列の場合はdataを使う', () => {
    const data: Item[] = [{ id: 1, name: 'メインデータ' }];

    const { result } = renderHook(() =>
      useReceiptBaseData(data, [], identity, filterConfig)
    );

    expect(result.current.sortedData).toEqual(data);
  });

  it('previewDataがundefinedの場合はdataを使う', () => {
    const data: Item[] = [{ id: 1, name: 'メインデータ' }];

    const { result } = renderHook(() =>
      useReceiptBaseData(data, undefined, identity, filterConfig)
    );

    expect(result.current.sortedData).toEqual(data);
  });
});

describe('useReceiptCalculations', () => {
  it('calculateFunctionの結果を返す', () => {
    const data: Item[] = [{ id: 1, name: 'A' }, { id: 2, name: 'B' }];

    const { result } = renderHook(() =>
      useReceiptCalculations(data, (items) => items.length)
    );

    expect(result.current).toBe(2);
  });
});
