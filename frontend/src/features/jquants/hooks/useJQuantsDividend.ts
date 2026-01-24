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
        const statements = await jquantsApiClient.getStatements(securityCode);
        let nextYearForecastDividendPerShareAnnual = '';
        for (const statement of statements.statements) {
          if (statement.NextYearForecastDividendPerShareAnnual === '') {
            continue;
          }
          nextYearForecastDividendPerShareAnnual = statement.NextYearForecastDividendPerShareAnnual;
        }
        setDividendPerShare(parseNumber(nextYearForecastDividendPerShareAnnual));
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
