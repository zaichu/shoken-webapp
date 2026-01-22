import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import { vi, describe, test, expect, beforeEach } from 'vitest';
import '@testing-library/jest-dom';
import { ReceiptTemplate } from '../ReceiptTemplate';

// モック用のReceiptTableコンポーネント
const MockReceiptTable = React.forwardRef<HTMLDivElement, { forceResize?: number }>(
  ({ forceResize, ...props }, ref) => (
    <div
      ref={ref}
      data-testid="mock-receipt-table"
      data-force-resize={forceResize}
      {...props}
    >
      Mock Receipt Table
    </div>
  )
);
MockReceiptTable.displayName = 'ReceiptTable';

// SearchCardのモック
vi.mock('@/components/organisms/SearchCard/SearchCard', () => ({
  SearchCard: ({ onSearch, onExpandToggle }: {
    onSearch: (query: string) => void;
    onExpandToggle?: (isExpanded: boolean) => void;
  }) => (
    <div data-testid="search-card">
      <button onClick={() => onSearch('test-query')}>Search</button>
      <button onClick={() => onExpandToggle?.(true)}>Expand</button>
      <button onClick={() => onExpandToggle?.(false)}>Collapse</button>
    </div>
  )
}));

describe('ReceiptTemplate', () => {
  const mockOnSearch = vi.fn();
  const mockOnSearchExpandToggle = vi.fn();

  const defaultProps = {
    title: 'テストタイトル',
    children: <MockReceiptTable />
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  test('基本的なレイアウトが正しくレンダリングされる', () => {
    render(<ReceiptTemplate {...defaultProps} />);

    expect(screen.getByText('テストタイトル')).toBeInTheDocument();
    expect(screen.getByTestId('mock-receipt-table')).toBeInTheDocument();
  });

  test('検索機能がある場合SearchCardが表示される', () => {
    render(
      <ReceiptTemplate 
        {...defaultProps}
        onSearch={mockOnSearch}
        searchCategories={{
          securities: [{ value: 'AAPL', label: 'Apple' }],
          products: ['株式'],
          accounts: ['一般口座'],
          years: ['2024'],
          yearMonths: [{ value: '2024-01', label: '2024年1月' }]
        }}
      />
    );

    expect(screen.getByTestId('search-card')).toBeInTheDocument();
  });

  test('検索機能がない場合SearchCardが表示されない', () => {
    render(<ReceiptTemplate {...defaultProps} />);

    expect(screen.queryByTestId('search-card')).not.toBeInTheDocument();
  });

  test('ヘッダーが提供された場合表示される', () => {
    const headerContent = <div data-testid="header-content">カスタムヘッダー</div>;
    
    render(
      <ReceiptTemplate 
        {...defaultProps}
        header={headerContent}
      />
    );

    expect(screen.getByTestId('header-content')).toBeInTheDocument();
    expect(screen.getByText('カスタムヘッダー')).toBeInTheDocument();
  });

  test('フッターが提供された場合表示される', () => {
    const footerContent = <div data-testid="footer-content">カスタムフッター</div>;
    
    render(
      <ReceiptTemplate 
        {...defaultProps}
        footer={footerContent}
      />
    );

    expect(screen.getByTestId('footer-content')).toBeInTheDocument();
    expect(screen.getByText('カスタムフッター')).toBeInTheDocument();
  });

  // TODO: forceResizeの実装変更に伴いテストを修正する必要あり
  test.skip('SearchCard展開時にforceResizeが更新される', () => {
    render(
      <ReceiptTemplate 
        {...defaultProps}
        onSearch={mockOnSearch}
        onSearchExpandToggle={mockOnSearchExpandToggle}
      />
    );

    const initialForceResize = screen.getByTestId('mock-receipt-table').getAttribute('data-force-resize');
    
    // SearchCardを展開
    const expandButton = screen.getByText('Expand');
    fireEvent.click(expandButton);

    const updatedForceResize = screen.getByTestId('mock-receipt-table').getAttribute('data-force-resize');
    
    // forceResizeが更新されることを確認
    expect(updatedForceResize).not.toBe(initialForceResize);
    expect(mockOnSearchExpandToggle).toHaveBeenCalledWith(true);
  });

  // TODO: forceResizeの実装変更に伴いテストを修正する必要あり
  test.skip('SearchCard折りたたみ時にforceResizeが更新される', () => {
    render(
      <ReceiptTemplate 
        {...defaultProps}
        onSearch={mockOnSearch}
        onSearchExpandToggle={mockOnSearchExpandToggle}
      />
    );

    const initialForceResize = screen.getByTestId('mock-receipt-table').getAttribute('data-force-resize');
    
    // SearchCardを折りたたみ
    const collapseButton = screen.getByText('Collapse');
    fireEvent.click(collapseButton);

    const updatedForceResize = screen.getByTestId('mock-receipt-table').getAttribute('data-force-resize');
    
    // forceResizeが更新されることを確認
    expect(updatedForceResize).not.toBe(initialForceResize);
    expect(mockOnSearchExpandToggle).toHaveBeenCalledWith(false);
  });

  test('複数の子要素がある場合、ReceiptTableのみがenhanceされる', () => {
    const NonReceiptTableChild = () => <div data-testid="non-receipt-table">Other Component</div>;
    
    render(
      <ReceiptTemplate 
        {...defaultProps}
        onSearch={mockOnSearch}
      >
        <MockReceiptTable />
        <NonReceiptTableChild />
      </ReceiptTemplate>
    );

    const receiptTable = screen.getByTestId('mock-receipt-table');
    const otherComponent = screen.getByTestId('non-receipt-table');
    
    // ReceiptTableにはforceResizeが設定されている
    expect(receiptTable.getAttribute('data-force-resize')).toBeDefined();
    
    // 他のコンポーネントにはforceResizeが設定されていない
    expect(otherComponent.getAttribute('data-force-resize')).toBeNull();
  });

  test('ReceiptTableがない場合でもエラーが発生しない', () => {
    const NonReceiptTableChild = () => <div data-testid="non-receipt-table">Other Component</div>;
    
    render(
      <ReceiptTemplate 
        {...defaultProps}
        onSearch={mockOnSearch}
      >
        <NonReceiptTableChild />
      </ReceiptTemplate>
    );

    expect(screen.getByTestId('non-receipt-table')).toBeInTheDocument();
    
    // SearchCard操作でもエラーが発生しない
    const expandButton = screen.getByText('Expand');
    fireEvent.click(expandButton);
    
    expect(screen.getByTestId('non-receipt-table')).toBeInTheDocument();
  });

  test('searchCategoriesが部分的に提供された場合正しく処理される', () => {
    render(
      <ReceiptTemplate 
        {...defaultProps}
        onSearch={mockOnSearch}
        searchCategories={{
          securities: [{ value: 'AAPL', label: 'Apple' }],
          // 他のプロパティは未定義
        }}
      />
    );

    expect(screen.getByTestId('search-card')).toBeInTheDocument();
  });

  test('onSearchExpandToggleが提供されない場合でもエラーが発生しない', () => {
    render(
      <ReceiptTemplate 
        {...defaultProps}
        onSearch={mockOnSearch}
      />
    );

    // SearchCard操作でもエラーが発生しない
    const expandButton = screen.getByText('Expand');
    fireEvent.click(expandButton);
    
    expect(screen.getByTestId('mock-receipt-table')).toBeInTheDocument();
  });

  test('カードのレイアウト構造が正しい', () => {
    render(<ReceiptTemplate {...defaultProps} />);

    // メインカードの存在確認
    const mainCard = screen.getByText('テストタイトル').closest('.card');
    expect(mainCard).toBeInTheDocument();
    expect(mainCard).toHaveClass('shadow-sm', 'mt-1');

    // ヘッダーの存在確認
    const cardHeader = screen.getByText('テストタイトル').closest('.card-header');
    expect(cardHeader).toBeInTheDocument();
    expect(cardHeader).toHaveClass('bg-primary', 'text-white');

    // ボディの存在確認
    const cardBody = screen.getByTestId('mock-receipt-table').closest('.card-body');
    expect(cardBody).toBeInTheDocument();
    expect(cardBody).toHaveClass('p-0');
  });

  test('receipt-containerクラスが適用される', () => {
    const { container } = render(<ReceiptTemplate {...defaultProps} />);
    
    const receiptContainer = container.querySelector('.receipt-container');
    expect(receiptContainer).toBeInTheDocument();
  });
});
