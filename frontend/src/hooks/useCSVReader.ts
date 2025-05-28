import { useState, useCallback } from 'react';
import { parseCSVFile } from '../lib/csv/parser';
import { CSVParseOptions } from '@/lib/types/csv';

export interface CSVReaderState {
  isLoading: boolean;
  error: string | null;
  fileName: string;
}

export interface CSVReaderActions {
  parseCSV: (file: File) => Promise<Record<string, unknown>[]>;
  resetError: () => void;
  setFile: (name: string) => void;
  reset: () => void;
}

export type CSVReaderHook = CSVReaderState & CSVReaderActions;

/**
 * CSVファイルを読み込むためのカスタムフック
 * ファイルの読み込み、パース、エラーハンドリングを統一的に管理する
 */
export function useCSVReader(options?: CSVParseOptions): CSVReaderHook {
  const [state, setState] = useState<CSVReaderState>({
    isLoading: false,
    error: null,
    fileName: '',
  });

  /**
   * CSVファイルをパースする
   */
  const parseCSV = useCallback(async (file: File): Promise<Record<string, unknown>[]> => {
    setState(prev => ({
      ...prev,
      isLoading: true,
      error: null,
      fileName: file.name,
    }));

    try {
      const data = await parseCSVFile(file,
        options,
        {
          onStart: () => setState(prev => ({ ...prev, isLoading: true })),
          onError: (errorMsg) => setState(prev => ({ ...prev, error: errorMsg })),
          onComplete: () => setState(prev => ({ ...prev, isLoading: false })),
        });
      return data.data;
    } catch (e) {
      setState(prev => ({ ...prev, isLoading: false }));
      throw e;
    }
  }, [options]);

  /**
   * エラー状態をリセットする
   */
  const resetError = useCallback(() => {
    setState(prev => ({ ...prev, error: null }));
  }, []);

  /**
   * ファイル名を手動で設定する
   */
  const setFile = useCallback((name: string) => {
    setState(prev => ({ ...prev, fileName: name }));
  }, []);

  /**
   * すべての状態をリセットする
   */
  const reset = useCallback(() => {
    setState({
      isLoading: false,
      error: null,
      fileName: '',
    });
  }, []);

  return {
    ...state,
    parseCSV,
    resetError,
    setFile,
    reset,
  };
}
