import {
  JQuantsStatementsResponse,
  JQuantsError
} from './types';

/**
 * J-Quants API クライアント（バックエンド経由）
 */
export class JQuantsApiClient {
  // バックエンドのベースURL（環境変数から取得）
  private baseUrl = import.meta.env.VITE_SHOKEN_WEBAPI_API_URL || 'http://localhost:8080';
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
      const response = await fetch(`${this.baseUrl}/jquants/auth`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
      });

      if (!response.ok) {
        const error: JQuantsError = await response.json();
        throw new Error(`認証に失敗しました: ${error.message}`);
      }

      const data = await response.json();
      this.refreshToken = data.refresh_token;
      return data.refresh_token;
    } catch (error) {
      console.error('認証エラー:', error);
      throw error;
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
      const response = await fetch(`${this.baseUrl}/jquants/refresh`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          refresh_token: this.refreshToken,
        }),
      });

      if (!response.ok) {
        const error: JQuantsError = await response.json();
        throw new Error(`認証に失敗しました: ${error.message}`);
      }

      const data = await response.json();
      return data.id_token;
    } catch (error) {
      console.error('IDトークン取得エラー:', error);
      throw error;
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

      const params = new URLSearchParams();
      params.append('code', code);
      if (from) params.append('from', from);
      if (to) params.append('to', to);

      const url = `${this.baseUrl}/jquants/fins/statements?${params.toString()}`;
      const response = await fetch(url,
        {
          headers: {
            Authorization: `Bearer ${idToken}`,
          },
        }
      );

      if (!response.ok) {
        const error: JQuantsError = await response.json();
        throw new Error(`財務諸表の取得に失敗しました: ${error.message}`);
      }

      return await response.json();
    } catch (error) {
      console.error('財務諸表取得エラー:', error);
      throw error;
    }
  }
}

// シングルトンインスタンス
export const jquantsApiClient = new JQuantsApiClient();
