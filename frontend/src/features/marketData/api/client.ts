import { FinancialStatementsResponse } from './types';
import { apiClient } from '@/lib/api/client';
import { listRequestConfig } from '@/lib/api/requestHelpers';
import { ApiError, ApiErrorType } from '@/lib/types/api';

/**
 * 決算情報 API クライアント（バックエンド経由）
 * バックエンドがAPIキー認証を処理するため、フロントエンドではトークン管理不要
 */
export class MarketDataApiClient {
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
  ): Promise<FinancialStatementsResponse> {
    try {
      const params: Record<string, string> = { code };
      if (from) params.from = from;
      if (to) params.to = to;

      const response = await apiClient.get<FinancialStatementsResponse>(
        '/api/v1/financial-statements',
        listRequestConfig(params)
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
export const marketDataApiClient = new MarketDataApiClient();
