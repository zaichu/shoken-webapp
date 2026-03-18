import { describe, expect, it } from 'vitest';

import { getDisplayErrorMessage } from '../errorHandler';

describe('getDisplayErrorMessage', () => {
  const fallbackMessage = '処理に失敗しました';

  it('Error インスタンスの message を返す', () => {
    expect(getDisplayErrorMessage(new Error('通信エラー'), fallbackMessage)).toBe('通信エラー');
  });

  it('文字列エラーをそのまま返す', () => {
    expect(getDisplayErrorMessage('バリデーションエラー', fallbackMessage)).toBe('バリデーションエラー');
  });

  it('message を持つオブジェクトから文字列化して返す', () => {
    expect(getDisplayErrorMessage({ message: 404 }, fallbackMessage)).toBe('404');
  });

  it('判別不能なエラーではフォールバック文言を返す', () => {
    expect(getDisplayErrorMessage({ code: 'UNKNOWN' }, fallbackMessage)).toBe(fallbackMessage);
    expect(getDisplayErrorMessage(null, fallbackMessage)).toBe(fallbackMessage);
  });
});
