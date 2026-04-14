import { act, renderHook } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { useIdleTimer } from '../useIdleTimer';

describe('useIdleTimer', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    act(() => {
      vi.runOnlyPendingTimers();
    });
    vi.useRealTimers();
    vi.restoreAllMocks();
  });

  it('enabled=true のとき timeout 経過後に onIdle を呼ぶ', () => {
    const onIdle = vi.fn();

    renderHook(() => useIdleTimer({ timeout: 1000, onIdle, enabled: true }));

    act(() => {
      vi.advanceTimersByTime(999);
    });
    expect(onIdle).not.toHaveBeenCalled();

    act(() => {
      vi.advanceTimersByTime(1);
    });
    expect(onIdle).toHaveBeenCalledTimes(1);
  });

  it('enabled=false のとき onIdle を呼ばない', () => {
    const onIdle = vi.fn();

    renderHook(() => useIdleTimer({ timeout: 1000, onIdle, enabled: false }));

    act(() => {
      vi.advanceTimersByTime(1000);
    });
    expect(onIdle).not.toHaveBeenCalled();
  });

  it('mousedown でタイマーがリセットされ、その後に再度 timeout 経過で onIdle を呼ぶ', () => {
    const onIdle = vi.fn();

    renderHook(() => useIdleTimer({ timeout: 1000, onIdle }));

    act(() => {
      vi.advanceTimersByTime(800);
    });
    expect(onIdle).not.toHaveBeenCalled();

    act(() => {
      document.dispatchEvent(new MouseEvent('mousedown'));
    });

    act(() => {
      vi.advanceTimersByTime(800);
    });
    expect(onIdle).not.toHaveBeenCalled();

    act(() => {
      vi.advanceTimersByTime(200);
    });
    expect(onIdle).toHaveBeenCalledTimes(1);
  });

  it('アンマウント時にイベントリスナーを解除する', () => {
    const addEventListenerSpy = vi.spyOn(document, 'addEventListener');
    const removeEventListenerSpy = vi.spyOn(document, 'removeEventListener');
    const onIdle = vi.fn();
    const expectedEvents = [
      'mousedown',
      'mousemove',
      'keydown',
      'scroll',
      'touchstart',
      'click',
    ] as const;

    const { unmount } = renderHook(() => useIdleTimer({ timeout: 1000, onIdle }));

    expectedEvents.forEach((eventName) => {
      expect(addEventListenerSpy).toHaveBeenCalledWith(
        eventName,
        expect.any(Function),
        { passive: true }
      );
    });

    const handlers = new Map(
      addEventListenerSpy.mock.calls
        .filter(([eventName]) => expectedEvents.includes(eventName as (typeof expectedEvents)[number]))
        .map(([eventName, handler]) => [eventName, handler])
    );

    unmount();

    expectedEvents.forEach((eventName) => {
      expect(removeEventListenerSpy).toHaveBeenCalledWith(eventName, handlers.get(eventName));
    });
  });
});
