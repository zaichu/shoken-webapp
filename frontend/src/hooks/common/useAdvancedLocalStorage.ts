import { useState, useEffect, useCallback } from 'react';

export interface UseAdvancedLocalStorageOptions<T = unknown> {
  serializer?: {
    serialize: (value: T) => string;
    deserialize: (text: string) => T;
  };
  syncAcrossTabs?: boolean;
}

/**
 * 高度なローカルストレージ機能を提供するカスタムフック
 * タブ間同期、カスタムシリアライザーなどの機能を含む
 */
export function useAdvancedLocalStorage<T>(
  key: string,
  initialValue: T,
  options: UseAdvancedLocalStorageOptions<T> = {}
) {
  const {
    serializer = {
      serialize: JSON.stringify,
      deserialize: JSON.parse,
    },
    syncAcrossTabs = true
  } = options;

  const isClient = typeof window !== 'undefined';

  const [storedValue, setStoredValue] = useState<T>(() => {
    if (!isClient) {
      return initialValue;
    }

    try {
      const item = window.localStorage.getItem(key);
      if (item === null) {
        return initialValue;
      }
      return serializer.deserialize(item);
    } catch (error) {
      console.warn(`ローカルストレージからの読み込みに失敗しました (キー: ${key}):`, error);
      return initialValue;
    }
  });

  const setValue = useCallback((value: T | ((prevValue: T) => T)) => {
    try {
      setStoredValue(prevValue => {
        const valueToStore = value instanceof Function ? value(prevValue) : value;

        if (isClient) {
          try {
            window.localStorage.setItem(key, serializer.serialize(valueToStore));

            // タブ間同期のカスタムイベントを発火
            if (syncAcrossTabs) {
              window.dispatchEvent(new CustomEvent('local-storage', {
                detail: { key, newValue: valueToStore }
              }));
            }
          } catch (error) {
            console.warn(`ローカルストレージへの保存に失敗しました (キー: ${key}):`, error);
          }
        }

        return valueToStore;
      });
    } catch (error) {
      console.warn(`値の設定に失敗しました (キー: ${key}):`, error);
    }
  }, [key, serializer, isClient, syncAcrossTabs]);

  const removeValue = useCallback(() => {
    try {
      setStoredValue(initialValue);

      if (isClient) {
        try {
          window.localStorage.removeItem(key);

          // タブ間同期のカスタムイベントを発火
          if (syncAcrossTabs) {
            window.dispatchEvent(new CustomEvent('local-storage', {
              detail: { key, newValue: null }
            }));
          }
        } catch (error) {
          console.warn(`ローカルストレージからの削除に失敗しました (キー: ${key}):`, error);
        }
      }
    } catch (error) {
      console.warn(`値の削除に失敗しました (キー: ${key}):`, error);
    }
  }, [key, initialValue, isClient, syncAcrossTabs]);

  // タブ間での同期を監視
  useEffect(() => {
    if (!isClient || !syncAcrossTabs) {
      return;
    }

    const handleStorageChange = (e: StorageEvent | CustomEvent) => {
      if ('key' in e && e.key === key) {
        // 標準のstorageイベント
        try {
          if (e.newValue === null) {
            setStoredValue(initialValue);
          } else {
            setStoredValue(serializer.deserialize(e.newValue));
          }
        } catch (error) {
          console.warn(`ストレージイベントの解析に失敗しました (キー: ${key}):`, error);
        }
      } else if ('detail' in e && e.detail.key === key) {
        // カスタムイベント
        if (e.detail.newValue === null) {
          setStoredValue(initialValue);
        } else {
          setStoredValue(e.detail.newValue);
        }
      }
    };

    // 標準のstorageイベントとカスタムイベントの両方を監視
    window.addEventListener('storage', handleStorageChange);
    window.addEventListener('local-storage', handleStorageChange as EventListener);

    return () => {
      window.removeEventListener('storage', handleStorageChange);
      window.removeEventListener('local-storage', handleStorageChange as EventListener);
    };
  }, [key, initialValue, serializer, isClient, syncAcrossTabs]);

  return [storedValue, setValue, removeValue] as const;
}

/**
 * 複数のローカルストレージキーを管理
 */
export function useMultipleLocalStorage<T extends Record<string, unknown>>(
  keys: Record<keyof T, unknown>
): [T, (key: keyof T, value: T[keyof T]) => void, (key: keyof T) => void] {
  const [values, setValues] = useState<T>(() => {
    if (typeof window === 'undefined') {
      return keys as T;
    }

    const result = {} as T;
    Object.entries(keys).forEach(([key, defaultValue]) => {
      try {
        const item = window.localStorage.getItem(key);
        result[key as keyof T] = item ? JSON.parse(item) : defaultValue;
      } catch {
        result[key as keyof T] = defaultValue;
      }
    });
    return result;
  });

  const setValue = useCallback((key: keyof T, value: T[keyof T]) => {
    setValues(prev => ({ ...prev, [key]: value }));

    if (typeof window !== 'undefined') {
      try {
        window.localStorage.setItem(key as string, JSON.stringify(value));
      } catch (error) {
        console.warn(`ローカルストレージへの保存に失敗しました (キー: ${String(key)}):`, error);
      }
    }
  }, []);

  const removeValue = useCallback((key: keyof T) => {
    setValues(prev => ({ ...prev, [key]: keys[key] }));

    if (typeof window !== 'undefined') {
      try {
        window.localStorage.removeItem(key as string);
      } catch (error) {
        console.warn(`ローカルストレージからの削除に失敗しました (キー: ${String(key)}):`, error);
      }
    }
  }, [keys]);

  return [values, setValue, removeValue];
}
