import { renderHook, act } from '@testing-library/react';
import { vi } from 'vitest';
import { useAdvancedLocalStorage, useMultipleLocalStorage } from '../useAdvancedLocalStorage';

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

// イベントリスナーのモック
const eventListeners: { [key: string]: ((event: any) => void)[] } = {};

const mockAddEventListener = vi.fn((type: string, listener: (event: any) => void) => {
  if (!eventListeners[type]) {
    eventListeners[type] = [];
  }
  eventListeners[type].push(listener);
});

const mockRemoveEventListener = vi.fn((type: string, listener: (event: any) => void) => {
  if (eventListeners[type]) {
    const index = eventListeners[type].indexOf(listener);
    if (index > -1) {
      eventListeners[type].splice(index, 1);
    }
  }
});

const mockDispatchEvent = vi.fn((event: any) => {
  const type = event.type;
  if (eventListeners[type]) {
    eventListeners[type].forEach(listener => listener(event));
  }
  return true;
});

Object.defineProperty(window, 'localStorage', { value: mockLocalStorage });
Object.defineProperty(window, 'addEventListener', { value: mockAddEventListener });
Object.defineProperty(window, 'removeEventListener', { value: mockRemoveEventListener });
Object.defineProperty(window, 'dispatchEvent', { value: mockDispatchEvent });

