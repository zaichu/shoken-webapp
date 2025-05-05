import Papa from 'papaparse';
import { CSVParseCallbacks } from '../types/csv';

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
    } catch (e) {
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
  callbacks: CSVParseCallbacks = {}
): Promise<any[]> {
  if (callbacks.onStart) callbacks.onStart();

  try {
    const arrayBuffer = await file.arrayBuffer();
    const uint8Array = new Uint8Array(arrayBuffer);

    try {
      // 複数のエンコーディングを試してデコード
      const { text, encoding } = tryDecodeWithMultipleEncodings(uint8Array);
      console.log(`CSVファイルを ${encoding} エンコーディングで読み込みました`);

      return new Promise((resolve, reject) => {
        Papa.parse(text, {
          header: true,
          skipEmptyLines: true,
          transformHeader: (header) => header.trim(),
          complete: (results) => {
            if (results.errors.length > 0) {
              const errorMessage = results.errors.map(e => e.message).join('. ');
              const error = `CSV解析エラー: ${errorMessage}`;
              if (callbacks.onError) callbacks.onError(error);
              reject(new Error(errorMessage));
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
    } catch (e) {
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
