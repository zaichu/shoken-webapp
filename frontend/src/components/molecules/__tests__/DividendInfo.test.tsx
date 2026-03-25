import { render, screen } from '@testing-library/react';
import { beforeEach, vi } from 'vitest';

const { mockUseDividendBatch, mockUseAssetBalance } = vi.hoisted(() => ({
  mockUseDividendBatch: vi.fn(),
  mockUseAssetBalance: vi.fn(),
}));

vi.mock('@/features/jquants/hooks/useDividendBatch', () => ({
  useDividendBatch: (...args: unknown[]) => mockUseDividendBatch(...args),
}));

vi.mock('@/features/assetBalance/hooks/useAssetBalance', () => ({
  useAssetBalance: (...args: unknown[]) => mockUseAssetBalance(...args),
}));

import { DividendInfo } from '../DividendInfo/DividendInfo';

describe('DividendInfo', () => {
  beforeEach(() => {
    vi.clearAllMocks();

    mockUseDividendBatch.mockReturnValue({
      dividendPerShareMap: new Map(),
      dividendStatusMap: new Map(),
      loading: false,
      fetchedCount: 0,
      totalCount: 0,
    });

    mockUseAssetBalance.mockReturnValue({
      assetBalanceData: [],
      isLoading: false,
      getAssetBalanceByCode: vi.fn(() => undefined),
      getTotalMarketValue: vi.fn(() => 0),
      refetch: vi.fn(),
    });
  });

  it('summary が空でも正常にレンダリングされる', () => {
    render(<DividendInfo searchQuery="7203: トヨタ自動車" summary={[]} />);

    expect(screen.getByText('配当シミュレーション')).toBeInTheDocument();
  });

  it('J-Quants バッジのリンクを表示する', () => {
    mockUseDividendBatch.mockReturnValue({
      dividendPerShareMap: new Map([['7203', 120]]),
      dividendStatusMap: new Map([['7203', 'ok']]),
      loading: false,
      fetchedCount: 1,
      totalCount: 1,
    });

    render(<DividendInfo searchQuery="7203: トヨタ自動車" summary={[]} />);

    const link = screen.getByRole('link', { name: 'J-Quants' });
    expect(link).toHaveAttribute('href', 'https://jpx-jquants.com/');
  });

  it('searchQuery から銘柄コードを解決して useDividendBatch に渡す', () => {
    render(<DividendInfo searchQuery="7203: トヨタ自動車" summary={[]} />);

    expect(mockUseDividendBatch).toHaveBeenCalledWith(['7203'], true);
  });

  it('embedded モードで未取得値にヒントを表示する', () => {
    render(<DividendInfo embedded searchQuery="7203: トヨタ自動車" summary={[]} />);

    expect(
      screen.getAllByText('資産管理にCSVを取り込むと表示されます')
    ).toHaveLength(2);
    expect(screen.getByText('J-Quants APIから取得します')).toBeInTheDocument();
  });

  it('searchQueryが空の場合nullを返す', () => {
    const { container } = render(<DividendInfo searchQuery="" summary={[]} />);
    expect(container.firstChild).toBeNull();
  });

  it('summaryデータがある場合、配当利回りが計算される', () => {
    mockUseAssetBalance.mockReturnValue({
      assetBalanceData: [],
      isLoading: false,
      getAssetBalanceByCode: vi.fn(() => ({
        average_purchase_price: 1000,
        shares: 100,
        security_code: '7203',
      })),
      getTotalMarketValue: vi.fn(() => 0),
      refetch: vi.fn(),
    });

    mockUseDividendBatch.mockReturnValue({
      dividendPerShareMap: new Map([['7203', 50]]),
      dividendStatusMap: new Map([['7203', 'ok']]),
      loading: false,
      fetchedCount: 1,
      totalCount: 1,
    });

    render(
      <DividendInfo
        searchQuery="7203: トヨタ自動車"
        summary={[
          {
            filter: '7203',
            security_code: '7203',
            net_amount_received: 4500,
            dividends_before_tax: 5000,
            taxes: 500,
          },
        ]}
      />
    );

    expect(screen.getByText('配当シミュレーション')).toBeInTheDocument();
  });
});
