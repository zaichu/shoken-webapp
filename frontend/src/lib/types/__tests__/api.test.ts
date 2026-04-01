import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { ApiError, ApiErrorType } from '../api';

describe('ApiError', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2026-04-01T00:00:00.000Z'));
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  describe('ApiError コンストラクタ', () => {
    it('type/message のみで生成できる', () => {
      const error = new ApiError(ApiErrorType.REQUEST_ERROR, 'リクエストに失敗しました');

      expect(error).toBeInstanceOf(Error);
      expect(error.type).toBe(ApiErrorType.REQUEST_ERROR);
      expect(error.message).toBe('リクエストに失敗しました');
    });

    it('全パラメータ指定で各プロパティが設定される', () => {
      const details = { code: 'INVALID_INPUT', field: 'name' };

      const error = new ApiError(
        ApiErrorType.VALIDATION_ERROR,
        '入力内容を確認してください',
        400,
        details,
        '/api/test',
        'POST'
      );

      expect(error.statusCode).toBe(400);
      expect(error.details).toEqual(details);
      expect(error.requestUrl).toBe('/api/test');
      expect(error.requestMethod).toBe('POST');
    });

    const captureStackTraceIt =
      typeof Error.captureStackTrace === 'function' ? it : it.skip;

    captureStackTraceIt('Error.captureStackTrace が存在する環境では呼ばれる', () => {
      const originalCaptureStackTrace = Error.captureStackTrace;
      const captureStackTrace = vi.fn();

      Object.defineProperty(Error, 'captureStackTrace', {
        value: captureStackTrace,
        configurable: true,
      });

      try {
        new ApiError(ApiErrorType.REQUEST_ERROR, 'リクエストに失敗しました');

        expect(captureStackTrace).toHaveBeenCalledTimes(1);
      } finally {
        Object.defineProperty(Error, 'captureStackTrace', {
          value: originalCaptureStackTrace,
          configurable: true,
        });
      }
    });
  });

  describe('toDetailedString', () => {
    it('type と message のみなら Timestamp を含む基本形式を返す', () => {
      const error = new ApiError(ApiErrorType.REQUEST_ERROR, 'リクエストに失敗しました');

      expect(error.toDetailedString()).toBe(
        '[REQUEST_ERROR] リクエストに失敗しました | Timestamp: 2026-04-01T00:00:00.000Z'
      );
    });

    it('statusCode がある場合は Status を含む', () => {
      const error = new ApiError(
        ApiErrorType.VALIDATION_ERROR,
        '入力内容を確認してください',
        400
      );

      expect(error.toDetailedString()).toContain('Status: 400');
    });

    it('requestUrl と requestMethod がある場合は URL を含む', () => {
      const error = new ApiError(
        ApiErrorType.REQUEST_ERROR,
        'リクエストに失敗しました',
        undefined,
        undefined,
        '/api/test',
        'POST'
      );

      expect(error.toDetailedString()).toContain('URL: POST /api/test');
    });

    it('details がある場合は Details を含む', () => {
      const error = new ApiError(
        ApiErrorType.REQUEST_ERROR,
        'リクエストに失敗しました',
        undefined,
        { code: 'INVALID_INPUT', field: 'name' }
      );

      expect(error.toDetailedString()).toContain(
        'Details: {"code":"INVALID_INPUT","field":"name"}'
      );
    });
  });

  describe('getUserMessage', () => {
    it.each([
      [ApiErrorType.NETWORK_ERROR, 'ネットワーク接続を確認してください'],
      [ApiErrorType.TIMEOUT_ERROR, 'リクエストがタイムアウトしました。もう一度お試しください'],
      [ApiErrorType.AUTHENTICATION_ERROR, 'ログインが必要です'],
      [ApiErrorType.AUTHORIZATION_ERROR, 'このリソースへのアクセス権限がありません'],
      [ApiErrorType.NOT_FOUND_ERROR, '指定されたリソースが見つかりません'],
      [ApiErrorType.VALIDATION_ERROR, '入力内容を確認してください'],
      [ApiErrorType.SERVER_ERROR, 'サーバーエラーが発生しました。しばらくしてから再度お試しください'],
      [
        ApiErrorType.DESERIALIZATION_ERROR,
        '銘柄情報の取得に失敗しました。時間をおいて再度お試しください。'
      ],
    ])('%s のユーザー向けメッセージを返す', (type, expectedMessage) => {
      const error = new ApiError(type, '元のメッセージ');

      expect(error.getUserMessage()).toBe(expectedMessage);
    });

    it('REQUEST_ERROR は元のメッセージをそのまま返す', () => {
      const error = new ApiError(ApiErrorType.REQUEST_ERROR, '詳細なエラーメッセージ');

      expect(error.getUserMessage()).toBe('詳細なエラーメッセージ');
    });
  });

  describe('fromHttpResponse', () => {
    it('400 で error.message がある場合は VALIDATION_ERROR とそのメッセージを使う', () => {
      const error = ApiError.fromHttpResponse(
        400,
        { error: { message: '入力内容が不正です' } },
        '/api/test',
        'POST'
      );

      expect(error.type).toBe(ApiErrorType.VALIDATION_ERROR);
      expect(error.message).toBe('入力内容が不正です');
      expect(error.statusCode).toBe(400);
      expect(error.requestUrl).toBe('/api/test');
      expect(error.requestMethod).toBe('POST');
    });

    it('400 で data が null の場合はデフォルトメッセージを使う', () => {
      const error = ApiError.fromHttpResponse(400, null);

      expect(error.type).toBe(ApiErrorType.VALIDATION_ERROR);
      expect(error.message).toBe('リクエストが不正です');
    });

    it.each([
      [401, ApiErrorType.AUTHENTICATION_ERROR, '認証が必要です'],
      [403, ApiErrorType.AUTHORIZATION_ERROR, 'アクセスが拒否されました'],
      [404, ApiErrorType.NOT_FOUND_ERROR, 'リソースが見つかりません'],
      [500, ApiErrorType.SERVER_ERROR, 'サーバーエラーが発生しました'],
      [502, ApiErrorType.SERVER_ERROR, 'サーバーエラーが発生しました'],
      [503, ApiErrorType.SERVER_ERROR, 'サーバーエラーが発生しました'],
      [504, ApiErrorType.SERVER_ERROR, 'サーバーエラーが発生しました'],
    ])('status=%i の場合は適切な ApiError を返す', (status, expectedType, expectedMessage) => {
      const error = ApiError.fromHttpResponse(status, null);

      expect(error.type).toBe(expectedType);
      expect(error.message).toBe(expectedMessage);
      expect(error.statusCode).toBe(status);
    });

    it('未定義の status は RESPONSE_ERROR とステータス付きメッセージを返す', () => {
      const error = ApiError.fromHttpResponse(418, null);

      expect(error.type).toBe(ApiErrorType.RESPONSE_ERROR);
      expect(error.message).toContain('ステータス: 418');
    });

    it('data が object の場合は details にマージする', () => {
      const data = {
        error: { message: '入力内容が不正です' },
        code: 'INVALID_INPUT',
        field: 'name',
      };

      const error = ApiError.fromHttpResponse(400, data);

      expect(error.details).toMatchObject(data);
    });

    it('data が string の場合は details.context.responseData に格納する', () => {
      const error = ApiError.fromHttpResponse(418, 'teapot');

      expect(error.details).toEqual({
        context: {
          responseData: 'teapot',
        },
      });
    });
  });

  describe('isRetryable', () => {
    it.each([
      [ApiErrorType.NETWORK_ERROR, true],
      [ApiErrorType.TIMEOUT_ERROR, true],
      [ApiErrorType.SERVER_ERROR, true],
      [ApiErrorType.VALIDATION_ERROR, false],
    ])('%s の retryable 判定', (type, expected) => {
      const error = new ApiError(type, 'error');

      expect(error.isRetryable()).toBe(expected);
    });
  });

  describe('isUserActionable', () => {
    it.each([
      [ApiErrorType.VALIDATION_ERROR, true],
      [ApiErrorType.AUTHENTICATION_ERROR, true],
      [ApiErrorType.AUTHORIZATION_ERROR, true],
      [ApiErrorType.NETWORK_ERROR, false],
    ])('%s の user actionable 判定', (type, expected) => {
      const error = new ApiError(type, 'error');

      expect(error.isUserActionable()).toBe(expected);
    });
  });
});
