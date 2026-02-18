import { useState, useEffect, useRef } from 'react';
import { jquantsApiClient } from '../api/client';
import { parseNumber } from '@/lib/utils/formatters';
import { extractDividendFromSummary } from './useJQuantsDividend';

const MAX_CONCURRENT_REQUESTS = 3;
const MAX_RETRIES = 3;
const RETRY_DELAY_MS = 60_000;

/**
 * 複数銘柄の1株配当を一括取得するフック
 * J-Quants APIから最新予想配当を取得し、Map<銘柄コード, 1株配当>を返す
 *
 * @deprecated バックエンド集約API移行後は useDividendBatch を使用すること
 */
export const useJQuantsDividendBatch = (
  securityCodes: string[],
  enabled: boolean = true
) => {
  const [dividendPerShareMap, setDividendPerShareMap] = useState<Map<string, number>>(new Map());
  const [loading, setLoading] = useState<boolean>(false);
  // 再試行トリガー（失敗時にインクリメントしてuseEffectを再実行）
  const [retryCount, setRetryCount] = useState(0);
  // 再試行回数の上限管理
  const retryCountRef = useRef(0);
  // 前回のコード一覧を保持（不要な再取得を防止）
  const prevCodesRef = useRef<string>('');

  useEffect(() => {
    const codesKey = securityCodes.slice().sort().join(',');
    if (!enabled || securityCodes.length === 0) {
      setDividendPerShareMap(new Map());
      prevCodesRef.current = '';
      retryCountRef.current = 0;
      return;
    }

    // 同じコード一覧かつ再試行でなければスキップ
    if (codesKey === prevCodesRef.current && retryCount === 0) return;

    let isActive = true;
    let retryTimer: ReturnType<typeof setTimeout> | null = null;

    const fetchOneCode = async (code: string) => {
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
    };

    const fetchAll = async () => {
      setLoading(true);
      const map = new Map<string, number>();
      const uniqueCodes = Array.from(new Set(securityCodes));
      const results: PromiseSettledResult<{ code: string; value: number | null }>[] = [];

      for (let i = 0; i < uniqueCodes.length; i += MAX_CONCURRENT_REQUESTS) {
        const chunk = uniqueCodes.slice(i, i + MAX_CONCURRENT_REQUESTS);
        const chunkResults = await Promise.allSettled(chunk.map(fetchOneCode));
        results.push(...chunkResults);
      }

      // value !== null のみ格納（0配当も含む）
      for (const result of results) {
        if (result.status === 'fulfilled' && result.value.value !== null) {
          map.set(result.value.code, result.value.value);
        }
      }

      const hasAnyFailure = results.some(
        r => r.status === 'rejected' ||
          (r.status === 'fulfilled' && r.value.value === null)
      );

      if (isActive) {
        setDividendPerShareMap(map);
        prevCodesRef.current = codesKey;
        setLoading(false);

        // 失敗があれば一定時間後に再試行（上限あり）
        if (hasAnyFailure && retryCountRef.current < MAX_RETRIES) {
          retryCountRef.current += 1;
          prevCodesRef.current = ''; // 次回 useEffect で再実行されるよう初期化
          retryTimer = setTimeout(() => {
            if (isActive) setRetryCount(c => c + 1);
          }, RETRY_DELAY_MS);
        }
      }
    };

    fetchAll();
    return () => {
      isActive = false;
      if (retryTimer !== null) clearTimeout(retryTimer);
    };
  }, [securityCodes, enabled, retryCount]);

  return { dividendPerShareMap, loading };
};
