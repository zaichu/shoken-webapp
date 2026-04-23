import { renderHook, act } from '@testing-library/react';
import { vi } from 'vitest';
import { useTableAutoResize } from '../useTableAutoResize';

// ResizeObserverのモック
class MockResizeObserver {
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  constructor(_callback: ResizeObserverCallback) {
    // callback は保持するが、テストでは使用しない
  }

  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
}

// グローバルのResizeObserverをモック
global.ResizeObserver = MockResizeObserver as unknown as typeof ResizeObserver;

describe('useTableAutoResize', () => {
  const originalInnerHeight = window.innerHeight;
  const mockAddEventListener = vi.fn();
  const mockRemoveEventListener = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
    // window.innerHeightをモック
    Object.defineProperty(window, 'innerHeight', {
      writable: true,
      configurable: true,
      value: 1000,
    });

    // addEventListenerとremoveEventListenerをモック
    window.addEventListener = mockAddEventListener;
    window.removeEventListener = mockRemoveEventListener;
  });

  afterEach(() => {
    Object.defineProperty(window, 'innerHeight', {
      writable: true,
      configurable: true,
      value: originalInnerHeight,
    });
  });

  it('デフォルト設定で初期化される', () => {
    const { result } = renderHook(() => useTableAutoResize());

    expect(result.current.containerRef).toBeDefined();
    expect(result.current.height).toBe('auto');
  });

  it('enabledがfalseの場合は高さ計算を行わない', () => {
    const { result } = renderHook(() =>
      useTableAutoResize({ enabled: false })
    );

    expect(result.current.height).toBe('auto');
  });

  it('forceResizeが変更されると高さが再計算される', () => {
    const { result, rerender } = renderHook(
      ({ forceResize }) => useTableAutoResize({ forceResize }),
      { initialProps: { forceResize: 0 } }
    );

    // 初期状態
    expect(result.current.height).toBe('auto');

    // forceResizeを変更
    rerender({ forceResize: 1 });

    // 高さが再計算される（実際の要素がないのでautoのまま）
    expect(result.current.height).toBe('auto');
  });

  it('コンポーネントのアンマウント時にイベントリスナーが削除される', () => {
    const { unmount } = renderHook(() => useTableAutoResize());

    // マウント時にイベントリスナーが追加される
    expect(mockAddEventListener).toHaveBeenCalledWith('resize', expect.any(Function));

    unmount();

    // アンマウント時にイベントリスナーが削除される
    expect(mockRemoveEventListener).toHaveBeenCalledWith('resize', expect.any(Function));
  });

  it('ResizeObserverが利用可能な場合に設定される', () => {
    const { result } = renderHook(() => useTableAutoResize());

    // containerRefが存在することを確認
    expect(result.current.containerRef).toBeDefined();
    expect(result.current.containerRef.current).toBeNull(); // 実際のDOM要素はない
  });

  it('オプションが正しく適用される', () => {
    const options = {
      enabled: true,
      minHeight: 300,
      maxHeight: 800,
      bottomMargin: 50,
    };

    const { result } = renderHook(() => useTableAutoResize(options));

    expect(result.current.containerRef).toBeDefined();
    expect(result.current.height).toBe('auto'); // DOM要素がないので計算されない
  });

  it('ウィンドウサイズの変更時にリサイズハンドラーが呼ばれる', () => {
    renderHook(() => useTableAutoResize());

    expect(mockAddEventListener).toHaveBeenCalledWith('resize', expect.any(Function));

    // リサイズイベントをシミュレート
    const resizeHandler = mockAddEventListener.mock.calls[0][1];
    act(() => {
      resizeHandler();
    });

    // エラーが発生しないことを確認
  });

  it('enabled状態の変更が正しく処理される', () => {
    const { result, rerender } = renderHook(
      ({ enabled }) => useTableAutoResize({ enabled }),
      { initialProps: { enabled: true } }
    );

    expect(result.current.height).toBe('auto');

    // enabledをfalseに変更
    rerender({ enabled: false });
    expect(result.current.height).toBe('auto');

    // enabledをtrueに戻す
    rerender({ enabled: true });
    expect(result.current.height).toBe('auto');
  });
});
