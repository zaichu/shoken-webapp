import Papa from 'papaparse';
import { CSVParseCallbacks } from '../types/csv';

export async function parseCSVFile(
  file: File,
  callbacks: CSVParseCallbacks = {}
): Promise<any[]> {
  if (callbacks.onStart) callbacks.onStart();

  try {
    const arrayBuffer = await file.arrayBuffer();
    const uint8Array = new Uint8Array(arrayBuffer);

    const decoder = new TextDecoder('shift-jis');
    try {
      const text = decoder.decode(uint8Array);

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
