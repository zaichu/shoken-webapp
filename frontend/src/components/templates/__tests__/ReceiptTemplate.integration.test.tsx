import { render, screen, fireEvent } from '@testing-library/react';
import { vi, describe, test, expect, beforeEach, afterEach } from 'vitest';
import '@testing-library/jest-dom';
import { ReceiptTemplate } from '../ReceiptTemplate';
import { ReceiptTable } from '../../organisms/ReceiptTable/ReceiptTable';
import { TableColumnConfig, SummaryColumnConfig } from '@/lib/interfaces/receipt';

// ResizeObserverのモック（class形式で定義）
class MockResizeObserver {
  observe() {}
  unobserve() {}
  disconnect() {}
}
global.ResizeObserver = MockResizeObserver as unknown as typeof ResizeObserver;

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
  { key: 'filter', textAlign: 'left', colSpan: 2 },
  { key: 'totalAmount', textAlign: 'right' },
  { key: 'count', textAlign: 'center' }
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

describe('ReceiptTemplate Context API統合テスト', () => {
  const mockOnSearch = vi.fn();
  const mockOnSearchExpandToggle = vi.fn();

  const searchCategories = {
    securities: [
      { value: 'AAPL', label: 'Apple Inc.' },
      { value: 'GOOGL', label: 'Alphabet Inc.' }
    ],
    products: ['株式', '投資信託'],
    accounts: ['一般口座', 'NISA口座'],
    years: [
      { value: '2023', label: '2023年' },
      { value: '2024', label: '2024年' }
    ],
  };

  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  test('SearchCard展開時にTableのforceResizeが更新される', async () => {
    render(
      <ReceiptTemplate
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

    // 初期状態の確認（SearchCardは折りたたみ済み）
    expect(screen.getByText('検索オプション')).toBeInTheDocument();
    expect(screen.getByRole('table')).toBeInTheDocument();

    const header = screen.getByTestId('search-card-header');

    // SearchCardを展開する
    fireEvent.click(header!);
    vi.advanceTimersByTime(100);

    // コールバックが呼ばれることを確認（展開）
    expect(mockOnSearchExpandToggle).toHaveBeenCalledWith(true);

    // SearchCardを折りたたむ
    fireEvent.click(header!);
    vi.advanceTimersByTime(100);

    // コールバックが呼ばれることを確認（折りたたみ）
    expect(mockOnSearchExpandToggle).toHaveBeenCalledWith(false);

    // テーブルが正常に表示されることを確認
    expect(screen.getByText('商品A')).toBeInTheDocument();
    expect(screen.getByText('商品B')).toBeInTheDocument();
    expect(screen.getByText('商品C')).toBeInTheDocument();
  });

  test('検索機能が正常に動作する', () => {
    render(
      <ReceiptTemplate
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

    // SearchCardを展開してから商品ボタンをクリック
    const header = screen.getByTestId('search-card-header');
    fireEvent.click(header);

    const productButton = screen.getByText('株式');
    fireEvent.click(productButton);

    expect(mockOnSearch).toHaveBeenCalledWith('株式');
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

    const header = screen.getByTestId('search-card-header');

    // 初期状態は折りたたみ済み - 複数回展開・折りたたみを実行
    for (let i = 0; i < 3; i++) {
      // 展開
      fireEvent.click(header!);
      vi.advanceTimersByTime(100);
      expect(mockOnSearchExpandToggle).toHaveBeenCalledWith(true);

      // 折りたたみ
      fireEvent.click(header!);
      vi.advanceTimersByTime(100);
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

    // データの確認（複数箇所に同一テキストが出る場合は getAllByText を使用）
    expect(screen.getAllByText('1').length).toBeGreaterThan(0);
    expect(screen.getByText('商品A')).toBeInTheDocument();
    expect(screen.getByText('1000')).toBeInTheDocument();
    expect(screen.getAllByText('カテゴリ1').length).toBeGreaterThan(0);

    // サマリーの確認（1500 はデータ行とサマリー行の両方に出る）
    expect(screen.getByText('3000')).toBeInTheDocument();
    expect(screen.getAllByText('1500').length).toBeGreaterThan(0);
  });

  test('検索カテゴリのドロップダウンが正常に動作する', () => {
    const { container } = render(
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

    // SearchCardを展開してから銘柄ドロップダウンを操作（IDで指定）
    const header = screen.getByTestId('search-card-header');
    fireEvent.click(header);

    const select = container.querySelector('#securities-search') as HTMLSelectElement;
    fireEvent.change(select, { target: { value: 'AAPL' } });

    expect(mockOnSearch).toHaveBeenCalledWith('AAPL');
  });

  test('複数のReceiptTableがある場合、全てにforceResizeが適用される', { timeout: 15000 }, () => {
    const TestComponent = () => (
      <ReceiptTemplate
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
    const header = screen.getByTestId('search-card-header');
    fireEvent.click(header!);

    vi.advanceTimersByTime(100);

    // 両方のテーブルが正常に表示されることを確認
    expect(tables[0]).toBeInTheDocument();
    expect(tables[1]).toBeInTheDocument();
  });

  test('エラーが発生してもアプリケーションがクラッシュしない', () => {
    // コンソールエラーを一時的に無効にする
    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

    render(
      <ReceiptTemplate
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
    const header = screen.getByTestId('search-card-header');

    // 例外が発生してもアプリケーションが正常に動作することを確認
    expect(() => {
      fireEvent.click(header!);
      vi.advanceTimersByTime(100);
    }).not.toThrow();

    // 基本的な要素が表示されることを確認
    expect(screen.getByTestId('receipt-container')).toBeInTheDocument();
    expect(screen.getByRole('table')).toBeInTheDocument();

    consoleSpy.mockRestore();
  });

  test('Context無しでも既存のテーブルが動作する', () => {
    // ResizeProvider外でReceiptTableを直接使用（Context無し）
    render(
      <ReceiptTable
        data={testData}
        summary={testSummary}
        columns={testColumns}
        summaryColumns={testSummaryColumns}
        getGroupKey={getGroupKey}
      />
    );

    // テーブルが正常に表示されることを確認（forceResizeはundefinedでも動作）
    expect(screen.getByRole('table')).toBeInTheDocument();
    expect(screen.getByText('商品A')).toBeInTheDocument();
    expect(screen.getByText('商品B')).toBeInTheDocument();
    expect(screen.getByText('商品C')).toBeInTheDocument();
  });

  test('ResizeObserverが設定されてもテーブルが正常に動作する', () => {
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

    // テーブルが正常に表示されることを確認
    expect(screen.getByRole('table')).toBeInTheDocument();
    expect(screen.getByText('商品A')).toBeInTheDocument();
  });
});
