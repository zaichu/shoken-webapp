import { useState, useEffect, useRef, useMemo } from 'react';
import { fetchDividendPerShareBatch, DividendStatus } from '../api/dividendPerShareApi';

const BASE_RETRIES = 3;
const RETRY_DELAY_MS = 15_000;
const SECS_PER_CODE = 12; // バックエンドのレート制御: 12秒/銘柄

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

  // 重複排除・ソートした安定キー（配列参照の変化に影響されない）
  const codesKey = useMemo(
    () => [...new Set(securityCodes)].sort().join(','),
    [securityCodes]
  );

  useEffect(() => {
    if (!enabled || codesKey === '') {
      setDividendPerShareMap(new Map());
      setDividendStatusMap(new Map());
      setFetchedCount(0);
      setTotalCount(0);
      prevCodesRef.current = '';
      retryCountRef.current = 0;
      return;
    }

    if (codesKey === prevCodesRef.current && retryCount === 0) return;

    // コードセットが変わった場合はリトライカウントをリセット
    if (codesKey !== prevCodesRef.current && retryCount === 0) {
      retryCountRef.current = 0;
    }

    // バックエンドが12秒/銘柄で処理するため、銘柄数に応じて最大リトライ数を動的に計算
    const uniqueCodes = codesKey.split(',');
    const uniqueCount = uniqueCodes.length;
    const maxRetries = Math.max(BASE_RETRIES, Math.ceil(uniqueCount * SECS_PER_CODE / RETRY_DELAY_MS) + 3);

    let isActive = true;
    let retryTimer: ReturnType<typeof setTimeout> | null = null;

    const scheduleRetry = () => {
      if (retryCountRef.current < maxRetries) {
        retryCountRef.current += 1;
        // prevCodesRef は timer 発火時にリセット（即時リセットすると再レンダー時に useEffect が再実行される）
        retryTimer = setTimeout(() => {
          if (isActive) {
            prevCodesRef.current = '';
            setRetryCount(c => c + 1);
          }
        }, RETRY_DELAY_MS);
      } else {
        prevCodesRef.current = codesKey;
      }
    };

    const fetchAll = async () => {
      setLoading(true);
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

      // error は自己回復しないため再試行不要。pending のみ対象
      const hasPending = items.some(item => item.status === 'pending');

      if (isActive) {
        setDividendPerShareMap(perShareMap);
        setDividendStatusMap(statusMap);
        setFetchedCount(confirmed);
        prevCodesRef.current = codesKey;
        setLoading(false);

        if (hasPending) {
          scheduleRetry();
        }
      }
    };

    fetchAll().catch(() => {
      if (isActive) {
        setLoading(false);
        // API失敗時もリトライをスケジュール（ネットワーク瞬断・5xx対応）
        scheduleRetry();
      }
    });

    return () => {
      isActive = false;
      if (retryTimer !== null) clearTimeout(retryTimer);
    };
  }, [codesKey, enabled, retryCount]);

  return { dividendPerShareMap, dividendStatusMap, loading, fetchedCount, totalCount };
};
