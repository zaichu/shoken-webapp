import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { DividendInfo } from '../DividendInfo';

vi.mock('@/features/jquants/hooks/useDividendBatch', () => ({
  useDividendBatch: vi.fn(() => ({
    dividendPerShareMap: new Map<string, number>(),
    loading: false,
  })),
}));

vi.mock('@/features/assetBalance/hooks/useAssetBalance', () => ({
  useAssetBalance: vi.fn(() => ({
    assetBalanceData: [],
    getAssetBalanceByCode: vi.fn(() => undefined),
  })),
}));

const emptySummary = [] as const;

describe('DividendInfo', () => {
  it('searchQuery が空のとき何も描画しない', () => {
    const { container } = render(
      <DividendInfo searchQuery="" summary={emptySummary as never} />
    );
    expect(container.firstChild).toBeNull();
  });

  it('searchQuery があるとき standalone モードで配当シミュレーションを表示する', () => {
    render(
      <DividendInfo searchQuery="1234" summary={emptySummary as never} />
    );
    expect(screen.getByText('配当シミュレーション')).toBeInTheDocument();
  });

  it('standalone モードで入力フィールドを表示する', () => {
    render(
      <DividendInfo searchQuery="1234" summary={emptySummary as never} />
    );
    expect(screen.getByLabelText('平均取得価格')).toBeInTheDocument();
    expect(screen.getByLabelText('保有数量(株)')).toBeInTheDocument();
    expect(screen.getByLabelText('一株配当')).toBeInTheDocument();
  });

  it('embedded モードで配当シミュレーション見出しを表示しない', () => {
    render(
      <DividendInfo searchQuery="1234" summary={emptySummary as never} embedded />
    );
    expect(screen.queryByText('配当シミュレーション')).not.toBeInTheDocument();
  });

  it('embedded モードで平均取得価格・保有数量・一株配当のラベルを表示する', () => {
    render(
      <DividendInfo searchQuery="1234" summary={emptySummary as never} embedded />
    );
    expect(screen.getByText('平均取得価格')).toBeInTheDocument();
    expect(screen.getByText('保有数量(株)')).toBeInTheDocument();
    expect(screen.getByText('一株配当')).toBeInTheDocument();
  });

  it('embedded モードでデータなし時に --- を表示する', () => {
    render(
      <DividendInfo searchQuery="1234" summary={emptySummary as never} embedded />
    );
    const dashes = screen.getAllByText('---');
    expect(dashes.length).toBeGreaterThanOrEqual(2);
  });

  it('embedded モードで一株配当取得中に取得中... を表示する', async () => {
    const { useDividendBatch } = await import('@/features/jquants/hooks/useDividendBatch');
    vi.mocked(useDividendBatch).mockReturnValueOnce({
      dividendPerShareMap: new Map(),
      loading: true,
    } as never);

    render(
      <DividendInfo searchQuery="1234" summary={emptySummary as never} embedded />
    );
    expect(screen.getByText('取得中...')).toBeInTheDocument();
  });

  it('summary データがある場合 embedded モードで集計金額を表示する', () => {
    const summary = [
      { dividends_before_tax: 10000, taxes: 2000, net_amount_received: 8000 },
    ];

    render(
      <DividendInfo
        searchQuery="1234"
        summary={summary as never}
        embedded
      />
    );
    // 配当金額ラベルが表示される
    expect(screen.getByText('配当金額 (配当利回り)')).toBeInTheDocument();
    expect(screen.getByText('受取金額 (累積利回り)')).toBeInTheDocument();
    expect(screen.getByText('税額')).toBeInTheDocument();
  });
});
