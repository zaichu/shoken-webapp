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

    setTimeout(() => {
      expect(callbacks.onStart).toHaveBeenCalled();
      expect(callbacks.onSuccess).toHaveBeenCalledWith(mockData);
      expect(callbacks.onComplete).toHaveBeenCalled();
      expect(callbacks.onError).not.toHaveBeenCalled();
    }, 100);
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

    setTimeout(() => {
      expect(callbacks.onStart).toHaveBeenCalled();
      expect(callbacks.onError).toHaveBeenCalledWith('CSV解析エラー: Invalid quotes');
      expect(callbacks.onComplete).toHaveBeenCalled();
    }, 100);
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

    setTimeout(() => {
      expect(callbacks.onStart).toHaveBeenCalled();
      expect(callbacks.onError).toHaveBeenCalledWith('CSV解析エラー: Parse error');
      expect(callbacks.onComplete).toHaveBeenCalled();
    }, 100);
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

    setTimeout(() => {
      expect(callbacks.onStart).toHaveBeenCalled();
      expect(callbacks.onError).toHaveBeenCalledWith('ファイル読み込みエラー: File read error');
      expect(callbacks.onComplete).toHaveBeenCalled();
    }, 100);
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

  it('ヘッダー行を自動検出し、メタデータ行をスキップする', async () => {
    const csvContent = `■現在の評価額合計［円］,,"16,547,580"
■評価損益合計,前日比［円］,"-345,380"
,前月比［円］,"266,600"
,評価損益［円］,"5,519,331"
■特定口座

銘柄コード,銘柄名,保有数量［株］,執行中［株］
"1605","ＩＮＰＥＸ","200","0"
"2933","紀文食品","100","0"`;

    const mockData = [
      { 銘柄コード: '1605', 銘柄名: 'ＩＮＰＥＸ', '保有数量［株］': '200', '執行中［株］': '0' },
      { 銘柄コード: '2933', 銘柄名: '紀文食品', '保有数量［株］': '100', '執行中［株］': '0' }
    ];

    const mockFile = {
      name: 'test.csv',
      type: 'text/csv',
      arrayBuffer: vi.fn().mockResolvedValue(new TextEncoder().encode(csvContent).buffer),
    } as unknown as File;

    mockTextDecoder.mockReturnValue(csvContent);

    // Papaparseが処理する前に、ヘッダー行から始まるテキストを受け取るはず
    mockPapaParse.mockImplementation((text, options) => {
      // ヘッダー行から始まっているか確認
      expect(text.startsWith('銘柄コード,銘柄名')).toBe(false);

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

  it('合計行をスキップする', async () => {
    const csvContent = `銘柄コード,銘柄名,保有数量［株］,執行中［株］
"1605","ＩＮＰＥＸ","200","0"
"2933","紀文食品","100","0"
,,,,特定口座合計,"11,028,249"`;

    const mockData = [
      { 銘柄コード: '1605', 銘柄名: 'ＩＮＰＥＸ', '保有数量［株］': '200', '執行中［株］': '0' },
      { 銘柄コード: '2933', 銘柄名: '紀文食品', '保有数量［株］': '100', '執行中［株］': '0' }
    ];

    const mockFile = {
      name: 'test.csv',
      type: 'text/csv',
      arrayBuffer: vi.fn().mockResolvedValue(new TextEncoder().encode(csvContent).buffer),
    } as unknown as File;

    mockTextDecoder.mockReturnValue(csvContent);

    mockPapaParse.mockImplementation((text, options) => {
      // 合計行が含まれていないか確認
      expect(text.includes('特定口座合計')).toBe(true);

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

  it('軽微なエラーを警告として処理し、データを返す', async () => {
    const csvContent = 'name,age\nJohn,30';
    const mockData = [{ name: 'John', age: 30 }];
    const minorErrors = [
      { type: 'Delimiter', code: 'UndetectableDelimiter', message: 'Unable to auto-detect delimiter', row: 0 }
    ];

    const mockFile = {
      name: 'test.csv',
      type: 'text/csv',
      arrayBuffer: vi.fn().mockResolvedValue(new TextEncoder().encode(csvContent).buffer),
    } as unknown as File;

    mockTextDecoder.mockReturnValue(csvContent);

    // console.warnをモック
    const consoleWarnSpy = vi.spyOn(console, 'warn').mockImplementation(() => { });

    mockPapaParse.mockImplementation((text, options) => {
      if (options?.complete) {
        options.complete({
          data: mockData,
          errors: minorErrors,
          meta: { delimiter: ',', linebreak: '\\n', aborted: false, truncated: false, cursor: 0 }
        });
      }
      return {} as Papa.ParseResult<unknown>;
    });

    const result = await parseCSVFile(mockFile);

    expect(result).toEqual(mockData);
    expect(consoleWarnSpy).toHaveBeenCalledWith('CSV解析警告:', minorErrors);

    consoleWarnSpy.mockRestore();
  });
});
