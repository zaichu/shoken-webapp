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
        // V2 API では fin_summary フィールドを使用
        const response = await jquantsApiClient.getStatements(securityCode);
        let dividendValue = '';

        // 配当予想を取得（優先順位: 来期予想 > 今期予想 > 実績）
        for (const summary of response.fin_summary) {
          // 来期予想年間配当金
          if (summary.NextYearForecastDividendPerShareAnnual && summary.NextYearForecastDividendPerShareAnnual !== '') {
            dividendValue = summary.NextYearForecastDividendPerShareAnnual;
          }
          // 今期予想年間配当金（来期予想がない場合のフォールバック）
          else if (!dividendValue && summary.ForecastDividendPerShareAnnual && summary.ForecastDividendPerShareAnnual !== '') {
            dividendValue = summary.ForecastDividendPerShareAnnual;
          }
          // 実績年間配当金（予想がない場合のフォールバック）
          else if (!dividendValue && summary.ResultDividendPerShareAnnual && summary.ResultDividendPerShareAnnual !== '') {
            dividendValue = summary.ResultDividendPerShareAnnual;
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
