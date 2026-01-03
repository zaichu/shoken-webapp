import { useState } from 'react';
import { withSyncErrorHandling } from '@/lib/utils/errorHandler';

/**
 * 基本的なローカルストレージ機能を提供するカスタムフック
 * シンプルで軽量な実装
 */
export function useLocalStorage<T>(
  key: string,
  initialValue: T
): [T, (value: T | ((prevValue: T) => T)) => void, () => void] {
  // SSR対応: サーバーサイドでは初期値を返す
  const isClient = typeof window !== 'undefined';

  // 初期値を取得
  const [storedValue, setStoredValue] = useState<T>(() => {
    if (!isClient) {
      return initialValue;
    }

    return withSyncErrorHandling(
      () => {
        const item = window.localStorage.getItem(key);
        if (item === null) {
          return initialValue;
        }
        return JSON.parse(item);
      },
      `LocalStorage読み込み (キー: ${key})`,
      initialValue
    ) ?? initialValue;
  });

  // 値を設定する関数
  const setValue = (value: T | ((prevValue: T) => T)) => {
    setStoredValue(prevValue => {
      const valueToStore = value instanceof Function ? value(prevValue) : value;

      if (isClient) {
        withSyncErrorHandling(
          () => window.localStorage.setItem(key, JSON.stringify(valueToStore)),
          `LocalStorage保存 (キー: ${key})`
        );
      }

      return valueToStore;
    });
  };

  // 値を削除する関数
  const removeValue = () => {
    setStoredValue(initialValue);

    if (isClient) {
      withSyncErrorHandling(
        () => window.localStorage.removeItem(key),
        `LocalStorage削除 (キー: ${key})`
      );
    }
  };

  return [storedValue, setValue, removeValue];
}

/**
 * ローカルストレージのキーの存在チェック
 */
export function useLocalStorageKey(key: string): boolean {
  if (typeof window === 'undefined') {
    return false;
  }

  return withSyncErrorHandling(
    () => window.localStorage.getItem(key) !== null,
    `LocalStorageキー確認 (キー: ${key})`,
    false
  ) ?? false;
}
