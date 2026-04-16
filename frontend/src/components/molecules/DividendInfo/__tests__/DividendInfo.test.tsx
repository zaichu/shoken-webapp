import type { ComponentProps } from 'react';
import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { DividendInfo } from '../DividendInfo';

function createMockDividendBatchResult(
  overrides: Partial<{
    dividendPerShareMap: Map<string, number>;
    dividendStatusMap: Map<string, 'ok' | 'zero' | 'pending' | 'error'>;
    loading: boolean;
    fetchedCount: number;
    totalCount: number;
  }> = {}
) {
  return {
    dividendPerShareMap: new Map<string, number>(),
    dividendStatusMap: new Map<string, 'ok' | 'zero' | 'pending' | 'error'>(),
    loading: false,
    fetchedCount: 0,
    totalCount: 0,
    ...overrides,
  };
}

vi.mock('@/features/jquants/hooks/useDividendBatch', () => ({
  useDividendBatch: vi.fn(() => createMockDividendBatchResult()),
}));

vi.mock('@/features/assetBalance/hooks/useAssetBalance', () => ({
  useAssetBalance: vi.fn(() => ({
    assetBalanceData: [],
    getAssetBalanceByCode: vi.fn(() => undefined),
  })),
}));

const emptySummary: ComponentProps<typeof DividendInfo>['summary'] = [];

describe('DividendInfo', () => {
  it('searchQuery が空のとき何も描画しない', () => {
    const { container } = render(
      <DividendInfo searchQuery="" summary={emptySummary} />
    );
    expect(container.firstChild).toBeNull();
  });

  it('searchQuery があるとき standalone モードで配当シミュレーションを表示する', () => {
    render(
      <DividendInfo searchQuery="1234" summary={emptySummary} />
    );
    expect(screen.getByText('配当シミュレーション')).toBeInTheDocument();
  });

  it('standalone モードで入力フィールドを表示する', () => {
    render(
      <DividendInfo searchQuery="1234" summary={emptySummary} />
    );
    expect(screen.getByLabelText('平均取得価格')).toBeInTheDocument();
    expect(screen.getByLabelText('保有数量(株)')).toBeInTheDocument();
    expect(screen.getByLabelText('一株配当')).toBeInTheDocument();
  });

  it('embedded モードで配当シミュレーション見出しを表示しない', () => {
    render(
      <DividendInfo searchQuery="1234" summary={emptySummary} embedded />
    );
    expect(screen.queryByText('配当シミュレーション')).not.toBeInTheDocument();
  });

  it('embedded モードで平均取得価格・保有数量・一株配当のラベルを表示する', () => {
    render(
      <DividendInfo searchQuery="1234" summary={emptySummary} embedded />
    );
    expect(screen.getByText('平均取得価格')).toBeInTheDocument();
    expect(screen.getByText('保有数量(株)')).toBeInTheDocument();
    expect(screen.getByText('一株配当')).toBeInTheDocument();
  });

  it('embedded モードでデータなし時に --- を表示する', () => {
    render(
      <DividendInfo searchQuery="1234" summary={emptySummary} embedded />
    );
    const dashes = screen.getAllByText('---');
    expect(dashes.length).toBeGreaterThanOrEqual(2);
  });

  it('embedded モードで一株配当取得中に取得中... を表示する', async () => {
    const { useDividendBatch } = await import('@/features/jquants/hooks/useDividendBatch');
    vi.mocked(useDividendBatch).mockReturnValueOnce(createMockDividendBatchResult({
      loading: true,
    }));

    render(
      <DividendInfo searchQuery="1234" summary={emptySummary} embedded />
    );
    expect(screen.getByText('取得中...')).toBeInTheDocument();
  });

  it('standalone モードで一株配当取得中に入力欄が disabled になりプレースホルダーを表示する', async () => {
    const { useDividendBatch } = await import('@/features/jquants/hooks/useDividendBatch');
    vi.mocked(useDividendBatch).mockReturnValueOnce(createMockDividendBatchResult({
      loading: true,
    }));

    render(
      <DividendInfo searchQuery="1234" summary={emptySummary} />
    );
    const input = screen.getByPlaceholderText('データ取得中...');
    expect(input).toBeDisabled();
  });

  it('standalone モードで一株配当と平均取得価格が揃ったとき配当利回りを計算して表示する', async () => {
    const { useDividendBatch } = await import('@/features/jquants/hooks/useDividendBatch');
    vi.mocked(useDividendBatch).mockReturnValueOnce(createMockDividendBatchResult({
      dividendPerShareMap: new Map([['1234', 30]]),
    }));
    const { useAssetBalance } = await import('@/features/assetBalance/hooks/useAssetBalance');
    vi.mocked(useAssetBalance).mockReturnValueOnce({
      assetBalanceData: [],
      getAssetBalanceByCode: vi.fn(() => ({
        average_purchase_price: 1000,
        shares: 100,
        security_code: '1234',
        security_name: 'テスト株式会社',
        total_purchase_price: 100000,
      })),
    });

    render(
      <DividendInfo searchQuery="1234" summary={emptySummary} />
    );
    // dividendYield = 30 / 1000 * 100 = 3%, annualDividendAmount = 100 * 30 = ¥ 3,000
    expect(screen.getByText('¥ 3,000 (3.00%)')).toBeInTheDocument();
  });

  it('summary データがある場合 embedded モードで集計金額を表示する', () => {
    const summary: ComponentProps<typeof DividendInfo>['summary'] = [
      {
        filter: '1234',
        dividends_before_tax: 10000,
        taxes: 2000,
        net_amount_received: 8000,
      },
    ];

    render(
      <DividendInfo
        searchQuery="1234"
        summary={summary}
        embedded
      />
    );
    // 配当金額ラベルが表示される
    expect(screen.getByText('配当金額 (配当利回り)')).toBeInTheDocument();
    expect(screen.getByText('受取金額 (累積利回り)')).toBeInTheDocument();
    expect(screen.getByText('税額')).toBeInTheDocument();
  });
});
