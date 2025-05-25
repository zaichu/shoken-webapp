import { describe, it, expect, beforeEach, vi } from 'vitest';
import { parseCSVFile } from '../parser';
import Papa from 'papaparse';

// Papaparseをモック化
vi.mock('papaparse', () => ({
  default: {
    parse: vi.fn(),
  },
}));

const mockPapaParse = vi.mocked(Papa.parse);

// TextDecoderをモック化
const mockTextDecoder = vi.fn();
Object.defineProperty(global, 'TextDecoder', {
  value: vi.fn().mockImplementation((encoding) => ({
    encoding,
    decode: mockTextDecoder,
  })),
});

describe('parseCSVFile', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockTextDecoder.mockReset();
    mockPapaParse.mockReset();
  });

  it('正常なCSVファイルをパースできる', async () => {
    const csvContent = 'name,age\nJohn,30\nJane,25';
    const mockData = [
      { name: 'John', age: 30 },
      { name: 'Jane', age: 25 }
    ];

    // ファイルのモック（arrayBufferメソッドを追加）
    const mockFile = {
      name: 'test.csv',
      type: 'text/csv',
      arrayBuffer: vi.fn().mockResolvedValue(new TextEncoder().encode(csvContent).buffer),
    } as unknown as File;
    
    // TextDecoderのモック
    mockTextDecoder.mockReturnValue(csvContent);

    // Papaparseのモック
    mockPapaParse.mockImplementation((text, options) => {
      // complete コールバックを呼び出す
      if (options?.complete) {
        options.complete({
          data: mockData,
          errors: [],
          meta: { delimiter: ',', linebreak: '\\n', aborted: false, truncated: false, cursor: 0 }
        });
      }
      return {} as Papa.ParseResult<unknown>;
    });

    const result = await parseCSVFile(mockFile);

    expect(result).toEqual(mockData);
    expect(mockPapaParse).toHaveBeenCalledWith(
      csvContent,
      expect.objectContaining({
        header: true,
        skipEmptyLines: true,
        dynamicTyping: true,
        transformHeader: expect.any(Function),
        complete: expect.any(Function),
        error: expect.any(Function)
      })
    );
  });

  it('コールバック関数が正しく呼ばれる', async () => {
    const csvContent = 'name,age\nJohn,30';
    const mockData = [{ name: 'John', age: 30 }];
    const callbacks = {
      onStart: vi.fn(),
      onSuccess: vi.fn(),
      onComplete: vi.fn(),
      onError: vi.fn(),
    };

    const mockFile = {
      name: 'test.csv',
      type: 'text/csv',
      arrayBuffer: vi.fn().mockResolvedValue(new TextEncoder().encode(csvContent).buffer),
    } as unknown as File;
    mockTextDecoder.mockReturnValue(csvContent);

    mockPapaParse.mockImplementation((text, options) => {
      if (options?.complete) {
        options.complete({
          data: mockData,
          errors: [],
          meta: { delimiter: ',', linebreak: '\\n', aborted: false, truncated: false, cursor: 0 }
        });
      }
      return {} as Papa.ParseResult<unknown>;
    });

    await parseCSVFile(mockFile, callbacks);

    expect(callbacks.onStart).toHaveBeenCalled();
    expect(callbacks.onSuccess).toHaveBeenCalledWith(mockData);
    expect(callbacks.onComplete).toHaveBeenCalled();
    expect(callbacks.onError).not.toHaveBeenCalled();
  });

  it('パースエラーが発生した場合にエラーを処理する', async () => {
    const csvContent = 'invalid,csv,content';
    const parseErrors = [
      { type: 'Quotes', code: 'InvalidQuotes', message: 'Invalid quotes', row: 0 }
    ];
    const callbacks = {
      onStart: vi.fn(),
      onError: vi.fn(),
      onComplete: vi.fn(),
    };

    const mockFile = {
      name: 'test.csv',
      type: 'text/csv',
      arrayBuffer: vi.fn().mockResolvedValue(new TextEncoder().encode(csvContent).buffer),
    } as unknown as File;
    mockTextDecoder.mockReturnValue(csvContent);

    mockPapaParse.mockImplementation((text, options) => {
      if (options?.complete) {
        options.complete({
          data: [],
          errors: parseErrors,
          meta: { delimiter: ',', linebreak: '\\n', aborted: false, truncated: false, cursor: 0 }
        });
      }
      return {} as Papa.ParseResult<unknown>;
    });

    await expect(parseCSVFile(mockFile, callbacks)).rejects.toThrow('Invalid quotes');
    
    expect(callbacks.onStart).toHaveBeenCalled();
    expect(callbacks.onError).toHaveBeenCalledWith('CSV解析エラー: Invalid quotes');
    expect(callbacks.onComplete).toHaveBeenCalled();
  });

  it('Papa.parseでエラーが発生した場合にエラーを処理する', async () => {
    const csvContent = 'name,age\nJohn,30';
    const error = new Error('Parse error');
    const callbacks = {
      onStart: vi.fn(),
      onError: vi.fn(),
      onComplete: vi.fn(),
    };

    const mockFile = {
      name: 'test.csv',
      type: 'text/csv',
      arrayBuffer: vi.fn().mockResolvedValue(new TextEncoder().encode(csvContent).buffer),
    } as unknown as File;
    mockTextDecoder.mockReturnValue(csvContent);

    mockPapaParse.mockImplementation((text, options) => {
      if (options?.error) {
        options.error(error);
      }
      return {} as Papa.ParseResult<unknown>;
    });

    await expect(parseCSVFile(mockFile, callbacks)).rejects.toThrow('Parse error');
    
    expect(callbacks.onStart).toHaveBeenCalled();
    expect(callbacks.onError).toHaveBeenCalledWith('CSV解析エラー: Parse error');
    expect(callbacks.onComplete).toHaveBeenCalled();
  });

  it('複数のエンコーディングを試行する', async () => {
    const csvContent = 'name,age\nJohn,30';
    const mockFile = {
      name: 'test.csv',
      type: 'text/csv',
      arrayBuffer: vi.fn().mockResolvedValue(new TextEncoder().encode(csvContent).buffer),
    } as unknown as File;

    // 最初のエンコーディングで文字化けが発生
    mockTextDecoder
      .mockReturnValueOnce('name,age\\n��,30') // shift-jis で文字化け
      .mockReturnValueOnce(csvContent); // utf-8 で成功

    mockPapaParse.mockImplementation((text, options) => {
      if (options?.complete) {
        options.complete({
          data: [{ name: 'John', age: 30 }],
          errors: [],
          meta: { delimiter: ',', linebreak: '\\n', aborted: false, truncated: false, cursor: 0 }
        });
      }
      return {} as Papa.ParseResult<unknown>;
    });

    const result = await parseCSVFile(mockFile);

    expect(result).toEqual([{ name: 'John', age: 30 }]);
    expect(global.TextDecoder).toHaveBeenCalledWith('shift-jis');
    expect(global.TextDecoder).toHaveBeenCalledWith('utf-8');
  });

  it('ファイル読み込みエラーを処理する', async () => {
    const callbacks = {
      onStart: vi.fn(),
      onError: vi.fn(),
      onComplete: vi.fn(),
    };

    // arrayBufferでエラーが発生するファイルをモック
    const mockFile = {
      arrayBuffer: vi.fn().mockRejectedValue(new Error('File read error'))
    } as unknown as File;

    await expect(parseCSVFile(mockFile, callbacks)).rejects.toThrow('File read error');
    
    expect(callbacks.onStart).toHaveBeenCalled();
    expect(callbacks.onError).toHaveBeenCalledWith('ファイル読み込みエラー: File read error');
    expect(callbacks.onComplete).toHaveBeenCalled();
  });

  it('transformHeader関数がヘッダーの空白を除去する', async () => {
    const csvContent = ' name , age \nJohn,30';
    const mockFile = {
      name: 'test.csv',
      type: 'text/csv',
      arrayBuffer: vi.fn().mockResolvedValue(new TextEncoder().encode(csvContent).buffer),
    } as unknown as File;
    mockTextDecoder.mockReturnValue(csvContent);

    let transformHeaderFunc: ((header: string) => string) | undefined;

    mockPapaParse.mockImplementation((text, options) => {
      transformHeaderFunc = options?.transformHeader;
      if (options?.complete) {
        options.complete({
          data: [{ name: 'John', age: 30 }],
          errors: [],
          meta: { delimiter: ',', linebreak: '\\n', aborted: false, truncated: false, cursor: 0 }
        });
      }
      return {} as Papa.ParseResult<unknown>;
    });

    await parseCSVFile(mockFile);

    expect(transformHeaderFunc).toBeDefined();
    expect(transformHeaderFunc!(' test header ')).toBe('test header');
  });

  it('コールバックなしでも動作する', async () => {
    const csvContent = 'name,age\nJohn,30';
    const mockData = [{ name: 'John', age: 30 }];
    const mockFile = {
      name: 'test.csv',
      type: 'text/csv',
      arrayBuffer: vi.fn().mockResolvedValue(new TextEncoder().encode(csvContent).buffer),
    } as unknown as File;
    
    mockTextDecoder.mockReturnValue(csvContent);

    mockPapaParse.mockImplementation((text, options) => {
      if (options?.complete) {
        options.complete({
          data: mockData,
          errors: [],
          meta: { delimiter: ',', linebreak: '\\n', aborted: false, truncated: false, cursor: 0 }
        });
      }
      return {} as Papa.ParseResult<unknown>;
    });

    const result = await parseCSVFile(mockFile);
    expect(result).toEqual(mockData);
  });
});
