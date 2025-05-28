import Papa from 'papaparse';
import { CSVParseCallbacks, CSVParseOptions, CSVParseResult } from '../types/csv';

// サポートされているエンコーディング
const SUPPORTED_ENCODINGS = ['shift-jis', 'utf-8', 'iso-8859-1', 'euc-jp'] as const;

interface DecodeResult {
  text: string;
  encoding: string;
  confidence: number;
}

/**
 * 文字化けを検出する
 * @param text チェックするテキスト
 * @returns 文字化けの可能性がある場合true
 */
const detectMojibake = (text: string): boolean => {
  // 一般的な文字化けパターン
  const mojibakePatterns = [
    /[\uFFFD]/g, // 置換文字
    /��/g, // よくある文字化け
    /[�]/g, // 不明な文字
  ];

  return mojibakePatterns.some(pattern => pattern.test(text));
};

/**
 * テキストのエンコーディング信頼度を計算
 * @param text デコードされたテキスト
 * @returns 0-1の信頼度スコア
 */
const calculateEncodingConfidence = (text: string): number => {
  if (!text || text.length === 0) return 0;

  let score = 1.0;

  // 文字化けチェック
  if (detectMojibake(text)) {
    score -= 0.5;
  }

  // 日本語文字の存在をチェック
  const hasJapanese = /[\u3000-\u303f\u3040-\u309f\u30a0-\u30ff\u4e00-\u9faf]/.test(text);
  if (hasJapanese) {
    score += 0.2;
  }

  // 制御文字の存在をチェック（改行とタブ以外）
  // eslint-disable-next-line no-control-regex
  const hasControlChars = /[\x00-\x08\x0B\x0C\x0E-\x1F\x7F]/.test(text);
  if (hasControlChars) {
    score -= 0.3;
  }

  return Math.max(0, Math.min(1, score));
};

/**
 * 複数のエンコーディングを試して正常に読み込めるものを使用
 * @param uint8Array CSVファイルのバイナリデータ
 * @returns デコード結果
 */
const tryDecodeWithMultipleEncodings = (uint8Array: Uint8Array): DecodeResult => {
  const results: DecodeResult[] = [];

  for (const encoding of SUPPORTED_ENCODINGS) {
    try {
      let decoder: TextDecoder;

      // ブラウザ環境でサポートされていないエンコーディングはスキップ
      try {
        decoder = new TextDecoder(encoding, { fatal: false });
      } catch (e) {
        console.warn(`Encoding ${encoding} is not supported:`, e);
        continue;
      }

      // デコード実行
      let text: string;
      try {
        text = decoder.decode(uint8Array);
      } catch (error) {
        console.warn(`Failed to decode with ${encoding}:`, error);
        continue;
      }

      const confidence = calculateEncodingConfidence(text);
      results.push({ text, encoding, confidence });
    } catch (error) {
      console.warn(`Failed to decode with ${encoding}:`, error);
    }
  }

  // 信頼度が最も高いものを選択
  results.sort((a, b) => b.confidence - a.confidence);

  if (results.length > 0 && results[0].confidence > 0.5) {
    return results[0];
  }

  // 全て失敗した場合はutf-8で強制的にデコード
  try {
    const decoder = new TextDecoder('utf-8', { fatal: false });
    return {
      text: decoder.decode(uint8Array),
      encoding: 'utf-8',
      confidence: 0.3
    };
  } catch {
    // utf-8もダメな場合は文字列変換
    const text = Array.from(uint8Array).map(byte => String.fromCharCode(byte)).join('');
    return {
      text,
      encoding: 'unknown',
      confidence: 0.1
    };
  }
};

/**
 * BOMを削除する
 * @param text テキスト
 * @returns BOMが削除されたテキスト
 */
const removeBOM = (text: string): string => {
  return text.replace(/^\uFEFF/, '');
};

/**
 * CSVのヘッダー行を処理する
 * @param headers ヘッダー配列
 * @returns 処理されたヘッダー配列
 */
