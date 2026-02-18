import { useState, useEffect, useRef } from 'react';
import { fetchDividendPerShareBatch, DividendStatus } from '../api/dividendPerShareApi';

const MAX_RETRIES = 3;
const RETRY_DELAY_MS = 15_000;

/**
 * バックエンド集約APIを使って複数銘柄の1株配当を取得するフック
 * キャッシュ・レート制御はバックエンドが担う
 */
export const useDividendBatch = (
  securityCodes: string[],
  enabled: boolean = true
) => {
  const [dividendPerShareMap, setDividendPerShareMap] = useState<Map<string, number>>(new Map());
  const [dividendStatusMap, setDividendStatusMap] = useState<Map<string, DividendStatus>>(new Map());
  const [loading, setLoading] = useState<boolean>(false);
  const [fetchedCount, setFetchedCount] = useState(0);
  const [totalCount, setTotalCount] = useState(0);
  const [retryCount, setRetryCount] = useState(0);
  const retryCountRef = useRef(0);
  const prevCodesRef = useRef<string>('');

  useEffect(() => {
    const codesKey = securityCodes.slice().sort().join(',');
    if (!enabled || securityCodes.length === 0) {
      setDividendPerShareMap(new Map());
      setDividendStatusMap(new Map());
      setFetchedCount(0);
      setTotalCount(0);
      prevCodesRef.current = '';
      retryCountRef.current = 0;
      return;
    }

    if (codesKey === prevCodesRef.current && retryCount === 0) return;

    let isActive = true;
    let retryTimer: ReturnType<typeof setTimeout> | null = null;

    const fetchAll = async () => {
      setLoading(true);
      const uniqueCodes = Array.from(new Set(securityCodes));
      setTotalCount(uniqueCodes.length);

      const items = await fetchDividendPerShareBatch(uniqueCodes);

      const perShareMap = new Map<string, number>();
      const statusMap = new Map<string, DividendStatus>();
      let confirmed = 0;

      for (const item of items) {
        statusMap.set(item.security_code, item.status);
        if (item.status === 'ok' && item.dividend_per_share !== null && item.dividend_per_share > 0) {
          perShareMap.set(item.security_code, item.dividend_per_share);
        }
        if (item.status === 'ok' || item.status === 'zero') {
          confirmed++;
        }
      }

      const hasPending = items.some(item => item.status === 'pending' || item.status === 'error');

      if (isActive) {
        setDividendPerShareMap(perShareMap);
        setDividendStatusMap(statusMap);
        setFetchedCount(confirmed);
        prevCodesRef.current = codesKey;
        setLoading(false);

        if (hasPending && retryCountRef.current < MAX_RETRIES) {
          retryCountRef.current += 1;
          prevCodesRef.current = '';
          retryTimer = setTimeout(() => {
            if (isActive) setRetryCount(c => c + 1);
          }, RETRY_DELAY_MS);
        }
      }
    };

    fetchAll().catch(() => {
      if (isActive) {
        setLoading(false);
        prevCodesRef.current = codesKey;
      }
    });

    return () => {
      isActive = false;
      if (retryTimer !== null) clearTimeout(retryTimer);
    };
  }, [securityCodes, enabled, retryCount]);

  return { dividendPerShareMap, dividendStatusMap, loading, fetchedCount, totalCount };
};
