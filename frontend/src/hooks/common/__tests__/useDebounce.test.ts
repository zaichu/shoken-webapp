import { renderHook, act } from '@testing-library/react';
import { describe, it, expect, afterEach, vi } from 'vitest';
import { useDebounce, useSimpleDebounce } from '../useDebounce';

// タイマーをモック化
vi.useFakeTimers();

describe('useDebounce', () => {
  afterEach(() => {
    vi.clearAllTimers();
  });

  it('基本的なデバウンス動作をする', () => {
    const { result, rerender } = renderHook(
      ({ value, delay }) => useDebounce(value, delay),
      { initialProps: { value: 'initial', delay: 500 } }
    );

    expect(result.current).toBe('initial');

    // 値を更新
    rerender({ value: 'updated', delay: 500 });
    expect(result.current).toBe('initial'); // まだ更新されない

    // 時間を進める（デバウンス時間未満）
    act(() => {
      vi.advanceTimersByTime(300);
    });
    expect(result.current).toBe('initial'); // まだ更新されない

    // デバウンス時間を経過
    act(() => {
      vi.advanceTimersByTime(200);
    });
    expect(result.current).toBe('updated'); // 更新される
  });

  it('複数回の更新をデバウンスする', () => {
    const { result, rerender } = renderHook(
      ({ value, delay }) => useDebounce(value, delay),
      { initialProps: { value: 'initial', delay: 500 } }
    );

    // 複数回連続で更新
    rerender({ value: 'first', delay: 500 });
    rerender({ value: 'second', delay: 500 });
    rerender({ value: 'third', delay: 500 });

    expect(result.current).toBe('initial'); // まだ更新されない

    // デバウンス時間を経過
    act(() => {
      vi.advanceTimersByTime(500);
    });

    expect(result.current).toBe('third'); // 最後の値で更新される
  });

  it('leadingオプションが動作する', () => {
    const { result, rerender } = renderHook(
      ({ value, delay }) => useDebounce(value, delay, { leading: true }),
      { initialProps: { value: 'initial', delay: 500 } }
    );

    expect(result.current).toBe('initial');

    // 値を更新（leading: trueなので即座に更新される）
    rerender({ value: 'updated', delay: 500 });
    expect(result.current).toBe('updated');
  });

  it('trailingオプションがfalseの場合は最後の更新をしない', () => {
    const { result, rerender } = renderHook(
      ({ value, delay }) => useDebounce(value, delay, { trailing: false }),
      { initialProps: { value: 'initial', delay: 500 } }
    );

    rerender({ value: 'updated', delay: 500 });
    
    act(() => {
      vi.advanceTimersByTime(500);
    });

    expect(result.current).toBe('initial'); // trailingがfalseなので更新されない
  });

  it('maxWaitオプションが動作する', () => {
    const { result, rerender } = renderHook(
      ({ value, delay }) => useDebounce(value, delay, { maxWait: 300 }),
      { initialProps: { value: 'initial', delay: 1000 } }
    );

    rerender({ value: 'first', delay: 1000 });
    
    // maxWait時間を経過
    act(() => {
      vi.advanceTimersByTime(300);
    });

    expect(result.current).toBe('first'); // maxWaitで強制的に更新される
  });

  it('異なる型の値でも動作する', () => {
    const { result, rerender } = renderHook(
      ({ value, delay }) => useDebounce(value, delay),
      { initialProps: { value: 0, delay: 500 } }
    );

    expect(result.current).toBe(0);

    rerender({ value: 42, delay: 500 });
    
    act(() => {
      vi.advanceTimersByTime(500);
    });

    expect(result.current).toBe(42);
  });

  it('オブジェクト値でも動作する', () => {
    const initialValue = { name: 'John', age: 30 };
    const updatedValue = { name: 'Jane', age: 25 };

    const { result, rerender } = renderHook(
      ({ value, delay }) => useDebounce(value, delay),
      { initialProps: { value: initialValue, delay: 500 } }
    );

    expect(result.current).toBe(initialValue);

    rerender({ value: updatedValue, delay: 500 });
    
    act(() => {
      vi.advanceTimersByTime(500);
    });

    expect(result.current).toBe(updatedValue);
  });

  it('delayが0の場合は即座に更新される', () => {
    const { result, rerender } = renderHook(
      ({ value, delay }) => useDebounce(value, delay),
      { initialProps: { value: 'initial', delay: 0 } }
    );

    rerender({ value: 'updated', delay: 0 });
    
    act(() => {
      vi.advanceTimersByTime(0);
    });

    expect(result.current).toBe('updated');
  });
});

describe('useSimpleDebounce', () => {
  afterEach(() => {
    vi.clearAllTimers();
  });

  it('シンプルなデバウンス動作をする', () => {
    const { result, rerender } = renderHook(
      ({ value, delay }) => useSimpleDebounce(value, delay),
      { initialProps: { value: 'initial', delay: 500 } }
    );

    expect(result.current).toBe('initial');

    rerender({ value: 'updated', delay: 500 });
    expect(result.current).toBe('initial');

    act(() => {
      vi.advanceTimersByTime(500);
    });

    expect(result.current).toBe('updated');
  });

  it('useDebounceのデフォルト設定と同じ動作をする', () => {
    const { result: simpleResult, rerender: simpleRerender } = renderHook(
      ({ value, delay }) => useSimpleDebounce(value, delay),
      { initialProps: { value: 'test', delay: 300 } }
    );

    const { result: normalResult, rerender: normalRerender } = renderHook(
      ({ value, delay }) => useDebounce(value, delay),
      { initialProps: { value: 'test', delay: 300 } }
    );

    // 両方とも同じ初期値
    expect(simpleResult.current).toBe('test');
    expect(normalResult.current).toBe('test');

    // 両方とも更新
    simpleRerender({ value: 'updated', delay: 300 });
    normalRerender({ value: 'updated', delay: 300 });

    // 時間経過前は両方とも更新されない
    expect(simpleResult.current).toBe('test');
    expect(normalResult.current).toBe('test');

    // 時間経過後は両方とも更新される
    act(() => {
      vi.advanceTimersByTime(300);
    });

    expect(simpleResult.current).toBe('updated');
    expect(normalResult.current).toBe('updated');
  });
});
