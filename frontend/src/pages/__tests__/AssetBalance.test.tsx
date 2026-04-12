import { render, screen, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { AssetBalanceData } from '@/types/api';
import { AssetBalanceInfo } from '../AssetBalance';
import { waitOpts } from '@/test/utils';

// SecurityCodeLinkのモック
vi.mock('@/components/atoms/SecurityCodeLink', () => ({
  SecurityCodeLink: ({ value }: { value: string }) => <span>{value}</span>,
}));

// ユーティリティ関数のモック
vi.mock('@/lib/utils/formatters', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/lib/utils/formatters')>();
  return {
    ...actual,
    formatCurrency: vi.fn().mockImplementation((value: number) => `¥${value.toLocaleString()}`),
    formatNumber: vi.fn().mockImplementation((value: number) => value.toLocaleString()),
    formatPercentageValue: vi.fn().mockImplementation((value: number) => `${value.toFixed(2)}%`),
    safeAdd: vi.fn().mockImplementation((a: number, b: number) => a + b),
  };
});

function makeAssetBalanceData(overrides: Partial<AssetBalanceData> = {}): AssetBalanceData {
  return {
    id: 'asset-balance-1',
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    security_code: '7203',
    security_name: 'トヨタ自動車',
    shares: 100,
    executing_shares: 0,
    average_purchase_price: 2500,
    total_purchase_amount: 250000,
    current_price: 2600,
    daily_change: 50,
    market_value: 260000,
    profit_loss_rate: 4.0,
    ...overrides,
  };
}

const mockAssetBalanceData: AssetBalanceData[] = [
  makeAssetBalanceData(),
  makeAssetBalanceData({
    id: 'asset-balance-2',
    security_code: '6758',
    security_name: 'ソニーグループ',
    shares: 50,
    average_purchase_price: 12000,
    total_purchase_amount: 600000,
    current_price: 11500,
    daily_change: -100,
    market_value: 575000,
    profit_loss_rate: -4.17,
  }),
];


describe('AssetBalanceInfo', { timeout: 20000 }, () => {
  const defaultProps = {
    assetBalanceData: mockAssetBalanceData,
    filteredData: mockAssetBalanceData,
    searchQuery: '',
    onClearFilter: vi.fn(),
    dividendPerShareMap: new Map<string, number>(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('ポートフォリオサマリーが表示される', async () => {
    render(<AssetBalanceInfo {...defaultProps} />);
    await waitFor(() => {
      expect(screen.getByTestId('asset-portfolio-summary')).toBeInTheDocument();
    }, waitOpts);
  });

  it('合計取得総額が正しく計算される', async () => {
    render(<AssetBalanceInfo {...defaultProps} />);
    await waitFor(() => {
      // 250,000 + 600,000 = 850,000
      expect(screen.getByText(/850,000/)).toBeInTheDocument();
    }, waitOpts);
  });

  it('銘柄名が表示される', async () => {
    render(<AssetBalanceInfo {...defaultProps} />);
    await waitFor(() => {
      expect(screen.getByText('トヨタ自動車')).toBeInTheDocument();
      expect(screen.getByText('ソニーグループ')).toBeInTheDocument();
    }, waitOpts);
  });

  it('空のデータの場合は空状態が表示される', async () => {
    render(
      <AssetBalanceInfo
        assetBalanceData={[]}
        filteredData={[]}
        searchQuery=""
        onClearFilter={vi.fn()}
        dividendPerShareMap={new Map()}
      />
    );
    await waitFor(() => {
      expect(screen.getByText('資産管理データがありません')).toBeInTheDocument();
    }, waitOpts);
  });

  it('絞り込み中はフィルタデータのみ表示される', async () => {
    const filteredData = [mockAssetBalanceData[0]];
    render(
      <AssetBalanceInfo
        assetBalanceData={mockAssetBalanceData}
        filteredData={filteredData}
        searchQuery="7203"
        onClearFilter={vi.fn()}
        dividendPerShareMap={new Map()}
      />
    );
    await waitFor(() => {
      expect(screen.getByTestId('asset-portfolio-summary')).toBeInTheDocument();
      expect(screen.getByText(/絞り込み中/)).toBeInTheDocument();
    }, waitOpts);
  });

  it('絞り込みで該当なしの場合は適切なメッセージが表示される', async () => {
    render(
      <AssetBalanceInfo
        assetBalanceData={mockAssetBalanceData}
        filteredData={[]}
        searchQuery="9999"
        onClearFilter={vi.fn()}
        dividendPerShareMap={new Map()}
      />
    );
    await waitFor(() => {
      expect(screen.getByText('該当する銘柄がありません')).toBeInTheDocument();
    }, waitOpts);
  });

  it('銘柄別構成比が表示される', async () => {
    render(<AssetBalanceInfo {...defaultProps} />);
    await waitFor(() => {
      expect(screen.getByText('銘柄別構成比')).toBeInTheDocument();
    }, waitOpts);
  });
});
