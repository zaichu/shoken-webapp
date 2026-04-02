import { JQuantsFinSummaryResponse } from './types';
import { apiClient } from '@/lib/api/client';
import { ApiError, ApiErrorType } from '@/lib/types/api';

/**
 * J-Quants API クライアント（バックエンド経由）
 * バックエンドがAPIキー認証を処理するため、フロントエンドではトークン管理不要
 */
export class JQuantsApiClient {
  /**
   * 決算サマリーを取得
   * @param code 銘柄コード
   * @param from 開始日付（YYYY-MM-DD形式）
   * @param to 終了日付（YYYY-MM-DD形式）
   */
  async getSummary(
    code: string,
    from?: string,
    to?: string
  ): Promise<JQuantsFinSummaryResponse> {
    try {
      const params: Record<string, string> = { code };
      if (from) params.from = from;
      if (to) params.to = to;

      const response = await apiClient.get<JQuantsFinSummaryResponse>(
        '/jquants/fins/summary',
        { params }
      );

      return response;
    } catch (error) {
      if (error instanceof ApiError) {
        throw error;
      }
      throw new ApiError(
        ApiErrorType.DESERIALIZATION_ERROR,
        '決算サマリーの取得に失敗しました'
      );
    }
  }
}

// シングルトンインスタンス
export const jquantsApiClient = new JQuantsApiClient();
