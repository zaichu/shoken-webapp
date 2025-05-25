import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { Holdings } from '../Receipt/Holdings';

// useHoldingsStorageのモック
const mockSaveHoldings = vi.fn();
const mockClearHoldings = vi.fn();
const mockGetHoldingByCode = vi.fn();

vi.mock('@/hooks/common/useHoldingsStorage', () => ({
  useHoldingsStorage: () => ({
    saveHoldings: mockSaveHoldings,
    clearHoldings: mockClearHoldings,
    getHoldingByCode: mockGetHoldingByCode,
    lastUpdated: '2024-01-01T00:00:00.000Z',
  }),
}));

// その他の依存関係のモック
vi.mock('@/components/templates/ReceiptTemplate', () => ({
  ReceiptTemplate: ({ children, title }: { children: React.ReactNode; title: string }) => (
    <div data-testid="receipt-template">
      <h1>{title}</h1>
      {children}
    </div>
  ),
}));

vi.mock('@/components/molecules/ReceiptHeader/ReceiptHeader', () => ({
  ReceiptHeader: ({ items }: { items: Array<{ title: string; value: number; format: (v: number) => string }> }) => (
    <div data-testid="receipt-header">
      {items.map((item, index) => (
        <div key={index}>
          {item.title}: {item.format(item.value)}
        </div>
      ))}
    </div>
  ),
}));

vi.mock('@/components/organisms/ReceiptTable/ReceiptTable', () => ({
  ReceiptTable: ({ data }: { data: Array<unknown> }) => (
    <div data-testid="receipt-table">
      データ数: {data.length}
    </div>
  ),
}));

const mockCsvData = [
  {
    '銘柄コード': '7203',
    '銘柄名': 'トヨタ自動車',
    '保有数量［株］': '100',
    '執行中［株］': '0',
    '平均取得価額［円］': '2,500.00',
    '取得総額［円］': '250,000',
    '現在値［円］': '2,600.0',
    '現在値（前日比）［円］': '50.0',
    '時価評価額［円］': '260,000',
    '評価損益［％］': '4.0',
  },
];

describe('Holdings', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('保有株データを正しく表示する', () => {
    render(<Holdings csvData={mockCsvData} />);

    expect(screen.getByText('保有株一覧')).toBeInTheDocument();
    expect(screen.getByTestId('receipt-table')).toBeInTheDocument();
  });

  it('CSVデータが空の場合でも正しく表示される', () => {
    render(<Holdings csvData={[]} />);

    expect(screen.getByText('保有株一覧')).toBeInTheDocument();
    expect(screen.getByText('データ数: 0')).toBeInTheDocument();
  });

  it('ローカルストレージに保存ボタンが機能する', async () => {
    const alertSpy = vi.spyOn(window, 'alert').mockImplementation(() => { });

    render(<Holdings csvData={mockCsvData} />);

    const saveButton = screen.getByText('ローカルストレージに保存');
    fireEvent.click(saveButton);

    await waitFor(() => {
      expect(mockSaveHoldings).toHaveBeenCalledWith([
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
          profit_loss_rate: 4,
        },
      ]);
    });

    alertSpy.mockRestore();
  });

  it('データが空の場合は保存ボタンが無効になる', () => {
    render(<Holdings csvData={[]} />);

    const saveButton = screen.getByText('ローカルストレージに保存');
    expect(saveButton).toBeDisabled();
  });

  it('削除ボタンが機能する', async () => {
    const confirmSpy = vi.spyOn(window, 'confirm').mockReturnValue(true);
    const alertSpy = vi.spyOn(window, 'alert').mockImplementation(() => { });

    render(<Holdings csvData={mockCsvData} />);

    const deleteButton = screen.getByText('保存データを削除');
    fireEvent.click(deleteButton);

    await waitFor(() => {
      expect(mockClearHoldings).toHaveBeenCalled();
    });

    confirmSpy.mockRestore();
    alertSpy.mockRestore();
  });
});
