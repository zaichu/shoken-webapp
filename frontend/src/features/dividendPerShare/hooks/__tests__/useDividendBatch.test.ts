import { describe, it, expect, vi, beforeEach, afterEach, Mock } from 'vitest';
import { act, renderHook, waitFor } from '@testing-library/react';
import { useDividendBatch } from '../useDividendBatch';
import * as api from '../../api/dividendPerShareApi';
import { DividendPerShareItem } from '../../api/dividendPerShareApi';
import { waitOpts } from '@/test/utils';

vi.mock('../../api/dividendPerShareApi', () => ({
  fetchDividendPerShareBatch: vi.fn(),
}));

const mockFetch = api.fetchDividendPerShareBatch as Mock;

const makeItem = (code: string, status: DividendPerShareItem['status'], div: number | null): DividendPerShareItem => ({
  security_code: code,
  dividend_per_share: div,
  status,
  fetched_at: status === 'ok' || status === 'zero' ? '2025-01-01T00:00:00Z' : null,
  is_stale: false,
});

const flushAsyncUpdates = async () => {
  await act(async () => {
    await Promise.resolve();
    await Promise.resolve();
  });
};

describe('useDividendBatch', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.restoreAllMocks();
  });

  it('zero と error のステータスが区別され、dividendPerShareMap には含まれない', async () => {
    mockFetch.mockResolvedValue([
      makeItem('1234', 'zero', 0),
      makeItem('5678', 'error', null),
    ]);

    const { result } = renderHook(() => useDividendBatch(['1234', '5678'], true));

    await waitFor(() => expect(result.current.loading).toBe(false), waitOpts);

    expect(result.current.dividendStatusMap.get('1234')).toBe('zero');
    expect(result.current.dividendStatusMap.get('5678')).toBe('error');
    expect(result.current.dividendPerShareMap.has('1234')).toBe(false);
    expect(result.current.dividendPerShareMap.has('5678')).toBe(false);
  });

  it('ok の配当は dividendPerShareMap に格納され、ok+zero が fetchedCount に加算される', async () => {
    mockFetch.mockResolvedValue([
      makeItem('7203', 'ok', 50),
      makeItem('6758', 'zero', 0),
    ]);

    const { result } = renderHook(() => useDividendBatch(['7203', '6758'], true));

    await waitFor(() => expect(result.current.loading).toBe(false));

    expect(result.current.dividendPerShareMap.get('7203')).toBe(50);
    expect(result.current.dividendPerShareMap.has('6758')).toBe(false);
    expect(result.current.fetchedCount).toBe(2);
    expect(result.current.totalCount).toBe(2);
  });

  it('pending が残る場合は fetchedCount < totalCount になる（再試行条件）', async () => {
    mockFetch.mockResolvedValue([
      makeItem('1111', 'ok', 100),
      makeItem('2222', 'pending', null),
    ]);

    const { result } = renderHook(() => useDividendBatch(['1111', '2222'], true));

    await waitFor(() => expect(result.current.loading).toBe(false));

    expect(result.current.fetchedCount).toBe(1);
    expect(result.current.totalCount).toBe(2);
    expect(result.current.dividendStatusMap.get('2222')).toBe('pending');
  });

  it('pending が返る場合はリトライタイマー経由で再フェッチする', async () => {
    vi.useFakeTimers();

    mockFetch
      .mockResolvedValueOnce([makeItem('7203', 'pending', null)])
      .mockResolvedValueOnce([makeItem('7203', 'ok', 50)]);

    const { result } = renderHook(() => useDividendBatch(['7203'], true));

    await flushAsyncUpdates();

    expect(mockFetch).toHaveBeenCalledTimes(1);
    expect(result.current.dividendStatusMap.get('7203')).toBe('pending');

    await act(async () => {
      await vi.runOnlyPendingTimersAsync();
    });
    await flushAsyncUpdates();

    expect(mockFetch).toHaveBeenCalledTimes(2);
    expect(result.current.dividendStatusMap.get('7203')).toBe('ok');
    expect(result.current.dividendPerShareMap.get('7203')).toBe(50);
  });

  it('pending が続いて maxRetries に達すると再フェッチを停止する', async () => {
    vi.useFakeTimers();

    mockFetch.mockResolvedValue([makeItem('7203', 'pending', null)]);

    renderHook(() => useDividendBatch(['7203'], true));

    await flushAsyncUpdates();
    expect(mockFetch).toHaveBeenCalledTimes(1);

    for (let i = 0; i < 4; i += 1) {
      await act(async () => {
        await vi.runOnlyPendingTimersAsync();
      });
      await flushAsyncUpdates();
    }

    expect(mockFetch).toHaveBeenCalledTimes(5);

    await act(async () => {
      await vi.runAllTimersAsync();
    });
    await flushAsyncUpdates();

    expect(mockFetch).toHaveBeenCalledTimes(5);
  });

  it('pending 時は重複排除したコードで件数とリトライ上限を計算する', async () => {
    vi.useFakeTimers();

    mockFetch.mockResolvedValue([
      makeItem('3001', 'pending', null),
      makeItem('3002', 'pending', null),
    ]);

    const codes = ['3002', '3001', '3001'];
    const { result } = renderHook(() => useDividendBatch(codes, true));

    await flushAsyncUpdates();

    expect(mockFetch).toHaveBeenCalledWith(['3001', '3002']);
    expect(result.current.totalCount).toBe(2);
    expect(result.current.fetchedCount).toBe(0);

    // 2銘柄は単位修正により maxRetries=5（初回 + 5回 = 6呼び出し）
    for (let i = 0; i < 5; i += 1) {
      await act(async () => {
        await vi.runOnlyPendingTimersAsync();
      });
      await flushAsyncUpdates();
    }

    expect(mockFetch).toHaveBeenCalledTimes(6);

    await act(async () => {
      await vi.runAllTimersAsync();
    });
    await flushAsyncUpdates();

    expect(mockFetch).toHaveBeenCalledTimes(6);
  });

  it('enabled=false のときは状態がクリアされ API が呼ばれない', async () => {
    // 安定参照を使う（インライン配列だと毎レンダーで新参照が生成され無限ループになる）
    const codes = ['1234'];
    const { result } = renderHook(() => useDividendBatch(codes, false));

    // enabled=false なので fetch は呼ばれず loading も false のまま
    expect(mockFetch).not.toHaveBeenCalled();
    expect(result.current.loading).toBe(false);
    expect(result.current.dividendPerShareMap.size).toBe(0);
    expect(result.current.dividendStatusMap.size).toBe(0);
    expect(result.current.totalCount).toBe(0);
  });

  it('enabled=false に切り替わるとマップをリセットして再取得しない', async () => {
    mockFetch.mockResolvedValue([makeItem('1234', 'ok', 45)]);

    let enabled = true;
    const codes = ['1234'];
    const { result, rerender } = renderHook(() => useDividendBatch(codes, enabled));

    await waitFor(() => expect(result.current.loading).toBe(false), waitOpts);
    expect(result.current.dividendPerShareMap.get('1234')).toBe(45);
    expect(result.current.dividendStatusMap.get('1234')).toBe('ok');

    enabled = false;
    rerender();

    await waitFor(() => expect(result.current.dividendPerShareMap.size).toBe(0), waitOpts);
    expect(result.current.dividendStatusMap.size).toBe(0);
    expect(result.current.fetchedCount).toBe(0);
    expect(result.current.totalCount).toBe(0);
    expect(mockFetch).toHaveBeenCalledTimes(1);
  });

  it('securityCodes が空配列になるとマップをリセットする', async () => {
    mockFetch.mockResolvedValue([makeItem('7203', 'ok', 50)]);

    const { result, rerender } = renderHook(
      ({ codes }: { codes: string[] }) => useDividendBatch(codes, true),
      { initialProps: { codes: ['7203'] } }
    );

    await waitFor(() => expect(result.current.loading).toBe(false), waitOpts);
    expect(result.current.dividendPerShareMap.get('7203')).toBe(50);

    rerender({ codes: [] });

    await waitFor(() => expect(result.current.dividendPerShareMap.size).toBe(0), waitOpts);
    expect(result.current.dividendStatusMap.size).toBe(0);
    expect(result.current.fetchedCount).toBe(0);
    expect(result.current.totalCount).toBe(0);
    expect(mockFetch).toHaveBeenCalledTimes(1);
  });

  it('pending 後に enabled を切り替えると再フェッチで ok に更新できる', async () => {
    // 1回目: pending あり
    mockFetch.mockResolvedValueOnce([
      makeItem('3001', 'ok', 200),
      makeItem('3002', 'pending', null),
    ]);
    // 2回目: 全件 ok
    mockFetch.mockResolvedValueOnce([
      makeItem('3001', 'ok', 200),
      makeItem('3002', 'ok', 100),
    ]);

    let enabled = true;
    const codes = ['3001', '3002']; // 安定参照（enabled=false 時のループ防止）
    const { result, rerender } = renderHook(() =>
      useDividendBatch(codes, enabled)
    );

    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.fetchedCount).toBe(1);
    expect(result.current.dividendStatusMap.get('3002')).toBe('pending');

    // enabled=false → true でコードセットをリセットし再フェッチを誘発
    enabled = false;
    rerender();
    await waitFor(() => expect(result.current.dividendPerShareMap.size).toBe(0));

    enabled = true;
    rerender();
    await waitFor(() => expect(result.current.fetchedCount).toBe(2), { timeout: 5000 });
    expect(mockFetch).toHaveBeenCalledTimes(2);
    expect(result.current.dividendPerShareMap.get('3002')).toBe(100);
  });

  it('アンマウント後はリトライタイマーがクリアされる', async () => {
    const clearTimeoutSpy = vi.spyOn(globalThis, 'clearTimeout');

    mockFetch.mockResolvedValueOnce([
      makeItem('4001', 'pending', null),
    ]);

    const codes = ['4001']; // 安定参照
    const { unmount, result } = renderHook(() => useDividendBatch(codes, true));

    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(mockFetch).toHaveBeenCalledTimes(1);

    // アンマウント時にクリーンアップ関数が clearTimeout を呼ぶことを確認
    unmount();
    expect(clearTimeoutSpy).toHaveBeenCalled();
  });

  it('同じ銘柄セットの再レンダーでは不要な再フェッチをしない', async () => {
    mockFetch.mockResolvedValue([makeItem('5001', 'ok', 50)]);

    // コードが固定の場合（useMemo等で安定参照を使う場合と等価）
    const codes = ['5001'];
    const { rerender, result } = renderHook(() => useDividendBatch(codes, true));

    await waitFor(() => expect(result.current.loading).toBe(false));
    const callCount = mockFetch.mock.calls.length;

    // 同じ参照で再レンダー
    rerender();
    rerender();

    // 再フェッチされていない
    expect(mockFetch).toHaveBeenCalledTimes(callCount);
  });

  it('同じ銘柄集合だが配列参照が変わっても再フェッチしない', async () => {
    mockFetch.mockResolvedValue([makeItem('6001', 'ok', 80)]);

    let codes = ['6001'];
    const { rerender, result } = renderHook(({ c }: { c: string[] }) => useDividendBatch(c, true), {
      initialProps: { c: codes },
    });

    await waitFor(() => expect(result.current.loading).toBe(false), waitOpts);
    const callCount = mockFetch.mock.calls.length;

    // 内容は同じだが新しい配列参照に変更
    codes = ['6001'];
    rerender({ c: codes });
    rerender({ c: codes });

    // codesKey が同じなので再フェッチしない
    expect(mockFetch).toHaveBeenCalledTimes(callCount);
  });

  it('N=2ではmaxRetriesが5にスケールする（単位修正の検証）', async () => {
    vi.useFakeTimers();

    mockFetch.mockResolvedValue([
      makeItem('7001', 'pending', null),
      makeItem('7002', 'pending', null),
    ]);

    // 単位不一致の旧式では銘柄数によらず4回で停止するが、修正後は
    // ceil(2*12*1000/15000)+3=5回リトライする
    renderHook(() => useDividendBatch(['7001', '7002'], true));

    await flushAsyncUpdates();
    expect(mockFetch).toHaveBeenCalledTimes(1);

    // 旧上限の4回を超えてもポーリングが継続すること
    for (let i = 0; i < 4; i += 1) {
      await act(async () => {
        await vi.runOnlyPendingTimersAsync();
      });
      await flushAsyncUpdates();
    }
    expect(mockFetch).toHaveBeenCalledTimes(5);

    // 5回目のリトライ後（計6呼び出し）で停止すること
    await act(async () => {
      await vi.runOnlyPendingTimersAsync();
    });
    await flushAsyncUpdates();
    expect(mockFetch).toHaveBeenCalledTimes(6);

    await act(async () => {
      await vi.runAllTimersAsync();
    });
    await flushAsyncUpdates();
    expect(mockFetch).toHaveBeenCalledTimes(6);
  });

  it('N=100ではmaxRetriesが83にスケールする', async () => {
    vi.useFakeTimers();

    const codes = Array.from({ length: 100 }, (_, i) => String(8000 + i));
    mockFetch.mockResolvedValue(codes.map((code) => makeItem(code, 'pending', null)));

    // ceil(100*12*1000/15000)+3=83回リトライ（初回 + 83回 = 84呼び出し）
    renderHook(() => useDividendBatch(codes, true));

    await flushAsyncUpdates();
    expect(mockFetch).toHaveBeenCalledTimes(1);

    // 旧上限の4回を超えてもポーリングが継続すること
    for (let i = 0; i < 4; i += 1) {
      await act(async () => {
        await vi.runOnlyPendingTimersAsync();
      });
      await flushAsyncUpdates();
    }
    expect(mockFetch).toHaveBeenCalledTimes(5);

    for (let i = 0; i < 79; i += 1) {
      await act(async () => {
        await vi.runOnlyPendingTimersAsync();
      });
      await flushAsyncUpdates();
    }
    expect(mockFetch).toHaveBeenCalledTimes(84);

    await act(async () => {
      await vi.runAllTimersAsync();
    });
    await flushAsyncUpdates();
    expect(mockFetch).toHaveBeenCalledTimes(84);
  });

  it('上限キャップによりN=200でも100回リトライで停止する', async () => {
    vi.useFakeTimers();

    const codes = Array.from({ length: 200 }, (_, i) => String(9000 + i));
    mockFetch.mockResolvedValue(codes.map((code) => makeItem(code, 'pending', null)));

    // キャップなしの計算値は163回だが、上限100で打ち切られる（初回 + 100回 = 101呼び出し）
    renderHook(() => useDividendBatch(codes, true));

    await flushAsyncUpdates();
    expect(mockFetch).toHaveBeenCalledTimes(1);

    for (let i = 0; i < 100; i += 1) {
      await act(async () => {
        await vi.runOnlyPendingTimersAsync();
      });
      await flushAsyncUpdates();
    }
    expect(mockFetch).toHaveBeenCalledTimes(101);

    await act(async () => {
      await vi.runAllTimersAsync();
    });
    await flushAsyncUpdates();
    expect(mockFetch).toHaveBeenCalledTimes(101);
  });

  it('通信失敗時のリトライはpending再確認と分離され3回で停止する', async () => {
    vi.useFakeTimers();

    // N=100ならpending上限は83回だが、通信失敗は別枠の3回で打ち切られる
    const codes = Array.from({ length: 100 }, (_, i) => String(8100 + i));
    mockFetch.mockRejectedValue(new Error('network error'));

    renderHook(() => useDividendBatch(codes, true));

    await flushAsyncUpdates();
    expect(mockFetch).toHaveBeenCalledTimes(1);

    for (let i = 0; i < 10; i += 1) {
      await act(async () => {
        await vi.runOnlyPendingTimersAsync();
      });
      await flushAsyncUpdates();
    }

    // 初回 + 3回 = 4呼び出しで停止する
    expect(mockFetch).toHaveBeenCalledTimes(4);

    await act(async () => {
      await vi.runAllTimersAsync();
    });
    await flushAsyncUpdates();
    expect(mockFetch).toHaveBeenCalledTimes(4);
  });

  it('pending再確認は通信失敗の回数を消費しない', async () => {
    vi.useFakeTimers();

    mockFetch
      .mockResolvedValueOnce([makeItem('7101', 'pending', null)])
      .mockRejectedValueOnce(new Error('network error'))
      .mockRejectedValueOnce(new Error('network error'))
      .mockResolvedValue([makeItem('7101', 'pending', null)]);

    // N=1のpending上限は4回。途中の通信失敗2回がpending枠を消費しなければ
    // pendingリトライは4回使い切る（初回 + 通信失敗2回 + pending4回 = 7呼び出しで停止する）
    // 共有カウンタの旧実装なら通信失敗が枠を消費して5呼び出しで止まる
    renderHook(() => useDividendBatch(['7101'], true));

    await flushAsyncUpdates();
    expect(mockFetch).toHaveBeenCalledTimes(1);

    for (let i = 0; i < 6; i += 1) {
      await act(async () => {
        await vi.runOnlyPendingTimersAsync();
      });
      await flushAsyncUpdates();
    }
    expect(mockFetch).toHaveBeenCalledTimes(7);

    await act(async () => {
      await vi.runAllTimersAsync();
    });
    await flushAsyncUpdates();
    expect(mockFetch).toHaveBeenCalledTimes(7);
  });
});
