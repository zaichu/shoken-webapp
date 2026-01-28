import { useState, useEffect } from 'react';
import { jquantsApiClient } from '../api/client';
import { JQuantsStatementData } from '../api/types';
import { parseNumber } from '@/lib/utils/formatters';

/**
 * 決算データから配当情報を抽出する
 * 優先順位: 来期予想 > 今期予想 > 実績
 */
const extractDividendFromSummary = (summary: JQuantsStatementData): string | null => {
  if (summary.NxFDivAnn && summary.NxFDivAnn !== '') {
    return summary.NxFDivAnn;
  }
  if (summary.FDivAnn && summary.FDivAnn !== '') {
    return summary.FDivAnn;
  }
  if (summary.DivAnn && summary.DivAnn !== '') {
    return summary.DivAnn;
  }
  return null;
};

/**
 * J-Quants APIを使用して配当情報を取得するフック
 * 最新の決算データから配当情報を優先的に取得する
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

        // レスポンスの data フィールドが配列でない場合はスキップ
        if (!response?.data || !Array.isArray(response.data)) {
          setDividendPerShare(undefined);
          return;
        }

        // 開示日で降順ソート（最新データを優先）
        const sortedData = [...response.data].sort((a, b) => {
          const dateA = a.DiscDate || '';
          const dateB = b.DiscDate || '';
          return dateB.localeCompare(dateA);
        });

        // 最新の決算データから順に配当情報を検索
        let dividendValue: string | null = null;
        for (const summary of sortedData) {
          dividendValue = extractDividendFromSummary(summary);
          if (dividendValue) {
            break;
          }
        }

        setDividendPerShare(parseNumber(dividendValue || ''));
      } catch (err) {
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
