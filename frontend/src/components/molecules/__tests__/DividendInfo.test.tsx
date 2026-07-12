import { render, screen } from '@testing-library/react';
import { beforeEach, vi } from 'vitest';

const { mockUseDividendBatch, mockUseAssetBalance } = vi.hoisted(() => ({
  mockUseDividendBatch: vi.fn(),
  mockUseAssetBalance: vi.fn(),
}));

vi.mock('@/features/dividendPerShare/hooks/useDividendBatch', () => ({
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
      refetch: vi.fn(),
    });
  });

  it('summary が空でも正常にレンダリングされる', () => {
    render(<DividendInfo searchQuery="7203: トヨタ自動車" summary={[]} />);

    expect(screen.getByText('配当シミュレーション')).toBeInTheDocument();
  });

  it('一株配当データが取得された場合、その値が入力欄に反映される', () => {
    mockUseDividendBatch.mockReturnValue({
      dividendPerShareMap: new Map([['7203', 120]]),
      dividendStatusMap: new Map([['7203', 'ok']]),
      loading: false,
      fetchedCount: 1,
      totalCount: 1,
    });

    render(<DividendInfo searchQuery="7203: トヨタ自動車" summary={[]} />);

    expect(screen.getByDisplayValue('120')).toBeInTheDocument();
  });

  it('searchQuery から銘柄コードを解決して useDividendBatch に渡す', () => {
    render(<DividendInfo searchQuery="7203: トヨタ自動車" summary={[]} />);

    expect(mockUseDividendBatch).toHaveBeenCalledWith(['7203'], true);
  });

  it('searchQuery から銘柄コードを解決して useAssetBalance に渡す', () => {
    render(<DividendInfo searchQuery="7203: トヨタ自動車" summary={[]} />);

    expect(mockUseAssetBalance).toHaveBeenCalledWith({ enabled: true, securityCode: '7203' });
  });

  it('embedded モードで未取得値にヒントを表示する', () => {
    render(<DividendInfo embedded searchQuery="7203: トヨタ自動車" summary={[]} />);

    expect(
      screen.getAllByText('資産管理にCSVを取り込むと表示されます')
    ).toHaveLength(2);
    expect(screen.getByText('自動で取得されます')).toBeInTheDocument();
  });

  it('searchQueryが空の場合nullを返す', () => {
    const { container } = render(<DividendInfo searchQuery="" summary={[]} />);
    expect(container.firstChild).toBeNull();
  });

  it('embeddedモードでapiLoading=trueのとき取得中...が表示される', () => {
    mockUseDividendBatch.mockReturnValue({
      dividendPerShareMap: new Map(),
      dividendStatusMap: new Map(),
      loading: true,
      fetchedCount: 0,
      totalCount: 1,
    });

    render(<DividendInfo embedded searchQuery="7203: トヨタ自動車" summary={[]} />);

    expect(screen.getByText('取得中...')).toBeInTheDocument();
  });

  it('embeddedモードでdividendPerShareが定義済みの場合---が表示されない', () => {
    mockUseDividendBatch.mockReturnValue({
      dividendPerShareMap: new Map([['7203', 100]]),
      dividendStatusMap: new Map([['7203', 'ok']]),
      loading: false,
      fetchedCount: 1,
      totalCount: 1,
    });

    render(<DividendInfo embedded searchQuery="7203: トヨタ自動車" summary={[]} />);

    // 一株配当が定義されているので---は表示されない（自動で取得されますのヒントも消える）
    expect(screen.queryByText('自動で取得されます')).not.toBeInTheDocument();
  });

  it('embeddedモードでsummaryが非ゼロの場合、配当利回りが表示される', () => {
    mockUseAssetBalance.mockReturnValue({
      assetBalanceData: [],
      isLoading: false,
      getAssetBalanceByCode: vi.fn(() => ({
        average_purchase_price: 1000,
        shares: 100,
        security_code: '7203',
      })),
      refetch: vi.fn(),
    });

    render(
      <DividendInfo
        embedded
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

    // grossDividendReturnRate > 0 なので利回りのspanが表示される
    // totalInvestment = 1000 * 100 = 100000, totalDividendsBeforeTax = 5000
    // grossDividendReturnRate = 5000 / 100000 * 100 = 5.00%
    expect(screen.getByText('(5.00%)')).toBeInTheDocument();
  });

  it('standaloneモードでapiLoading=trueのときplaceholderが設定される', () => {
    mockUseDividendBatch.mockReturnValue({
      dividendPerShareMap: new Map(),
      dividendStatusMap: new Map(),
      loading: true,
      fetchedCount: 0,
      totalCount: 1,
    });

    render(<DividendInfo searchQuery="7203: トヨタ自動車" summary={[]} />);

    expect(screen.getByPlaceholderText('データ取得中...')).toBeInTheDocument();
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
