import { JQuantsStatementsResponse } from './types';
import { apiClient } from '@/lib/api/client';
import { ApiError, ApiErrorType } from '@/lib/types/api';
import axios from 'axios';

/**
 * J-Quants API クライアント（バックエンド経由）
 * バックエンドがAPIキー認証を処理するため、フロントエンドではトークン管理不要
 */
export class JQuantsApiClient {
  /**
   * 財務諸表を取得
   * @param code 銘柄コード
   * @param from 開始日付（YYYY-MM-DD形式）
   * @param to 終了日付（YYYY-MM-DD形式）
   */
  async getStatements(
    code: string,
    from?: string,
    to?: string
  ): Promise<JQuantsStatementsResponse> {
    try {
      const params: Record<string, string> = { code };
      if (from) params.from = from;
      if (to) params.to = to;

      const response = await apiClient.get<JQuantsStatementsResponse>(
        '/jquants/fins/statements',
        { params }
      );

      return response;
    } catch (error) {
      if (axios.isAxiosError(error)) {
        throw ApiError.fromAxiosError(error);
      }
      throw new ApiError(
        ApiErrorType.DESERIALIZATION_ERROR,
        '財務諸表の取得に失敗しました'
      );
    }
  }
}

// シングルトンインスタンス
export const jquantsApiClient = new JQuantsApiClient();
