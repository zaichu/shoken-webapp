import { useState, useEffect, useRef } from 'react';
import { jquantsApiClient } from '../api/client';
import { parseNumber } from '@/lib/utils/formatters';
import { extractDividendFromSummary } from './useJQuantsDividend';

/**
 * 複数銘柄の1株配当を一括取得するフック
 * J-Quants APIから最新予想配当を取得し、Map<銘柄コード, 1株配当>を返す
 */
export const useJQuantsDividendBatch = (
  securityCodes: string[],
  enabled: boolean = true
) => {
  const [dividendPerShareMap, setDividendPerShareMap] = useState<Map<string, number>>(new Map());
  const [loading, setLoading] = useState<boolean>(false);
  // 前回のコード一覧を保持（不要な再取得を防止）
  const prevCodesRef = useRef<string>('');

  useEffect(() => {
    const codesKey = securityCodes.slice().sort().join(',');
    if (!enabled || securityCodes.length === 0) {
      setDividendPerShareMap(new Map());
      prevCodesRef.current = '';
      return;
    }

    // 同じコード一覧なら再取得しない
    if (codesKey === prevCodesRef.current) return;

    let isActive = true;

    const fetchAll = async () => {
      setLoading(true);
      const map = new Map<string, number>();

      // 並列で取得
      const results = await Promise.allSettled(
        securityCodes.map(async (code) => {
          const response = await jquantsApiClient.getStatements(code);
          if (!response?.data || !Array.isArray(response.data)) return { code, value: null };

          // 開示日で降順ソート（最新データを優先）
          const sortedData = [...response.data].sort((a, b) => {
            const dateA = a.DiscDate || '';
            const dateB = b.DiscDate || '';
            return dateB.localeCompare(dateA);
          });

          // 最新の決算データから順に配当情報を検索
          for (const summary of sortedData) {
            const dividendValue = extractDividendFromSummary(summary);
            if (dividendValue) {
              return { code, value: parseNumber(dividendValue) };
            }
          }
          return { code, value: null };
        })
      );

      for (const result of results) {
        if (result.status === 'fulfilled' && result.value.value !== null && result.value.value > 0) {
          map.set(result.value.code, result.value.value);
        }
      }

      if (isActive) {
        setDividendPerShareMap(map);
        prevCodesRef.current = codesKey;
        setLoading(false);
      }
    };

    fetchAll();
    return () => {
      isActive = false;
    };
  }, [securityCodes, enabled]);

  return { dividendPerShareMap, loading };
};