const processHeaders = (headers: string[]): string[] => {
  return headers.map(header => {
    // 前後の空白を削除
    let processed = header.trim();

    // BOMを削除
    processed = removeBOM(processed);

    // 引用符を削除
    processed = processed.replace(/^["']|["']$/g, '');

    return processed;
  });
};

/**
 * CSVファイルをパースする
 * @param file CSVファイル
 * @param options パースオプション
 * @param callbacks コールバック関数群
 * @returns パース結果
 */
export async function parseCSVFile(
  file: File,
  options?: CSVParseOptions,
  callbacks: CSVParseCallbacks = {}
): Promise<CSVParseResult> {
  if (callbacks.onStart) {
    callbacks.onStart();
  }

  try {
    // ファイルサイズチェック
    const MAX_FILE_SIZE = 50 * 1024 * 1024; // 50MB
    if (file.size > MAX_FILE_SIZE) {
      throw new Error(`ファイルサイズが大きすぎます。最大${MAX_FILE_SIZE / 1024 / 1024}MBまでです。`);
    }

    const arrayBuffer = await file.arrayBuffer();
    const uint8Array = new Uint8Array(arrayBuffer);

    // エンコーディング検出とデコード
    const { text, encoding, confidence } = tryDecodeWithMultipleEncodings(uint8Array);

    console.log(`CSVファイルを ${encoding} エンコーディングで読み込みました (信頼度: ${(confidence * 100).toFixed(1)}%)`);

    if (confidence < 0.7) {
      console.warn('エンコーディングの信頼度が低いため、文字化けの可能性があります');
    }

    // BOMを削除
    let processedText = removeBOM(text);

    // オプションに基づいてテキストを処理
    if (options?.skipHeaderRows) {
      const lines = processedText.split(/\r?\n/);
      const dataLines = lines.slice(options.skipHeaderRows);
      processedText = dataLines.join('\n');
    }

    return new Promise((resolve, reject) => {
      Papa.parse<Record<string, unknown>>(processedText, {
        header: true,
        skipEmptyLines: 'greedy', // 空行を積極的にスキップ
        dynamicTyping: true, // 数値や日付を適切な型に変換
        transformHeader: (header) => {
          // ヘッダーを処理
          return processHeaders([header])[0];
        },
        transform: (value) => {
          // 値の前後の空白を削除
          if (typeof value === 'string') {
            return value.trim();
          }
          return value;
        },
        complete: (results: Papa.ParseResult<Record<string, unknown>>) => {
          const parseResult: CSVParseResult = {
            data: results.data,
            errors: results.errors.map(error => ({
              type: error.type,
              code: error.code,
              message: error.message,
              row: error.row,
            })),
            meta: {
              encoding,
              encodingConfidence: confidence,
              delimiter: results.meta.delimiter,
              linebreak: results.meta.linebreak,
              aborted: results.meta.aborted,
              truncated: results.meta.truncated,
              fields: results.meta.fields,
            },
          };

          // エラーチェック
          if (results.errors.length > 0) {
            const criticalErrors = results.errors.filter(e =>
              e.type === 'Quotes' || e.type === 'FieldMismatch'
            );

            if (criticalErrors.length > 0) {
              const errorMessage = criticalErrors
                .map(e => `行 ${e.row}: ${e.message}`)
                .join('\n');

              if (callbacks.onError) {
                callbacks.onError(`CSV解析エラー:\n${errorMessage}`);
              }
              reject(new Error(errorMessage));
              return;
            }

            // 軽微なエラーは警告として表示
            console.warn('CSV解析警告:', results.errors);
          }

          // データの検証
          if (!results.data || results.data.length === 0) {
            const error = 'CSVデータが見つかりませんでした';
            if (callbacks.onError) {
              callbacks.onError(error);
            }
            reject(new Error(error));
            return;
          }

          // フィールドの検証
          if (!results.meta.fields || results.meta.fields.length === 0) {
            const error = 'CSVヘッダーが見つかりませんでした';
            if (callbacks.onError) {
              callbacks.onError(error);
            }
            reject(new Error(error));
            return;
          }

          if (callbacks.onSuccess) {
            callbacks.onSuccess(parseResult);
          }
          resolve(parseResult);
        },
        error: (error: Error) => {
          const errorMessage = `CSV解析エラー: ${error.message}`;
          if (callbacks.onError) {
            callbacks.onError(errorMessage);
          }
          reject(new Error(errorMessage));
        }
      });
    });
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : '不明なエラー';
    const formattedError = `ファイル読み込みエラー: ${errorMessage}`;

    if (callbacks.onError) {
      callbacks.onError(formattedError);
    }
    throw new Error(formattedError);
  } finally {
    if (callbacks.onComplete) {
      callbacks.onComplete();
    }
  }
}

/**
 * CSV文字列をパースする（テスト用）
 * @param csvString CSV文字列
 * @param options パースオプション
 * @returns パース結果
 */
export function parseCSVString(
  csvString: string,
  options?: Partial<Papa.ParseConfig>
): CSVParseResult {
  const results = Papa.parse<Record<string, unknown>>(csvString, {
    header: true,
    skipEmptyLines: 'greedy',
    dynamicTyping: true,
    ...options,
  });

  return {
    data: results.data,
    errors: results.errors.map(error => ({
      type: error.type,
      code: error.code,
      message: error.message,
      row: error.row,
    })),
    meta: {
      encoding: 'utf-8',
      encodingConfidence: 1,
      delimiter: results.meta.delimiter,
      linebreak: results.meta.linebreak,
      aborted: results.meta.aborted,
      truncated: results.meta.truncated,
      fields: results.meta.fields,
    },
  };
}

/**
 * CSVデータを検証する
 * @param data パースされたCSVデータ
 * @param requiredFields 必須フィールド
 * @returns 検証結果
 */
export function validateCSVData(
  data: Record<string, unknown>[],
  requiredFields: string[]
): { valid: boolean; errors: string[] } {
  const errors: string[] = [];

  if (!data || data.length === 0) {
    errors.push('データが空です');
    return { valid: false, errors };
  }

  // ヘッダーの検証
  const headers = Object.keys(data[0]);
  const missingFields = requiredFields.filter(field => !headers.includes(field));

  if (missingFields.length > 0) {
    errors.push(`必須フィールドが不足しています: ${missingFields.join(', ')}`);
  }

  // 各行のデータ検証
  data.forEach((row, index) => {
    requiredFields.forEach(field => {
      if (row[field] === null || row[field] === undefined || row[field] === '') {
        errors.push(`行 ${index + 1}: "${field}" が空です`);
      }
    });
  });

  return {
    valid: errors.length === 0,
    errors,
  };
}
