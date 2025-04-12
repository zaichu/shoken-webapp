import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useCSVReader } from './useCSVReader';
import { parseCSVFile } from '../services/csvUtils';

// parseCSVFileをモック化
vi.mock('../services/csvUtils', () => ({
  parseCSVFile: vi.fn()
}));

describe('useCSVReader フック', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('初期状態が正しい', () => {
    const { result } = renderHook(() => useCSVReader());

    expect(result.current.isLoading).toBe(false);
    expect(result.current.error).toBeNull();
    expect(result.current.fileName).toBe('');
  });

  it('parseCSV が正しくファイルを処理する', async () => {
    const mockData = [{ column1: 'value1', column2: 'value2' }];

    // コールバックを実行するようにモックを実装
    (parseCSVFile as unknown as ReturnType<typeof vi.fn>).mockImplementation(
      (_file, callbacks: any) => {
        // onStartコールバックを呼び出し
        if (callbacks.onStart) callbacks.onStart();

        // データを返す前にonCompleteコールバックを呼び出し
        if (callbacks.onComplete) callbacks.onComplete();

        return Promise.resolve(mockData);
      }
    );

    const { result } = renderHook(() => useCSVReader());
    const file = new File(['test csv content'], 'test.csv', { type: 'text/csv' });

    let returnedData: any;
    await act(async () => {
      returnedData = await result.current.parseCSV(file);
    });

    expect(returnedData).toEqual(mockData);
    expect(result.current.fileName).toBe('test.csv');
    expect(result.current.isLoading).toBe(false);
    expect(result.current.error).toBeNull();

    expect(parseCSVFile).toHaveBeenCalledWith(file, expect.any(Object));
  });

  it('エラーが発生した場合に適切に処理する', async () => {
    const testError = new Error('CSVパースエラー');
    (parseCSVFile as unknown as ReturnType<typeof vi.fn>).mockRejectedValue(testError);

    const { result } = renderHook(() => useCSVReader());
    const file = new File(['invalid csv'], 'invalid.csv', { type: 'text/csv' });

    let error: Error | null = null;
    await act(async () => {
      try {
        await result.current.parseCSV(file);
      } catch (e) {
        error = e as Error;
      }
    });

    expect(error).toBeTruthy();
    expect(result.current.fileName).toBe('invalid.csv');
    // onCompleteが呼ばれるとisLoadingはfalseになるはず
    expect(result.current.isLoading).toBe(false);
  });

  it('resetError が正しくエラー状態をリセットする', async () => {
    const { result } = renderHook(() => useCSVReader());

    // エラーを設定
    await act(async () => {
      (parseCSVFile as unknown as ReturnType<typeof vi.fn>).mockImplementationOnce(
        (_file: File, callbacks: any) => {
          callbacks.onError('テストエラー');
          return Promise.reject(new Error('テストエラー'));
        }
      );

      try {
        await result.current.parseCSV(new File([], 'test.csv'));
      } catch (e) {
        // エラーは期待通り
      }
    });

    expect(result.current.error).toBeTruthy();

    // エラーをリセット
    act(() => {
      result.current.resetError();
    });

    expect(result.current.error).toBeNull();
  });
});
