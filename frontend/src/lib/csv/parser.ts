import Papa from 'papaparse';
import { CSVParseCallbacks, CSVParseOptions } from '../types/csv';

/**
 * 複数のエンコーディングを試して正常に読み込めるものを使用
 * @param uint8Array CSVファイルのバイナリデータ
 * @returns デコード結果のテキスト
 */
const tryDecodeWithMultipleEncodings = (uint8Array: Uint8Array): { text: string; encoding: string } => {
  // 試すエンコーディングの順序
  const encodings = ['shift-jis', 'utf-8', 'iso-8859-1'];

  for (const encoding of encodings) {
    try {
      const decoder = new TextDecoder(encoding);
      const text = decoder.decode(uint8Array);

      // 文字化けチェック（文字化けしていると �� が含まれることが多い）
      if (!text.includes('��') && text.trim().length > 0) {
        return { text, encoding };
      }
    } catch {
      // エラーが出ても次のエンコーディングを試す
      continue;
    }
  }

  // 全て失敗した場合は最後にShift-JISで強制的にデコード
  const decoder = new TextDecoder('shift-jis', { fatal: false });
  return { text: decoder.decode(uint8Array), encoding: 'shift-jis' };
};

/**
 * CSVファイルをパースする
 * @param file CSVファイル
 * @param callbacks コールバック関数群
 * @returns パース結果の配列
 */
export async function parseCSVFile(
  file: File,
  options?: CSVParseOptions,
  callbacks: CSVParseCallbacks = {}
): Promise<Record<string, unknown>[]> {
  if (callbacks.onStart) callbacks.onStart();

  try {
    const arrayBuffer = await file.arrayBuffer();
    const uint8Array = new Uint8Array(arrayBuffer);

    try {
      const { text, encoding } = tryDecodeWithMultipleEncodings(uint8Array);
      console.log(`CSVファイルを ${encoding} エンコーディングで読み込みました`);

      let processedText = text;
      if (options) {
        const lines = text.split("\n");
        const dataLines = lines.slice(options.skipHeaderRows);
        const cleanedDataLines = [];
        for (const line of dataLines) {
          cleanedDataLines.push(line);
        }
        processedText = cleanedDataLines.join('\n');
        console.log(`ヘッダー行をスキップして、データ行のみを処理しました。\n${processedText}`);
      }

      return new Promise((resolve, reject) => {
        Papa.parse<Record<string, unknown>>(processedText, {
          header: true,
          skipEmptyLines: true,
          dynamicTyping: true, // 数値や日付を適切な型に変換
          transformHeader: (header) => header.trim(),
          complete: (results: Papa.ParseResult<Record<string, unknown>>) => {
            if (results.errors.length > 0) {
              // 軽微なエラーは警告として表示し、処理を継続
              const criticalErrors = results.errors.filter(e =>
                e.type === 'Quotes' || e.type === 'FieldMismatch'
              );

              if (criticalErrors.length > 0) {
                const errorMessage = criticalErrors.map(e => e.message).join('. ');
                const error = `CSV解析エラー: ${errorMessage}`;
                if (callbacks.onError) callbacks.onError(error);
                reject(new Error(errorMessage));
              } else {
                // 軽微なエラーは警告として表示
                console.warn('CSV解析警告:', results.errors);
                // データが存在する場合は成功として扱う
                if (results.data && results.data.length > 0) {
                  if (callbacks.onSuccess) callbacks.onSuccess(results.data);
                  resolve(results.data);
                } else {
                  const error = 'CSVデータが見つかりませんでした';
                  if (callbacks.onError) callbacks.onError(error);
                  reject(new Error(error));
                }
              }
            } else {
              if (callbacks.onSuccess) callbacks.onSuccess(results.data);
              resolve(results.data);
            }
            if (callbacks.onComplete) callbacks.onComplete();
          },
          error: (error: Error) => {
            const errorMessage = `CSV解析エラー: ${error.message}`;
            if (callbacks.onError) callbacks.onError(errorMessage);
            reject(error);
            if (callbacks.onComplete) callbacks.onComplete();
          }
        });
      });
    } catch {
      const errorMessage = 'デコードに失敗しました';
      if (callbacks.onError) callbacks.onError(errorMessage);
      throw new Error(errorMessage);
    }
  } catch (e) {
    const errorMessage = e instanceof Error ? e.message : '不明なエラー';
    const formattedError = `ファイル読み込みエラー: ${errorMessage}`;
    if (callbacks.onError) callbacks.onError(formattedError);
    throw e;
  } finally {
    if (callbacks.onComplete) callbacks.onComplete();
  }
}
