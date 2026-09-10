import { useState, useEffect, useRef } from 'react';
import { fetchDividendPerShareBatch, DividendStatus } from '../api/dividendPerShareApi';

const BASE_RETRIES = 3;
const RETRY_DELAY_MS = 15_000;
const SECS_PER_CODE = 12; // バックエンドのレート制御: 12秒/銘柄
const MS_PER_SEC = 1_000;
// pending再確認の上限キャップ: 100回 × 15秒 = 約25分。
// 100銘柄(バックエンド処理約20分)をカバーしつつ、1000銘柄超での数時間ポーリングを防ぐ
const MAX_PENDING_RETRIES = 100;
// 通信失敗時はバックエンドの処理待ちと無関係のため、固定少数回で打ち切る
const MAX_NETWORK_RETRIES = 3;

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
  const [retryTick, setRetryTick] = useState(0);
  // pending再確認と通信失敗のリトライは別カウンタで管理する
  // （共有すると単位修正後のpending上限が通信障害時にも適用されてしまう）
  const pendingRetryCountRef = useRef(0);
  const networkRetryCountRef = useRef(0);
  const prevCodesRef = useRef<string>('');
  // 直前フェッチ開始時の codesKey を保持し、銘柄集合の実変更を判定する
  const lastCodesKeyRef = useRef<string>('');

  // 重複排除・ソートした安定キー（配列参照の変化に影響されない）
  const codesKey = [...new Set(securityCodes)].sort().join(',');

  useEffect(() => {
    if (!enabled || codesKey === '') {
      setDividendPerShareMap(new Map());
      setDividendStatusMap(new Map());
      setFetchedCount(0);
      setTotalCount(0);
      prevCodesRef.current = '';
      pendingRetryCountRef.current = 0;
      networkRetryCountRef.current = 0;
      lastCodesKeyRef.current = '';
      return;
    }

    // 前回フェッチ済みの codesKey なら再フェッチしない
    if (codesKey === prevCodesRef.current) return;

    // 実際の銘柄集合が変わった場合のみリトライカウントをリセット
    // （リトライ時は prevCodesRef が '' になるが lastCodesKeyRef は変わらないため区別できる）
    if (codesKey !== lastCodesKeyRef.current) {
      pendingRetryCountRef.current = 0;
      networkRetryCountRef.current = 0;
      lastCodesKeyRef.current = codesKey;
    }

    // バックエンドが12秒/銘柄で処理するため、銘柄数に応じて最大リトライ数を動的に計算
    // SECS_PER_CODE(秒)をミリ秒に変換してからRETRY_DELAY_MS(ミリ秒)で割る
    const uniqueCodes = codesKey.split(',');
    const uniqueCount = uniqueCodes.length;
    const pendingMaxRetries = Math.min(
      MAX_PENDING_RETRIES,
      Math.max(BASE_RETRIES, Math.ceil(uniqueCount * SECS_PER_CODE * MS_PER_SEC / RETRY_DELAY_MS) + 3)
    );

    let isActive = true;
    let retryTimer: ReturnType<typeof setTimeout> | null = null;

    const scheduleRetry = (kind: 'pending' | 'network') => {
      const countRef = kind === 'pending' ? pendingRetryCountRef : networkRetryCountRef;
      const maxRetries = kind === 'pending' ? pendingMaxRetries : MAX_NETWORK_RETRIES;
      if (countRef.current < maxRetries) {
        countRef.current += 1;
        // prevCodesRef は timer 発火時にリセット（即時リセットすると再レンダー時に useEffect が再実行される）
        retryTimer = setTimeout(() => {
          if (isActive) {
            prevCodesRef.current = '';
            setRetryTick(c => c + 1);
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
        if (item.status === 'ok' && item.dividend_per_share !== null && item.dividend_per_share !== undefined && item.dividend_per_share > 0) {
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
          scheduleRetry('pending');
        }
      }
    };

    fetchAll().catch(() => {
      if (isActive) {
        setLoading(false);
        // API失敗時もリトライをスケジュール（ネットワーク瞬断・5xx対応）
        // pending再確認とは別枠の固定少数回で打ち切る
        scheduleRetry('network');
      }
    });

    return () => {
      isActive = false;
      if (retryTimer !== null) clearTimeout(retryTimer);
    };
  }, [codesKey, enabled, retryTick]);

  return { dividendPerShareMap, dividendStatusMap, loading, fetchedCount, totalCount };
};
