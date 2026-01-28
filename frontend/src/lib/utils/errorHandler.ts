/**
 * エラーハンドリングユーティリティ
 */

/**
 * エラーオブジェクトから適切なエラーメッセージを抽出
 */
export function getErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  if (typeof error === 'string') {
    return error;
  }
  if (error && typeof error === 'object' && 'message' in error) {
    return String(error.message);
  }
  return '不明なエラーが発生しました';
}

/**
 * エラーをコンソールに記録
 */
export function logError(context: string, error: unknown): void {
  const message = getErrorMessage(error);
  console.error(`[${context}] ${message}`);
}

/**
 * エラーをコンソールに警告として記録
 */
export function logWarning(context: string, message: string, details?: unknown): void {
  if (details !== undefined) {
    console.warn(`[${context}] ${message}`, details);
  } else {
    console.warn(`[${context}] ${message}`);
  }
}

/**
 * エラーハンドリング付きの非同期関数実行
 */
export async function withErrorHandling<T>(
  fn: () => Promise<T>,
  context: string,
  fallbackValue?: T
): Promise<T | undefined> {
  try {
    return await fn();
  } catch (error) {
    logError(context, error);
    return fallbackValue;
  }
}

/**
 * エラーハンドリング付きの同期関数実行
 */
export function withSyncErrorHandling<T>(
  fn: () => T,
  context: string,
  fallbackValue?: T
): T | undefined {
  try {
    return fn();
  } catch (error) {
    logError(context, error);
    return fallbackValue;
  }
}
