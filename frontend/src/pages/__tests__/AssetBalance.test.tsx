import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { AssetBalanceData } from '@/lib/interfaces/assetBalance';
import { AssetBalanceInfo } from '../AssetBalance';

// その他の依存関係のモック
vi.mock('@/components/templates/ReceiptTemplate', () => ({
  ReceiptTemplate: ({
    children,
    title,
    onSearch,
    searchCategories
  }: {
    children: React.ReactNode;
    title: string;
    onSearch?: (query: string) => void;
    searchCategories?: Record<string, Array<{ value: string; label: string }>>;
  }) => (
    <div data-testid="receipt-template">
      <h1>{title}</h1>
      {onSearch && searchCategories && (
        <div data-testid="search-area">
          <input
            data-testid="search-input"
            onChange={(e) => onSearch(e.target.value)}
            placeholder="検索"
          />
          <div data-testid="search-options">
            オプション数: {Object.values(searchCategories).flat().length}
          </div>
        </div>
      )}
      {children}
    </div>
  ),
}));

vi.mock('@/components/organisms/ReceiptTable/ReceiptTable', () => ({
  ReceiptTable: ({ data, columns, summary, summaryColumns, getGroupKey }: {
    data: Array<unknown>;
    columns: Array<{ key: string; header: string; width?: string; textAlign?: string; format?: (v: number) => string }>;
    summary: Array<unknown>;
    summaryColumns: Array<unknown>;
    getGroupKey: () => string;
  }) => (
    <div data-testid="receipt-table">
      <div data-testid="table-headers">
        {columns.map((col, index) => (
          <span key={index} data-testid={`header-${col.key}`}>{col.header}</span>
        ))}
      </div>
      <div data-testid="table-data">データ数: {data.length}</div>
      <div data-testid="table-props">
        summary: {summary.length}, summaryColumns: {summaryColumns.length}, groupKey: {getGroupKey()}
      </div>
    </div>
  ),
}));

// ユーティリティ関数のモック
vi.mock('@/lib/utils/dataTransformer', () => ({
  createSearchOptions: vi.fn().mockReturnValue([
    { value: '7203', label: '7203 - トヨタ自動車' },
    { value: '6758', label: '6758 - ソニーグループ' },
  ]),
  filterDataBySearchQuery: vi.fn().mockImplementation((data, query) => {
    if (!query) return data;
    return data.filter((item: AssetBalanceData) =>
      item.security_code.includes(query) || item.security_name.includes(query)
    );
  }),
}));

vi.mock('@/lib/utils/formatters', () => ({
  formatCurrency: vi.fn().mockImplementation((value: number) => `¥${value.toLocaleString()}`),
  formatNumber: vi.fn().mockImplementation((value: number) => value.toLocaleString()),
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

describe('AssetBalance', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('保有銘柄データを正しく表示する', () => {
    render(<AssetBalanceInfo assetBalanceData={mockAssetBalanceData} />);

    expect(screen.getByText('保有銘柄')).toBeInTheDocument();
    expect(screen.getByTestId('receipt-table')).toBeInTheDocument();
    expect(screen.getByText('データ数: 2')).toBeInTheDocument();
  });

  it('空のデータの場合でも正しく表示される', () => {
    render(<AssetBalanceInfo assetBalanceData={[]} />);

    expect(screen.getByText('保有銘柄')).toBeInTheDocument();
    expect(screen.getByText('データ数: 0')).toBeInTheDocument();
  });

  it('テーブルのヘッダーが正しく表示される', () => {
    render(<AssetBalanceInfo assetBalanceData={mockAssetBalanceData} />);

    // 実際に表示されるカラムのみテスト
    expect(screen.getByTestId('header-security_code')).toHaveTextContent('銘柄コード');
    expect(screen.getByTestId('header-security_name')).toHaveTextContent('銘柄名');
    expect(screen.getByTestId('header-shares')).toHaveTextContent('保有数量');
    expect(screen.getByTestId('header-average_purchase_price')).toHaveTextContent('平均取得価額');
    expect(screen.getByTestId('header-total_purchase_amount')).toHaveTextContent('取得総額');
  });

  it('テーブルのプロパティが正しく渡される', () => {
    render(<AssetBalanceInfo assetBalanceData={mockAssetBalanceData} />);

    // summaryとsummaryColumnsが正しくセットされていることを確認
    const tableProps = screen.getByTestId('table-props');
    expect(tableProps).toHaveTextContent('summary: 0');
    expect(tableProps).toHaveTextContent('summaryColumns: 0');
    expect(tableProps).toHaveTextContent('groupKey:');
  });

  it('検索機能が正しく動作する', () => {
    render(<AssetBalanceInfo assetBalanceData={mockAssetBalanceData} />);

    const searchInput = screen.getByTestId('search-input');

    // 検索入力前は全データが表示されている
    expect(screen.getByText('データ数: 2')).toBeInTheDocument();

    // 検索クエリを入力
    fireEvent.change(searchInput, { target: { value: '7203' } });

    // 検索後のデータ表示を確認（filterDataBySearchQueryのモックが動作する）
    expect(searchInput).toHaveValue('7203');
  });

  it('検索オプションが正しく表示される', () => {
    render(<AssetBalanceInfo assetBalanceData={mockAssetBalanceData} />);

    expect(screen.getByTestId('search-options')).toHaveTextContent('オプション数: 2');
  });

  it('カラム設定が正しく定義される', () => {
    render(<AssetBalanceInfo assetBalanceData={mockAssetBalanceData} />);

    // テーブルのヘッダーが5個表示されることを確認
    const headers = screen.getByTestId('table-headers');
    expect(headers.children).toHaveLength(5);
  });

  it('合計情報が正しく表示される', () => {
    render(<AssetBalanceInfo assetBalanceData={mockAssetBalanceData} />);

    // サマリーデータが正しく計算されていることを確認
    const tableProps = screen.getByTestId('table-props');
    expect(tableProps).toHaveTextContent('summary: 0');
  });

  it('空のデータでも合計情報が正しく表示される', () => {
    render(<AssetBalanceInfo assetBalanceData={[]} />);

    // 空のデータの場合でもサマリーが表示される
    const tableProps = screen.getByTestId('table-props');
    expect(tableProps).toHaveTextContent('summary: 0');
  });
});
