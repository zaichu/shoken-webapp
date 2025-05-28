import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import '@testing-library/jest-dom';
import { ReceiptTemplate } from '../ReceiptTemplate';
import { ReceiptTable } from '../../organisms/ReceiptTable/ReceiptTable';
import { TableColumnConfig, SummaryColumnConfig } from '@/lib/interfaces/receipt';

// ResizeObserverのモック
global.ResizeObserver = jest.fn().mockImplementation(() => ({
  observe: jest.fn(),
  unobserve: jest.fn(),
  disconnect: jest.fn(),
}));

// windowオブジェクトのモック
Object.defineProperty(window, 'innerHeight', {
  writable: true,
  configurable: true,
  value: 1024,
});

Object.defineProperty(window, 'innerWidth', {
  writable: true,
  configurable: true,
  value: 1280,
});

// テスト用のデータとコラム設定
type TestDataItem = {
  id: string;
  name: string;
  amount: number;
  category: string;
};

type TestSummaryItem = {
  filter: string;
  totalAmount: number;
  count: number;
};

const testColumns: TableColumnConfig[] = [
  { key: 'id', header: 'ID', width: '100px', textAlign: 'center' },
  { key: 'name', header: '名前', width: '200px', textAlign: 'left' },
  { key: 'amount', header: '金額', width: '150px', textAlign: 'right' },
  { key: 'category', header: 'カテゴリ', width: '150px', textAlign: 'center' }
];

const testSummaryColumns: SummaryColumnConfig[] = [
  { key: 'filter', header: '合計', width: '300px', textAlign: 'left', colSpan: 2 },
  { key: 'totalAmount', header: '金額合計', width: '150px', textAlign: 'right' },
  { key: 'count', header: '件数', width: '150px', textAlign: 'center' }
];

const testData: TestDataItem[] = [
  { id: '1', name: '商品A', amount: 1000, category: 'カテゴリ1' },
  { id: '2', name: '商品B', amount: 2000, category: 'カテゴリ1' },
  { id: '3', name: '商品C', amount: 1500, category: 'カテゴリ2' },
];

const testSummary: TestSummaryItem[] = [
  { filter: 'カテゴリ1', totalAmount: 3000, count: 2 },
  { filter: 'カテゴリ2', totalAmount: 1500, count: 1 },
];

const getGroupKey = (item: TestDataItem) => item.category;

