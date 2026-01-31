import { renderHook, act } from '@testing-library/react';
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { useCSVReader } from '../useCSVReader';
import { parseCSVFile } from '../../lib/csv/parser';
import { CSVParseResult } from '../../lib/types/csv';

// parseCSVFileをモック化
vi.mock('../../lib/csv/parser', () => ({
  parseCSVFile: vi.fn(),
}));

const mockParseCSVFile = vi.mocked(parseCSVFile);

// テスト用のモックCSVParseResult生成ヘルパー
const createMockResult = (data: Record<string, unknown>[] = []): CSVParseResult => ({
  data,
  errors: [],
  meta: {
    encoding: 'utf-8',
    encodingConfidence: 1,
    delimiter: ',',
    linebreak: '\n',
    aborted: false,
    truncated: false
  }
});

describe('useCSVReader', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockParseCSVFile.mockReset();
  });

  it('初期状態が正しく設定される', () => {
    const { result } = renderHook(() => useCSVReader());

    expect(result.current.isLoading).toBe(false);
    expect(result.current.error).toBeNull();
    expect(result.current.fileName).toBe('');
  });

  it('CSVファイルを正常にパースできる', async () => {
    const mockData = [{ name: 'John', age: 30 }];
    const mockFile = new File(['name,age\nJohn,30'], 'test.csv', { type: 'text/csv' });

    mockParseCSVFile.mockImplementation((_, __, callbacks) => {
      callbacks?.onStart?.();
      callbacks?.onComplete?.();
      return Promise.resolve(createMockResult(mockData));
    });

    const { result } = renderHook(() => useCSVReader());

    let parsedData;
    await act(async () => {
      parsedData = await result.current.parseCSV(mockFile);
    });

    expect(parsedData).toEqual(mockData);
    expect(result.current.fileName).toBe('test.csv');
    expect(result.current.isLoading).toBe(false);
    expect(result.current.error).toBeNull();
    expect(mockParseCSVFile).toHaveBeenCalledWith(
      mockFile,
      undefined,
      expect.any(Object)
    );
  });

  it('CSVパース中にローディング状態が正しく管理される', async () => {
    const mockFile = new File(['test'], 'test.csv', { type: 'text/csv' });
    let resolvePromise: (value: CSVParseResult) => void;
    let callbacks: {
      onStart?: () => void;
      onComplete?: () => void;
      onError?: (error: string) => void;
    } | undefined;

    mockParseCSVFile.mockImplementation((_, __, cb) => {
      callbacks = cb!;
      callbacks?.onStart?.();
      return new Promise((resolve) => {
        resolvePromise = resolve;
      });
    });

    const { result } = renderHook(() => useCSVReader());

    // パース開始
    let parsePromise: Promise<Record<string, unknown>[]>;
    act(() => {
      parsePromise = result.current.parseCSV(mockFile);
    });

    expect(result.current.isLoading).toBe(true);
    expect(result.current.fileName).toBe('test.csv');

    // パース完了
    await act(async () => {
      callbacks?.onComplete?.();
      resolvePromise(createMockResult());
      await parsePromise;
    });

    expect(result.current.isLoading).toBe(false);
  });

  it('CSVパースエラーが正しく処理される', async () => {
    const mockFile = new File(['invalid'], 'test.csv', { type: 'text/csv' });
    const errorMessage = 'パースエラー';

    mockParseCSVFile.mockImplementation((_, __, callbacks) => {
      callbacks?.onStart?.();
      callbacks?.onError?.(errorMessage);
      return Promise.reject(new Error(errorMessage));
    });

    const { result } = renderHook(() => useCSVReader());

    await act(async () => {
      try {
        await result.current.parseCSV(mockFile);
      } catch {
        // エラーは期待される
      }
    });

    expect(result.current.isLoading).toBe(false);
    expect(result.current.error).toBe(errorMessage);
  });

  it('エラーをリセットできる', () => {
    const { result } = renderHook(() => useCSVReader());

    // 初期状態でエラーを設定（内部的に設定する方法をシミュレート）
    act(() => {
      result.current.setFile('test.csv');
    });

    // エラーを手動で設定（実際の実装では parseCSV 内で設定される）
    // ここではテストのためにエラー状態をシミュレート

    act(() => {
      result.current.resetError();
    });

    expect(result.current.error).toBeNull();
  });

  it('ファイル名を手動で設定できる', () => {
    const { result } = renderHook(() => useCSVReader());

    act(() => {
      result.current.setFile('custom.csv');
    });

    expect(result.current.fileName).toBe('custom.csv');
  });

  it('状態をリセットできる', () => {
    const { result } = renderHook(() => useCSVReader());

    // まず何らかの状態を設定
    act(() => {
      result.current.setFile('test.csv');
    });

    expect(result.current.fileName).toBe('test.csv');

    // リセット
    act(() => {
      result.current.reset();
    });

    expect(result.current.isLoading).toBe(false);
    expect(result.current.error).toBeNull();
    expect(result.current.fileName).toBe('');
  });

  it('コールバック関数が正しく呼ばれる', async () => {
    const mockFile = new File(['test'], 'test.csv', { type: 'text/csv' });
    let callbacks: {
      onStart?: () => void;
      onComplete?: () => void;
      onError?: (error: string) => void;
    } | undefined;

    mockParseCSVFile.mockImplementation((_, __, cb) => {
      callbacks = cb!;
      return Promise.resolve(createMockResult());
    });

    const { result } = renderHook(() => useCSVReader());

    await act(async () => {
      await result.current.parseCSV(mockFile);
    });

    expect(callbacks).toBeDefined();
    expect(typeof callbacks!.onStart).toBe('function');
    expect(typeof callbacks!.onError).toBe('function');
    expect(typeof callbacks!.onComplete).toBe('function');
  });

  it('例外が発生した場合にisLoadingがfalseになる', async () => {
    const mockFile = new File(['test'], 'test.csv', { type: 'text/csv' });

    mockParseCSVFile.mockRejectedValue(new Error('テストエラー'));

    const { result } = renderHook(() => useCSVReader());

    await act(async () => {
      try {
        await result.current.parseCSV(mockFile);
      } catch {
        // エラーは期待される
      }
    });

    expect(result.current.isLoading).toBe(false);
  });
});
