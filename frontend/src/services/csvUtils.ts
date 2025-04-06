// src/services/csvUtils.ts
import Papa from 'papaparse';

// 日付文字列をパースする関数
export function parseDate(dateStr: string): Date | null {
  if (!dateStr) return null;
  
  // YYYY/MM/DD形式の日付をパース
  const match = dateStr.match(/^(\d{4})\/(\d{1,2})\/(\d{1,2})$/);
  if (match) {
    const year = parseInt(match[1], 10);
    const month = parseInt(match[2], 10) - 1; // JavaScriptの月は0から始まる
    const day = parseInt(match[3], 10);
    return new Date(year, month, day);
  }
  
  return null;
}

// 数値文字列をパースする関数（カンマを除去）
export function parseNumberString(numStr: string | null | undefined): number | null {
  if (!numStr) return null;
  const cleanStr = numStr.replace(/,/g, '');
  const num = parseFloat(cleanStr);
  return isNaN(num) ? null : num;
}

// CSVファイルパース関数
export async function parseCSVFile(file: File, 
  callbacks: {
    onStart?: () => void;
    onSuccess?: (data: any[]) => void;
    onError?: (error: string) => void;
    onComplete?: () => void;
  } = {}
): Promise<any[]> {
  if (callbacks.onStart) callbacks.onStart();
  
  try {
    // ファイルをバイナリとして読み込む
    const arrayBuffer = await file.arrayBuffer();
    const uint8Array = new Uint8Array(arrayBuffer);
    
    // Shift-JISエンコーディングの処理
    const decoder = new TextDecoder('shift-jis');
    try {
      const text = decoder.decode(uint8Array);
      
      // CSVをパース
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
          error: (error) => {
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
