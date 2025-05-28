import { renderHook, act } from '@testing-library/react';
import { vi } from 'vitest';
import { useLocalStorage, useLocalStorageKey } from '../useLocalStorage';

// localStorageのモック
const mockLocalStorage = (() => {
  let store: Record<string, string> = {};

  return {
    getItem: vi.fn((key: string) => store[key] || null),
    setItem: vi.fn((key: string, value: string) => {
      store[key] = value;
    }),
    removeItem: vi.fn((key: string) => {
      delete store[key];
    }),
    clear: vi.fn(() => {
      store = {};
    }),
    length: 0,
    key: vi.fn()
  };
})();

Object.defineProperty(window, 'localStorage', {
  value: mockLocalStorage,
});

describe('useLocalStorage', () => {
  beforeEach(() => {
    mockLocalStorage.clear();
    vi.clearAllMocks();
  });

  it('初期値で正しく初期化される', () => {
    const { result } = renderHook(() => useLocalStorage('test-key', 'default-value'));
    
    expect(result.current[0]).toBe('default-value');
  });

  it('既存の値がある場合は正しく読み込まれる', () => {
    mockLocalStorage.setItem('existing-key', JSON.stringify('existing-value'));
    
    const { result } = renderHook(() => useLocalStorage('existing-key', 'default-value'));
    
    expect(result.current[0]).toBe('existing-value');
  });

  it('値を正しく設定できる', () => {
    const { result } = renderHook(() => useLocalStorage('test-key', 'initial'));
    
    act(() => {
      result.current[1]('new-value');
    });
    
    expect(result.current[0]).toBe('new-value');
    expect(mockLocalStorage.setItem).toHaveBeenCalledWith('test-key', JSON.stringify('new-value'));
  });

  it('関数形式で値を更新できる', () => {
    const { result } = renderHook(() => useLocalStorage('test-key', 0));
    
    act(() => {
      result.current[1](prev => prev + 1);
    });
    
    expect(result.current[0]).toBe(1);
    expect(mockLocalStorage.setItem).toHaveBeenCalledWith('test-key', JSON.stringify(1));
  });

  it('値を正しく削除できる', () => {
    const { result } = renderHook(() => useLocalStorage('test-key', 'default'));
    
    // 値を設定
    act(() => {
      result.current[1]('some-value');
    });
    
    // 値を削除
    act(() => {
      result.current[2]();
    });
    
    expect(result.current[0]).toBe('default');
    expect(mockLocalStorage.removeItem).toHaveBeenCalledWith('test-key');
  });

  it('オブジェクトも正しく処理できる', () => {
    const initialObject = { name: 'test', count: 0 };
    const { result } = renderHook(() => useLocalStorage('object-key', initialObject));
    
    const newObject = { name: 'updated', count: 1 };
    
    act(() => {
      result.current[1](newObject);
    });
    
    expect(result.current[0]).toEqual(newObject);
    expect(mockLocalStorage.setItem).toHaveBeenCalledWith('object-key', JSON.stringify(newObject));
  });

  it('JSONパースエラーが発生した場合は初期値を使用する', () => {
    mockLocalStorage.setItem('broken-key', 'invalid-json');
    
    const { result } = renderHook(() => useLocalStorage('broken-key', 'fallback'));
    
    expect(result.current[0]).toBe('fallback');
  });

  it('localStorageエラーが発生してもクラッシュしない', () => {
    const consoleSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});
    
    // setItemでエラーを発生させる
    mockLocalStorage.setItem.mockImplementationOnce(() => {
      throw new Error('Storage quota exceeded');
    });
    
    const { result } = renderHook(() => useLocalStorage('error-key', 'default'));
    
    act(() => {
      result.current[1]('new-value');
    });
    
    // エラーが警告として出力されることを確認
    expect(consoleSpy).toHaveBeenCalled();
    expect(result.current[0]).toBe('new-value'); // 状態は更新される
    
    consoleSpy.mockRestore();
  });

  it('配列も正しく処理できる', () => {
    const initialArray = [1, 2, 3];
    const { result } = renderHook(() => useLocalStorage('array-key', initialArray));
    
    const newArray = [4, 5, 6];
    
    act(() => {
      result.current[1](newArray);
    });
    
    expect(result.current[0]).toEqual(newArray);
    expect(mockLocalStorage.setItem).toHaveBeenCalledWith('array-key', JSON.stringify(newArray));
  });

  it('nullやundefinedを適切に処理する', () => {
    const { result } = renderHook(() => useLocalStorage<string | null>('null-key', null));
    
    act(() => {
      result.current[1]('not-null');
    });
    
    expect(result.current[0]).toBe('not-null');
    
    act(() => {
      result.current[1](null);
    });
    
    expect(result.current[0]).toBe(null);
    expect(mockLocalStorage.setItem).toHaveBeenCalledWith('null-key', JSON.stringify(null));
  });

  it('ブール値も正しく処理できる', () => {
    const { result } = renderHook(() => useLocalStorage('boolean-key', false));
    
    act(() => {
      result.current[1](true);
    });
    
    expect(result.current[0]).toBe(true);
    expect(mockLocalStorage.setItem).toHaveBeenCalledWith('boolean-key', JSON.stringify(true));
  });

  it('数値も正しく処理できる', () => {
    const { result } = renderHook(() => useLocalStorage('number-key', 0));
    
    act(() => {
      result.current[1](42);
    });
    
    expect(result.current[0]).toBe(42);
    expect(mockLocalStorage.setItem).toHaveBeenCalledWith('number-key', JSON.stringify(42));
  });
});

describe('useLocalStorageKey', () => {
  beforeEach(() => {
    mockLocalStorage.clear();
    vi.clearAllMocks();
  });

  it('キーが存在しない場合はfalseを返す', () => {
    const { result } = renderHook(() => useLocalStorageKey('non-existent-key'));
    
    expect(result.current).toBe(false);
  });

  it('キーが存在する場合はtrueを返す', () => {
    mockLocalStorage.setItem('existing-key', 'value');
    
    const { result } = renderHook(() => useLocalStorageKey('existing-key'));
    
    expect(result.current).toBe(true);
  });

  it('キーの変更を正しく追跡する', () => {
    const { result, rerender } = renderHook(
      ({ key }) => useLocalStorageKey(key),
      { initialProps: { key: 'key1' } }
    );
    
    expect(result.current).toBe(false);
    
    // key1に値を設定
    mockLocalStorage.setItem('key1', 'value1');
    rerender({ key: 'key1' });
    
    expect(result.current).toBe(true);
    
    // 異なるキーに変更
    rerender({ key: 'key2' });
    
    expect(result.current).toBe(false);
  });
});