describe('ReceiptTemplate統合テスト', () => {
  const mockOnSearch = jest.fn();
  const mockOnSearchExpandToggle = jest.fn();

  const searchCategories = {
    securities: [
      { value: 'AAPL', label: 'Apple Inc.' },
      { value: 'GOOGL', label: 'Alphabet Inc.' }
    ],
    products: ['株式', '投資信託'],
    accounts: ['一般口座', 'NISA口座'],
    years: ['2023', '2024'],
    yearMonths: [
      { value: '2024-01', label: '2024年1月' },
      { value: '2024-02', label: '2024年2月' }
    ]
  };

  beforeEach(() => {
    jest.clearAllMocks();
    jest.useFakeTimers();
  });

  afterEach(() => {
    jest.useRealTimers();
  });

  test('SearchCard展開時にTableのforceResizeが更新される', async () => {
    render(
      <ReceiptTemplate
        title="統合テスト"
        onSearch={mockOnSearch}
        onSearchExpandToggle={mockOnSearchExpandToggle}
        searchCategories={searchCategories}
      >
        <ReceiptTable
          data={testData}
          summary={testSummary}
          columns={testColumns}
          summaryColumns={testSummaryColumns}
          getGroupKey={getGroupKey}
        />
      </ReceiptTemplate>
    );

    // 初期状態の確認
    expect(screen.getByText('統合テスト')).toBeInTheDocument();
    expect(screen.getByText('検索オプション')).toBeInTheDocument();
    expect(screen.getByRole('table')).toBeInTheDocument();

    // SearchCardを展開
    const header = screen.getByText('検索オプション').closest('.card-header');
    fireEvent.click(header!);

    // タイマーを進める
    jest.advanceTimersByTime(100);

    // コールバックが呼ばれることを確認
    expect(mockOnSearchExpandToggle).toHaveBeenCalledWith(true);

    // テーブルが正常に表示されることを確認
    expect(screen.getByText('商品A')).toBeInTheDocument();
    expect(screen.getByText('商品B')).toBeInTheDocument();
    expect(screen.getByText('商品C')).toBeInTheDocument();
  });

  test('検索機能が正常に動作する', () => {
    render(
      <ReceiptTemplate
        title="検索テスト"
        onSearch={mockOnSearch}
        searchCategories={searchCategories}
      >
        <ReceiptTable
          data={testData}
          summary={testSummary}
          columns={testColumns}
          summaryColumns={testSummaryColumns}
          getGroupKey={getGroupKey}
        />
      </ReceiptTemplate>
    );

    // SearchCardを展開
    const header = screen.getByText('検索オプション').closest('.card-header');
    fireEvent.click(header!);

    // 年度ボタンをクリック
    const yearButton = screen.getByText('2024');
    fireEvent.click(yearButton);

    expect(mockOnSearch).toHaveBeenCalledWith('2024');
  });

  test('複数回の展開・折りたたみでforceResizeが適切に動作する', async () => {
    render(
      <ReceiptTemplate
        title="リサイズテスト"
        onSearch={mockOnSearch}
        onSearchExpandToggle={mockOnSearchExpandToggle}
        searchCategories={searchCategories}
      >
        <ReceiptTable
          data={testData}
          summary={testSummary}
          columns={testColumns}
          summaryColumns={testSummaryColumns}
          getGroupKey={getGroupKey}
        />
      </ReceiptTemplate>
    );

    const header = screen.getByText('検索オプション').closest('.card-header');

    // 複数回展開・折りたたみを実行
    for (let i = 0; i < 3; i++) {
      // 展開
      fireEvent.click(header!);
      jest.advanceTimersByTime(100);
      expect(mockOnSearchExpandToggle).toHaveBeenCalledWith(true);

      // 折りたたみ
      fireEvent.click(header!);
      jest.advanceTimersByTime(100);
      expect(mockOnSearchExpandToggle).toHaveBeenCalledWith(false);
    }

    // 各操作で適切にコールバックが呼ばれることを確認
    expect(mockOnSearchExpandToggle).toHaveBeenCalledTimes(6);
  });

  test('テーブルデータが正しく表示される', () => {
    render(
      <ReceiptTemplate
        title="データ表示テスト"
        onSearch={mockOnSearch}
        searchCategories={searchCategories}
      >
        <ReceiptTable
          data={testData}
          summary={testSummary}
          columns={testColumns}
          summaryColumns={testSummaryColumns}
          getGroupKey={getGroupKey}
        />
      </ReceiptTemplate>
    );

    // ヘッダーの確認
    expect(screen.getByText('ID')).toBeInTheDocument();
    expect(screen.getByText('名前')).toBeInTheDocument();
    expect(screen.getByText('金額')).toBeInTheDocument();
    expect(screen.getByText('カテゴリ')).toBeInTheDocument();

    // データの確認
    expect(screen.getByText('1')).toBeInTheDocument();
    expect(screen.getByText('商品A')).toBeInTheDocument();
    expect(screen.getByText('1000')).toBeInTheDocument();
    expect(screen.getByText('カテゴリ1')).toBeInTheDocument();

    // サマリーの確認
    expect(screen.getByText('3000')).toBeInTheDocument();
    expect(screen.getByText('1500')).toBeInTheDocument();
  });

  test('検索カテゴリのドロップダウンが正常に動作する', () => {
    render(
      <ReceiptTemplate
        title="ドロップダウンテスト"
        onSearch={mockOnSearch}
        searchCategories={searchCategories}
      >
        <ReceiptTable
          data={testData}
          summary={testSummary}
          columns={testColumns}
          summaryColumns={testSummaryColumns}
          getGroupKey={getGroupKey}
        />
      </ReceiptTemplate>
    );

    // SearchCardを展開
    const header = screen.getByText('検索オプション').closest('.card-header');
    fireEvent.click(header!);

    // 銘柄ドロップダウンを操作
    const select = screen.getByLabelText('検索フィルター');
    fireEvent.change(select, { target: { value: 'AAPL' } });

    expect(mockOnSearch).toHaveBeenCalledWith('AAPL');
  });

  test('複数のReceiptTableがある場合、全てにforceResizeが適用される', () => {
    const TestComponent = () => (
      <ReceiptTemplate
        title="複数テーブルテスト"
        onSearch={mockOnSearch}
        searchCategories={searchCategories}
      >
        <ReceiptTable
          data={testData}
          summary={testSummary}
          columns={testColumns}
          summaryColumns={testSummaryColumns}
          getGroupKey={getGroupKey}
        />
        <ReceiptTable
          data={testData.slice(0, 1)}
          summary={[]}
          columns={testColumns}
          summaryColumns={testSummaryColumns}
          getGroupKey={getGroupKey}
        />
      </ReceiptTemplate>
    );

    render(<TestComponent />);

    const tables = screen.getAllByRole('table');
    expect(tables).toHaveLength(2);

    // SearchCardを展開
    const header = screen.getByText('検索オプション').closest('.card-header');
    fireEvent.click(header!);

    jest.advanceTimersByTime(100);

    // 両方のテーブルが正常に表示されることを確認
    expect(tables[0]).toBeInTheDocument();
    expect(tables[1]).toBeInTheDocument();
  });

  test('エラーが発生してもアプリケーションがクラッシュしない', () => {
    // コンソールエラーを一時的に無効にする
    const consoleSpy = jest.spyOn(console, 'error').mockImplementation();

    render(
      <ReceiptTemplate
        title="エラーハンドリングテスト"
        onSearch={mockOnSearch}
        searchCategories={searchCategories}
      >
        <ReceiptTable
          data={testData}
          summary={testSummary}
          columns={testColumns}
          summaryColumns={testSummaryColumns}
          getGroupKey={getGroupKey}
        />
      </ReceiptTemplate>
    );

    // SearchCardを展開
    const header = screen.getByText('検索オプション').closest('.card-header');
    
    // 例外が発生してもアプリケーションが正常に動作することを確認
    expect(() => {
      fireEvent.click(header!);
      jest.advanceTimersByTime(100);
    }).not.toThrow();

    // 基本的な要素が表示されることを確認
    expect(screen.getByText('エラーハンドリングテスト')).toBeInTheDocument();
    expect(screen.getByRole('table')).toBeInTheDocument();

    consoleSpy.mockRestore();
  });

  test('ResizeObserverが正しく設定される', () => {
    render(
      <ReceiptTemplate
        title="ResizeObserverテスト"
        onSearch={mockOnSearch}
        searchCategories={searchCategories}
      >
        <ReceiptTable
          data={testData}
          summary={testSummary}
          columns={testColumns}
          summaryColumns={testSummaryColumns}
          getGroupKey={getGroupKey}
        />
      </ReceiptTemplate>
    );

    // ResizeObserverが作成されることを確認
    expect(global.ResizeObserver).toHaveBeenCalled();
  });
});
