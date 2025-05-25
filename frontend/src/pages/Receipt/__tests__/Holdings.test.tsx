import React from 'react';
import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { Holdings } from '../Holdings';

// 依存関係のモック
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

vi.mock('@/hooks/common/useHoldingsStorage', () => ({
  useHoldingsStorage: () => ({
    saveHoldings: vi.fn(),
    clearHoldings: vi.fn(),
    getHoldingByCode: vi.fn(),
    lastUpdated: null,
  }),
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
  it('保有株一覧が正しく表示される', () => {
    render(<Holdings csvData={mockCsvData} />);

    expect(screen.getByText('保有株一覧')).toBeInTheDocument();
    expect(screen.getByTestId('receipt-table')).toBeInTheDocument();
  });

  it('空のデータでも正しく表示される', () => {
    render(<Holdings csvData={[]} />);

    expect(screen.getByText('保有株一覧')).toBeInTheDocument();
    expect(screen.getByText('データ数: 0')).toBeInTheDocument();
  });
});
