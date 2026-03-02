/**
 * エラーハンドリングユーティリティ
 */

/**
 * エラーオブジェクトから適切なエラーメッセージを抽出
 */
function getErrorMessage(error: unknown): string {
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
 * 表示用のエラーメッセージを生成
 */
export function getDisplayErrorMessage(error: unknown, fallbackMessage: string): string {
  const message = getErrorMessage(error);
  return message === '不明なエラーが発生しました' ? fallbackMessage : message;
}

