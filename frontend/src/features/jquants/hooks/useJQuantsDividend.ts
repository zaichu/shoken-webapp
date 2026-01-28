import { useState, useEffect } from 'react';
import { jquantsApiClient } from '../api/client';
import { parseNumber } from '@/lib/utils/formatters';

/**
 * J-Quants APIを使用して配当情報を取得するフック
 */
export const useJQuantsDividend = (
  securityCode: string,
  enabled: boolean = true
) => {
  const [dividendPerShare, setDividendPerShare] = useState<number | undefined>(undefined);
  const [loading, setLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchDividend = async () => {
      if (!enabled || !securityCode) {
        setDividendPerShare(undefined);
        return;
      }

      setLoading(true);
      setError(null);

      try {
        // V2 API では data フィールドを使用
        const response = await jquantsApiClient.getStatements(securityCode);
        let dividendValue = '';

        // レスポンスの data フィールドが配列でない場合はスキップ
        if (!response?.data || !Array.isArray(response.data)) {
          setDividendPerShare(undefined);
          return;
        }

        // 配当予想を取得（優先順位: 来期予想 > 今期予想 > 実績）
        for (const summary of response.data) {
          // 来期予想年間配当金 (NxFDivAnn)
          if (summary.NxFDivAnn && summary.NxFDivAnn !== '') {
            dividendValue = summary.NxFDivAnn;
          }
          // 今期予想年間配当金 (FDivAnn)（来期予想がない場合のフォールバック）
          else if (!dividendValue && summary.FDivAnn && summary.FDivAnn !== '') {
            dividendValue = summary.FDivAnn;
          }
          // 実績年間配当金 (DivAnn)（予想がない場合のフォールバック）
          else if (!dividendValue && summary.DivAnn && summary.DivAnn !== '') {
            dividendValue = summary.DivAnn;
          }
        }
        setDividendPerShare(parseNumber(dividendValue));
      } catch (err) {
        console.error('配当取得エラー:', err);
        setError(err instanceof Error ? err.message : '配当情報の取得に失敗しました');
        setDividendPerShare(undefined);
      } finally {
        setLoading(false);
      }
    };

    fetchDividend();
  }, [securityCode, enabled]);

  return { dividendPerShare, loading, error };
};
