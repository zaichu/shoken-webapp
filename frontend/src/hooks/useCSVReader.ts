import { useState, useCallback } from 'react';
import { parseCSVFile } from '../lib/csv/parser';

/**
 * CSVファイルを読み込むためのカスタムフック
 * 各明細種類ごとに独立したCSVファイル処理を提供
 */
export function useCSVReader() {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [fileName, setFileName] = useState('');

  /**
   * CSVファイルをパースする
   * @param file CSVファイル
   * @returns パース結果の配列
   */
  const parseCSV = useCallback(async (file: File): Promise<Record<string, unknown>[]> => {
    setIsLoading(true);
    setError(null);
    setFileName(file.name);

    try {
      const data = await parseCSVFile(file, {
        onStart: () => setIsLoading(true),
        onError: (errorMsg) => setError(errorMsg),
        onComplete: () => setIsLoading(false)
      });
      return data;
    } catch (e) {
      setIsLoading(false);
      throw e;
    }
  }, []);

  /**
   * エラー状態をリセットする
   */
  const resetError = useCallback(() => {
    setError(null);
  }, []);

  /**
   * ファイル名を手動で設定する
   * @param name ファイル名
   */
  const setFile = useCallback((name: string) => {
    setFileName(name);
  }, []);

  /**
   * すべての状態をリセットする
   */
  const reset = useCallback(() => {
    setIsLoading(false);
    setError(null);
    setFileName('');
  }, []);

  return {
    parseCSV,
    isLoading,
    error,
    resetError,
    fileName,
    setFile,
    reset
  };
}
