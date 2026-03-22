import { describe, expect, it } from 'vitest';

import { ApiError, ApiErrorType } from '../api';

describe('ApiError', () => {
  it('DESERIALIZATION_ERROR ではユーザー向けメッセージを返す', () => {
    const error = new ApiError(
      ApiErrorType.DESERIALIZATION_ERROR,
      'J-Quants API の応答形式が変更された可能性があります'
    );

    expect(error.getUserMessage()).toBe(
      '銘柄情報の取得に失敗しました。時間をおいて再度お試しください。'
    );
  });
});
