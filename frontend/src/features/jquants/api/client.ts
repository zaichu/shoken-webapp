import {
  JQuantsStatementsResponse
} from './types';
import { apiClient } from '@/lib/api/client';
import { ApiError, ApiErrorType } from '@/lib/types/api';
import axios from 'axios';

/**
 * J-Quants API クライアント（バックエンド経由）
 */
export class JQuantsApiClient {
  private refreshToken: string | null = null;

  constructor(refreshToken?: string) {
    this.refreshToken = refreshToken || null;
  }

  /**
   * リフレッシュトークンを設定
   */
  setRefreshToken(token: string) {
    this.refreshToken = token;
  }

  /**
   * バックエンドの設定済み認証情報を使用してリフレッシュトークンを取得
   */
  async authenticate(): Promise<string> {
    try {
      const response = await apiClient.post<{ refresh_token: string }>('/jquants/auth');
      this.refreshToken = response.data.refresh_token;
      return response.data.refresh_token;
    } catch (error) {
      if (axios.isAxiosError(error)) {
        throw ApiError.fromAxiosError(error);
      }
      throw new ApiError(
        ApiErrorType.DESERIALIZATION_ERROR,
        '認証に失敗しました'
      );
    }
  }

  /**
   * IDトークンを取得
   */
  private async getIdToken(): Promise<string> {
    if (!this.refreshToken) {
      // リフレッシュトークンがない場合は、認証を実行
      await this.authenticate();
    }

    try {
      const response = await apiClient.post<{ id_token: string }>('/jquants/refresh', {
        refresh_token: this.refreshToken,
      });
      return response.data.id_token;
    } catch (error) {
      if (axios.isAxiosError(error)) {
        throw ApiError.fromAxiosError(error);
      }
      throw new ApiError(
        ApiErrorType.DESERIALIZATION_ERROR,
        'IDトークン取得に失敗しました'
      );
    }
  }

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
      const idToken = await this.getIdToken();

      const params: Record<string, string> = { code };
      if (from) params.from = from;
      if (to) params.to = to;

      const response = await apiClient.get<JQuantsStatementsResponse>('/jquants/fins/statements', {
        params,
        headers: {
          Authorization: `Bearer ${idToken}`,
        },
      });

      return response.data;
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
