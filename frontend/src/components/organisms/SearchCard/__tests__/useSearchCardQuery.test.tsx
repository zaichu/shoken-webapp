import { act, renderHook } from '@testing-library/react';
import { beforeEach, describe, expect, test, vi } from 'vitest';
import { useSearchCardQuery } from '../useSearchCardQuery';
import type { SearchCategories } from '@/types/common';

const categories: SearchCategories = {
  securities: [
    { value: '9433', label: 'KDDI' },
    { value: '9434', label: 'ソフトバンク' },
  ],
  products: ['国内株式', '投資信託'],
  accounts: ['特定・一般', 'NISA'],
  years: [
    { value: '2025', label: '2025年' },
    { value: '2026', label: '2026年' },
  ],
  dates: true,
};

describe('useSearchCardQuery', () => {
  const onSearch = vi.fn();
  const onCloseYearPicker = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  test('valueが空のまま再レンダーされても不安定なonCloseYearPicker参照でリセットeffectを繰り返さない', () => {
    const { rerender } = renderHook(
      ({ value }: { value: string }) =>
        useSearchCardQuery({
          categories,
          value,
          onSearch,
          onCloseYearPicker: () => onCloseYearPicker(),
        }),
      { initialProps: { value: '' } },
    );

    expect(onCloseYearPicker).toHaveBeenCalledTimes(1);

    rerender({ value: '' });
    rerender({ value: '' });

    expect(onCloseYearPicker).toHaveBeenCalledTimes(1);
    expect(onSearch).not.toHaveBeenCalled();
  });

  test('検索条件をAND条件のクエリ文字列として追加する', () => {
    const { result } = renderHook(() =>
      useSearchCardQuery({ categories, value: undefined, onSearch, onCloseYearPicker }),
    );

    act(() => result.current.handleQuickSearch('9433', 'securities'));
    expect(onSearch).toHaveBeenLastCalledWith('9433');

    act(() => result.current.handleQuickSearch('国内株式', 'products'));
    expect(onSearch).toHaveBeenLastCalledWith('9433 国内株式');

    act(() => result.current.handleQuickSearch('特定・一般', 'accounts'));
    expect(onSearch).toHaveBeenLastCalledWith('9433 国内株式 特定・一般');
  });

  test('商品と口座は同じ値を再選択すると検索条件から解除される', () => {
    const { result } = renderHook(() =>
      useSearchCardQuery({ categories, value: undefined, onSearch, onCloseYearPicker }),
    );

    act(() => result.current.handleQuickSearch('国内株式', 'products'));
    expect(onSearch).toHaveBeenLastCalledWith('国内株式');

    act(() => result.current.handleQuickSearch('国内株式', 'products'));
    expect(onSearch).toHaveBeenLastCalledWith('');
  });

  test('範囲日付の入力をdate条件へ反映し、クリアで初期状態に戻す', () => {
    const { result } = renderHook(() =>
      useSearchCardQuery({ categories, value: undefined, onSearch, onCloseYearPicker }),
    );

    act(() => result.current.handleSegmentChange('範囲'));
    act(() => result.current.handleRangeStartChange('2026-06-01'));
    expect(onSearch).toHaveBeenLastCalledWith('2026-06-01..');

    act(() => result.current.handleRangeEndChange('2026-06-30'));
    expect(onSearch).toHaveBeenLastCalledWith('2026-06-01..2026-06-30');

    act(() => result.current.handleClearSearch());
    expect(onSearch).toHaveBeenLastCalledWith('');
    expect(result.current.dateInputs).toEqual({
      yearValue: '',
      monthValue: '',
      dateValue: '',
      rangeStart: '',
      rangeEnd: '',
    });
  });
});
