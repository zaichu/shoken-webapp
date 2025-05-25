import { renderHook, act } from '@testing-library/react';
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { useLocalStorage, useLocalStorageKey, useMultipleLocalStorage } from '../useLocalStorage';

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

// storageイベントリスナーのモック
const eventListeners: { [key: string]: ((event: StorageEvent) => void)[] } = {};

Object.defineProperty(window, 'addEventListener', {
  value: vi.fn((type: string, listener: (event: StorageEvent) => void) => {
    if (!eventListeners[type]) {
      eventListeners[type] = [];
    }
    eventListeners[type].push(listener);
  }),
});

Object.defineProperty(window, 'removeEventListener', {
  value: vi.fn((type: string, listener: (event: StorageEvent) => void) => {
    if (eventListeners[type]) {
      const index = eventListeners[type].indexOf(listener);
      if (index > -1) {
        eventListeners[type].splice(index, 1);
      }
    }
  }),
});

Object.defineProperty(window, 'dispatchEvent', {
  value: vi.fn((event: StorageEvent) => {
    const type = event.type;
    if (eventListeners[type]) {
      eventListeners[type].forEach(listener => listener(event));
    }
    return true;
  }),
});

describe('useLocalStorage', () => {
  beforeEach(() => {
    localStorageMock.clear();
    localStorageMock._storage = {};
    vi.clearAllMocks();
    Object.keys(eventListeners).forEach(key => {
      eventListeners[key] = [];
    });

    // localStorageMockをリセット
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

  it('初期値を正しく設定する', () => {
    const { result } = renderHook(() => useLocalStorage('test-key', 'initial-value'));

    expect(result.current[0]).toBe('initial-value');
  });

  it('既存のローカルストレージの値を読み込む', () => {
    localStorageMock.setItem('existing-key', JSON.stringify('existing-value'));

    const { result } = renderHook(() => useLocalStorage('existing-key', 'initial-value'));

    expect(result.current[0]).toBe('existing-value');
  });

  it('値を設定できる', () => {
    const { result } = renderHook(() => useLocalStorage('test-key', 'initial'));

    act(() => {
      result.current[1]('new-value');
    });

    expect(result.current[0]).toBe('new-value');
    expect(localStorageMock.setItem).toHaveBeenCalledWith('test-key', JSON.stringify('new-value'));
  });

  it('関数で値を設定できる', () => {
    const { result } = renderHook(() => useLocalStorage('test-key', 'initial'));

    act(() => {
      result.current[1]((prev) => prev + '-updated');
    });

    expect(result.current[0]).toBe('initial-updated');
  });

  it('値を削除できる', () => {
    const { result } = renderHook(() => useLocalStorage('test-key', 'initial'));

    act(() => {
      result.current[1]('some-value');
    });

    expect(result.current[0]).toBe('some-value');

    act(() => {
      result.current[2](); // removeValue
    });

    expect(result.current[0]).toBe('initial');
    expect(localStorageMock.removeItem).toHaveBeenCalledWith('test-key');
  });

  it('オブジェクトの値を扱える', () => {
    const initialObject = { name: 'John', age: 30 };
    const { result } = renderHook(() => useLocalStorage('object-key', initialObject));

    const newObject = { name: 'Jane', age: 25 };
    act(() => {
      result.current[1](newObject);
    });

    expect(result.current[0]).toEqual(newObject);
    expect(localStorageMock.setItem).toHaveBeenCalledWith('object-key', JSON.stringify(newObject));
  });

  it('配列の値を扱える', () => {
    const initialArray = [1, 2, 3];
    const { result } = renderHook(() => useLocalStorage('array-key', initialArray));

    const newArray = [4, 5, 6];
    act(() => {
      result.current[1](newArray);
    });

    expect(result.current[0]).toEqual(newArray);
  });

  it('無効なJSONを処理する', () => {
    localStorageMock.setItem('invalid-json', 'invalid json string');

    const { result } = renderHook(() => useLocalStorage('invalid-json', 'fallback'));

    expect(result.current[0]).toBe('fallback');
  });

  it('カスタムシリアライザーを使用できる', () => {
    const customSerializer = {
      serialize: (value: string) => `custom:${value}`,
      deserialize: (text: string) => text.replace('custom:', ''),
    };

    const { result } = renderHook(() =>
      useLocalStorage('custom-key', 'initial', { serializer: customSerializer })
    );

    act(() => {
      result.current[1]('test-value');
    });

    expect(localStorageMock.setItem).toHaveBeenCalledWith('custom-key', 'custom:test-value');
  });

  it('タブ間同期が動作する', () => {
    const { result } = renderHook(() => useLocalStorage('sync-key', 'initial'));

    // 別タブからのstorageイベントをシミュレート
    const storageEvent = new StorageEvent('storage', {
      key: 'sync-key',
      newValue: JSON.stringify('synced-value'),
    });

    act(() => {
      window.dispatchEvent(storageEvent);
    });

    expect(result.current[0]).toBe('synced-value');
  });

  it('カスタムイベントでの同期が動作する', () => {
    const { result } = renderHook(() => useLocalStorage('custom-sync-key', 'initial'));

    // カスタムイベントをシミュレート
    const customEvent = new CustomEvent('local-storage', {
      detail: { key: 'custom-sync-key', newValue: 'custom-synced-value' }
    });

    act(() => {
      window.dispatchEvent(customEvent);
    });

    expect(result.current[0]).toBe('custom-synced-value');
  });

  it('syncAcrossTabsをfalseに設定すると同期しない', () => {
    const { result } = renderHook(() =>
      useLocalStorage('no-sync-key', 'initial', { syncAcrossTabs: false })
    );

    const storageEvent = new StorageEvent('storage', {
      key: 'no-sync-key',
      newValue: JSON.stringify('should-not-sync'),
    });

    act(() => {
      window.dispatchEvent(storageEvent);
    });

    expect(result.current[0]).toBe('initial');
  });

  it('エラーハンドリングが正しく動作する', () => {
    const consoleSpy = vi.spyOn(console, 'warn').mockImplementation(() => { });

    // 最初にコンポーネントをレンダリング
    const { result } = renderHook(() => useLocalStorage('error-key', 'initial'));

    // localStorage.setItemをスパイしてエラーを発生させる
    localStorageMock.setItem = vi.fn(() => {
      throw new Error('Storage quota exceeded');
    });

    try {
      // setValueを呼び出してエラーを発生させる
      act(() => {
        result.current[1]('error-value');
      });

      expect(consoleSpy).toHaveBeenCalled();
      expect(consoleSpy).toHaveBeenCalledWith(
        'Error setting localStorage key "error-key":',
        expect.any(Error)
      );
    } finally {
      // クリーンアップ
      consoleSpy.mockRestore();
    }
  });

  it('removeValueでのエラーハンドリングが正しく動作する', () => {
    const consoleSpy = vi.spyOn(console, 'warn').mockImplementation(() => { });

    // 最初にコンポーネントをレンダリング
    const { result } = renderHook(() => useLocalStorage('error-key', 'initial'));

    // localStorageMockのremoveItemをエラーを発生させるように変更
    localStorageMock.removeItem = vi.fn(() => {
      throw new Error('Storage error during removal');
    });

    try {
      // removeValueを呼び出してエラーを発生させる
      act(() => {
        result.current[2](); // removeValue
      });

      expect(consoleSpy).toHaveBeenCalled();
      expect(consoleSpy).toHaveBeenCalledWith(
        'Error removing localStorage key "error-key":',
        expect.any(Error)
      );
    } finally {
      // クリーンアップ
      consoleSpy.mockRestore();
    }
  });

  it('getItemでのエラーハンドリングが正しく動作する', () => {
    const consoleSpy = vi.spyOn(console, 'warn').mockImplementation(() => { });

    // localStorageMockのgetItemをエラーを発生させるように変更
    localStorageMock.getItem = vi.fn(() => {
      throw new Error('Storage access error');
    });

    try {
      // コンポーネントをレンダリングしてgetItemエラーを発生させる
      const { result } = renderHook(() => useLocalStorage('error-key', 'initial'));

      expect(consoleSpy).toHaveBeenCalled();
      expect(consoleSpy).toHaveBeenCalledWith(
        'Error reading localStorage key "error-key":',
        expect.any(Error)
      );

      // 初期値が返されることを確認
      expect(result.current[0]).toBe('initial');
    } finally {
      // クリーンアップ
      consoleSpy.mockRestore();
    }
  });
});

describe('useLocalStorageKey', () => {
  beforeEach(() => {
    localStorageMock.clear();
    localStorageMock._storage = {};
    // モックをリセット
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

  it('存在するキーでtrueを返す', () => {
    localStorageMock.setItem('existing-key', 'value');

    const { result } = renderHook(() => useLocalStorageKey('existing-key'));

    expect(result.current).toBe(true);
  });

  it('存在しないキーでfalseを返す', () => {
    const { result } = renderHook(() => useLocalStorageKey('non-existing-key'));

    expect(result.current).toBe(false);
  });
});

describe('useMultipleLocalStorage', () => {
  beforeEach(() => {
    localStorageMock.clear();
    localStorageMock._storage = {};
    // モックをリセット
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

  it('複数のキーを管理できる', () => {
    const keys = {
      name: 'Default Name',
      age: 0,
      email: ''
    };

    const { result } = renderHook(() => useMultipleLocalStorage(keys));

    expect(result.current[0]).toEqual(keys);
  });

  it('個別のキーを設定できる', () => {
    const keys = {
      name: 'Default Name',
      age: 0
    };

    const { result } = renderHook(() => useMultipleLocalStorage(keys));

    act(() => {
      result.current[1]('name', 'John Doe');
    });

    expect(result.current[0].name).toBe('John Doe');
    expect(result.current[0].age).toBe(0);
    expect(localStorageMock.setItem).toHaveBeenCalledWith('name', JSON.stringify('John Doe'));
  });

  it('個別のキーを削除できる', () => {
    const keys = {
      name: 'Default Name',
      age: 0
    };

    const { result } = renderHook(() => useMultipleLocalStorage(keys));

    act(() => {
      result.current[1]('name', 'John Doe');
    });

    act(() => {
      result.current[2]('name'); // removeValue
    });

    expect(result.current[0].name).toBe('Default Name');
    expect(localStorageMock.removeItem).toHaveBeenCalledWith('name');
  });

  it('既存の値を読み込む', () => {
    localStorageMock.setItem('name', JSON.stringify('Existing Name'));
    localStorageMock.setItem('age', JSON.stringify(25));

    const keys = {
      name: 'Default Name',
      age: 0
    };

    const { result } = renderHook(() => useMultipleLocalStorage(keys));

    expect(result.current[0]).toEqual({
      name: 'Existing Name',
      age: 25
    });
  });
});
