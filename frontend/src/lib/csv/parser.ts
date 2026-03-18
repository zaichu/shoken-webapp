import Papa from 'papaparse';
import { CSVParseCallbacks, CSVParseOptions, CSVParseResult } from '../types/csv';
import { tryDecodeWithMultipleEncodings, removeBOM } from './encoding';
import { processHeaders, validateFileSize, validateParseErrors, validateParseResult } from './validation';

/**
 * CSVファイルをパースする
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
    validateFileSize(file);

    const arrayBuffer = await file.arrayBuffer();
    const uint8Array = new Uint8Array(arrayBuffer);

    // エンコーディング検出とデコード
    const { text, encoding, confidence } = tryDecodeWithMultipleEncodings(uint8Array);

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
          try {
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
            validateParseErrors(results.errors);

            // データの検証
            validateParseResult(results.data, results.meta.fields);

            if (callbacks.onSuccess) {
              callbacks.onSuccess(parseResult);
            }
            resolve(parseResult);
          } catch (error) {
            const errorMessage = error instanceof Error ? error.message : '不明なエラー';
            if (callbacks.onError) {
              callbacks.onError(errorMessage);
            }
            reject(new Error(errorMessage));
          }
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

// 後方互換性のために元の関数もエクスポート
export { validateCSVData } from './validation';