describe('useAdvancedLocalStorage', () => {
  beforeEach(() => {
    mockLocalStorage.clear();
    vi.clearAllMocks();
    Object.keys(eventListeners).forEach(key => {
      eventListeners[key] = [];
    });
  });

  it('基本的な機能が動作する', () => {
    const { result } = renderHook(() => useAdvancedLocalStorage('test-key', 'initial'));
    
    expect(result.current[0]).toBe('initial');
    
    act(() => {
      result.current[1]('updated');
    });
    
    expect(result.current[0]).toBe('updated');
    expect(mockLocalStorage.setItem).toHaveBeenCalledWith('test-key', JSON.stringify('updated'));
  });

  it('カスタムシリアライザーが動作する', () => {
    const customSerializer = {
      serialize: (value: string) => `custom:${value}`,
      deserialize: (text: string) => text.replace('custom:', ''),
    };

    const { result } = renderHook(() => 
      useAdvancedLocalStorage('custom-key', 'initial', { serializer: customSerializer })
    );

    act(() => {
      result.current[1]('test-value');
    });

    expect(mockLocalStorage.setItem).toHaveBeenCalledWith('custom-key', 'custom:test-value');
  });

  it('既存の値をカスタムシリアライザーで読み込む', () => {
    const customSerializer = {
      serialize: (value: string) => `custom:${value}`,
      deserialize: (text: string) => text.replace('custom:', ''),
    };

    mockLocalStorage.setItem('existing-key', 'custom:existing-value');
    
    const { result } = renderHook(() => 
      useAdvancedLocalStorage('existing-key', 'default', { serializer: customSerializer })
    );

    expect(result.current[0]).toBe('existing-value');
  });

  it('タブ間同期が動作する', () => {
    const { result } = renderHook(() => useAdvancedLocalStorage('sync-key', 'initial'));

    // StorageEventをシミュレート
    const storageEvent = new StorageEvent('storage', {
      key: 'sync-key',
      newValue: JSON.stringify('synced-value'),
    });

    act(() => {
      mockDispatchEvent(storageEvent);
    });

    expect(result.current[0]).toBe('synced-value');
  });

  it('カスタムイベントでの同期が動作する', () => {
    const { result } = renderHook(() => useAdvancedLocalStorage('custom-sync-key', 'initial'));

    // カスタムイベントをシミュレート
    const customEvent = new CustomEvent('local-storage', {
      detail: { key: 'custom-sync-key', newValue: 'custom-synced-value' }
    });

    act(() => {
      mockDispatchEvent(customEvent);
    });

    expect(result.current[0]).toBe('custom-synced-value');
  });

  it('syncAcrossTabsをfalseに設定すると同期しない', () => {
    const { result } = renderHook(() =>
      useAdvancedLocalStorage('no-sync-key', 'initial', { syncAcrossTabs: false })
    );

    const storageEvent = new StorageEvent('storage', {
      key: 'no-sync-key',
      newValue: JSON.stringify('should-not-sync'),
    });

    act(() => {
      mockDispatchEvent(storageEvent);
    });

    expect(result.current[0]).toBe('initial');
  });

  it('値の設定時にカスタムイベントが発火される', () => {
    const { result } = renderHook(() => useAdvancedLocalStorage('event-key', 'initial'));

    act(() => {
      result.current[1]('new-value');
    });

    expect(mockDispatchEvent).toHaveBeenCalledWith(
      expect.objectContaining({
        type: 'local-storage',
        detail: { key: 'event-key', newValue: 'new-value' }
      })
    );
  });

  it('値の削除時にカスタムイベントが発火される', () => {
    const { result } = renderHook(() => useAdvancedLocalStorage('remove-key', 'initial'));

    act(() => {
      result.current[2](); // removeValue
    });

    expect(mockDispatchEvent).toHaveBeenCalledWith(
      expect.objectContaining({
        type: 'local-storage',
        detail: { key: 'remove-key', newValue: null }
      })
    );
  });

  it('null値の同期が正しく動作する', () => {
    const { result } = renderHook(() => useAdvancedLocalStorage('null-sync-key', 'initial'));

    // null値のStorageEventをシミュレート
    const storageEvent = new StorageEvent('storage', {
      key: 'null-sync-key',
      newValue: null,
    });

    act(() => {
      mockDispatchEvent(storageEvent);
    });

    expect(result.current[0]).toBe('initial');
  });

  it('エラーハンドリングが正しく動作する', () => {
    const consoleSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});
    
    const { result } = renderHook(() => useAdvancedLocalStorage('error-key', 'initial'));

    // setItemでエラーを発生させる
    mockLocalStorage.setItem.mockImplementationOnce(() => {
      throw new Error('Storage quota exceeded');
    });

    act(() => {
      result.current[1]('error-value');
    });

    expect(consoleSpy).toHaveBeenCalled();
    expect(result.current[0]).toBe('error-value'); // 状態は更新される
    
    consoleSpy.mockRestore();
  });

  it('デシリアライズエラーハンドリングが正しく動作する', () => {
    const consoleSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});
    
    const { result } = renderHook(() => useAdvancedLocalStorage('parse-error-key', 'initial'));

    // 無効なJSONでStorageEventをシミュレート
    const storageEvent = new StorageEvent('storage', {
      key: 'parse-error-key',
      newValue: 'invalid-json',
    });

    act(() => {
      mockDispatchEvent(storageEvent);
    });

    expect(consoleSpy).toHaveBeenCalled();
    expect(result.current[0]).toBe('initial'); // 初期値にフォールバック
    
    consoleSpy.mockRestore();
  });
});

describe('useMultipleLocalStorage', () => {
  beforeEach(() => {
    mockLocalStorage.clear();
    vi.clearAllMocks();
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
    expect(mockLocalStorage.setItem).toHaveBeenCalledWith('name', JSON.stringify('John Doe'));
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
    expect(mockLocalStorage.removeItem).toHaveBeenCalledWith('name');
  });

  it('既存の値を読み込む', () => {
    mockLocalStorage.setItem('name', JSON.stringify('Existing Name'));
    mockLocalStorage.setItem('age', JSON.stringify(25));

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

  it('エラーハンドリングが正しく動作する', () => {
    const consoleSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});
    
    const keys = { name: 'Default Name' };
    const { result } = renderHook(() => useMultipleLocalStorage(keys));

    // setItemでエラーを発生させる
    mockLocalStorage.setItem.mockImplementationOnce(() => {
      throw new Error('Storage quota exceeded');
    });

    act(() => {
      result.current[1]('name', 'Error Name');
    });

    expect(consoleSpy).toHaveBeenCalled();
    expect(result.current[0].name).toBe('Error Name'); // 状態は更新される
    
    consoleSpy.mockRestore();
  });
});
