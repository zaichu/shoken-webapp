import React from 'react';
import { render, screen, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { AssetBalanceData } from '@/lib/interfaces/assetBalance';
import { AssetBalanceInfo } from '../AssetBalance';

// SecurityCodeLinkのモック
vi.mock('@/components/atoms/SecurityCodeLink', () => ({
  SecurityCodeLink: ({ value }: { value: string }) => <span>{value}</span>,
}));

// ユーティリティ関数のモック
vi.mock('@/lib/utils/formatters', () => ({
  formatCurrency: vi.fn().mockImplementation((value: number) => `¥${value.toLocaleString()}`),
  formatNumber: vi.fn().mockImplementation((value: number) => value.toLocaleString()),
  safeAdd: vi.fn().mockImplementation((a: number, b: number) => a + b),
}));

const mockAssetBalanceData: AssetBalanceData[] = [
  {
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
  },
  {
    security_code: '6758',
    security_name: 'ソニーグループ',
    shares: 50,
    executing_shares: 0,
    average_purchase_price: 12000,
    total_purchase_amount: 600000,
    current_price: 11500,
    daily_change: -100,
    market_value: 575000,
    profit_loss_rate: -4.17,
  },
];

describe('AssetBalanceInfo', () => {
  const defaultProps = {
    assetBalanceData: mockAssetBalanceData,
    filteredData: mockAssetBalanceData,
    searchQuery: '',
    onClearFilter: vi.fn(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('ポートフォリオサマリーが表示される', async () => {
    render(<AssetBalanceInfo {...defaultProps} />);

    await waitFor(() => {
      expect(screen.getByTestId('asset-portfolio-summary')).toBeInTheDocument();
    });
  });

  it('合計取得総額が正しく計算される', async () => {
    render(<AssetBalanceInfo {...defaultProps} />);

    await waitFor(() => {
      // 250,000 + 600,000 = 850,000
      expect(screen.getByText(/850,000/)).toBeInTheDocument();
    });
  });

  it('銘柄名が表示される', async () => {
    render(<AssetBalanceInfo {...defaultProps} />);

    await waitFor(() => {
      expect(screen.getByText('トヨタ自動車')).toBeInTheDocument();
      expect(screen.getByText('ソニーグループ')).toBeInTheDocument();
    });
  });

  it('空のデータの場合は空状態が表示される', async () => {
    render(
      <AssetBalanceInfo
        assetBalanceData={[]}
        filteredData={[]}
        searchQuery=""
        onClearFilter={vi.fn()}
      />
    );

    await waitFor(() => {
      expect(screen.getByText('保有銘柄がありません')).toBeInTheDocument();
    });
  });

  it('絞り込み中はフィルタデータのみ表示される', async () => {
    const filteredData = [mockAssetBalanceData[0]];
    render(
      <AssetBalanceInfo
        assetBalanceData={mockAssetBalanceData}
        filteredData={filteredData}
        searchQuery="7203"
        onClearFilter={vi.fn()}
      />
    );

    await waitFor(() => {
      expect(screen.getByTestId('asset-portfolio-summary')).toBeInTheDocument();
      // 絞り込み中の表示
      expect(screen.getByText(/絞り込み中/)).toBeInTheDocument();
    });
  });

  it('絞り込みで該当なしの場合は適切なメッセージが表示される', async () => {
    render(
      <AssetBalanceInfo
        assetBalanceData={mockAssetBalanceData}
        filteredData={[]}
        searchQuery="9999"
        onClearFilter={vi.fn()}
      />
    );

    await waitFor(() => {
      expect(screen.getByText('該当する銘柄がありません')).toBeInTheDocument();
    });
  });

  it('銘柄別構成比が表示される', async () => {
    render(<AssetBalanceInfo {...defaultProps} />);

    await waitFor(() => {
      expect(screen.getByText('銘柄別構成比')).toBeInTheDocument();
    });
  });
});
