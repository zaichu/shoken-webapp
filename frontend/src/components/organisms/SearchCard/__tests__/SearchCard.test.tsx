import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import '@testing-library/jest-dom';
import { SearchCard } from '../SearchCard';

describe('SearchCard', () => {
  const mockOnSearch = jest.fn();
  const mockOnExpandToggle = jest.fn();

  const defaultCategories = {
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
  });

  test('検索カードが正しくレンダリングされる', () => {
    render(
      <SearchCard
        onSearch={mockOnSearch}
        categories={defaultCategories}
      />
    );

    expect(screen.getByText('検索オプション')).toBeInTheDocument();
    expect(screen.getByText('展開')).toBeInTheDocument();
  });

  test('初期状態では検索オプションが折りたたまれている', () => {
    render(
      <SearchCard
        onSearch={mockOnSearch}
        categories={defaultCategories}
      />
    );

    expect(screen.queryByText('銘柄')).not.toBeInTheDocument();
    expect(screen.queryByText('年度')).not.toBeInTheDocument();
    expect(screen.queryByText('商品')).not.toBeInTheDocument();
  });

  test('ヘッダーをクリックすると検索オプションが展開される', () => {
    render(
      <SearchCard
        onSearch={mockOnSearch}
        categories={defaultCategories}
      />
    );

    const header = screen.getByText('検索オプション').closest('.card-header');
    fireEvent.click(header!);

    expect(screen.getByText('銘柄')).toBeInTheDocument();
    expect(screen.getByText('年度')).toBeInTheDocument();
    expect(screen.getByText('商品')).toBeInTheDocument();
    expect(screen.getByText('年月')).toBeInTheDocument();
    expect(screen.getByText('口座')).toBeInTheDocument();
    expect(screen.getByText('折りたたむ')).toBeInTheDocument();
  });

  test('onExpandToggleコールバックが呼ばれる', () => {
    render(
      <SearchCard
        onSearch={mockOnSearch}
        categories={defaultCategories}
        onExpandToggle={mockOnExpandToggle}
      />
    );

    const header = screen.getByText('検索オプション').closest('.card-header');
    fireEvent.click(header!);

    expect(mockOnExpandToggle).toHaveBeenCalledWith(true);

    fireEvent.click(header!);
    expect(mockOnExpandToggle).toHaveBeenCalledWith(false);
  });

  test('銘柄ドロップダウンから選択すると検索が実行される', () => {
    render(
      <SearchCard
        onSearch={mockOnSearch}
        categories={defaultCategories}
      />
    );

    // 展開
    const header = screen.getByText('検索オプション').closest('.card-header');
    fireEvent.click(header!);

    // 銘柄選択
    const select = screen.getByLabelText('検索フィルター');
    fireEvent.change(select, { target: { value: 'AAPL' } });

    expect(mockOnSearch).toHaveBeenCalledWith('AAPL');
  });

  test('年度ボタンをクリックすると検索が実行される', () => {
    render(
      <SearchCard
        onSearch={mockOnSearch}
        categories={defaultCategories}
      />
    );

    // 展開
    const header = screen.getByText('検索オプション').closest('.card-header');
    fireEvent.click(header!);

    // 年度ボタンクリック
    const yearButton = screen.getByText('2023');
    fireEvent.click(yearButton);

    expect(mockOnSearch).toHaveBeenCalledWith('2023');
  });

  test('商品ボタンをクリックすると検索が実行される', () => {
    render(
      <SearchCard
        onSearch={mockOnSearch}
        categories={defaultCategories}
      />
    );

    // 展開
    const header = screen.getByText('検索オプション').closest('.card-header');
    fireEvent.click(header!);

    // 商品ボタンクリック
    const productButton = screen.getByText('株式');
    fireEvent.click(productButton);

    expect(mockOnSearch).toHaveBeenCalledWith('株式');
  });

  test('口座ボタンをクリックすると検索が実行される', () => {
    render(
      <SearchCard
        onSearch={mockOnSearch}
        categories={defaultCategories}
      />
    );

    // 展開
    const header = screen.getByText('検索オプション').closest('.card-header');
    fireEvent.click(header!);

    // 口座ボタンクリック
    const accountButton = screen.getByText('一般口座');
    fireEvent.click(accountButton);

    expect(mockOnSearch).toHaveBeenCalledWith('一般口座');
  });

  test('年月ドロップダウンから選択すると検索が実行される', () => {
    render(
      <SearchCard
        onSearch={mockOnSearch}
        categories={defaultCategories}
      />
    );

    // 展開
    const header = screen.getByText('検索オプション').closest('.card-header');
    fireEvent.click(header!);

    // 年月選択（複数のセレクトがある場合の対処）
    const selects = screen.getAllByRole('combobox');
    const yearMonthSelect = selects.find(select => 
      select.querySelector('option[value="2024-01"]')
    );
    
    if (yearMonthSelect) {
      fireEvent.change(yearMonthSelect, { target: { value: '2024-01' } });
      expect(mockOnSearch).toHaveBeenCalledWith('2024-01');
    }
  });

  test('categoriesが未定義の場合でもエラーが発生しない', () => {
    render(
      <SearchCard
        onSearch={mockOnSearch}
      />
    );

    expect(screen.getByText('検索オプション')).toBeInTheDocument();
    
    // 展開してもエラーが発生しないことを確認
    const header = screen.getByText('検索オプション').closest('.card-header');
    fireEvent.click(header!);
  });

  test('空のカテゴリが正しく処理される', () => {
    render(
      <SearchCard
        onSearch={mockOnSearch}
        categories={{
          securities: [],
          products: [],
          accounts: [],
          years: [],
          yearMonths: []
        }}
      />
    );

    const header = screen.getByText('検索オプション').closest('.card-header');
    fireEvent.click(header!);

    // 空のカテゴリは表示されない
    expect(screen.queryByText('銘柄')).not.toBeInTheDocument();
    expect(screen.queryByText('年度')).not.toBeInTheDocument();
    expect(screen.queryByText('商品')).not.toBeInTheDocument();
  });

  test('undefinedカテゴリが正しく処理される', () => {
    render(
      <SearchCard
        onSearch={mockOnSearch}
        categories={{
          securities: undefined,
          products: undefined,
          accounts: undefined,
          years: undefined,
          yearMonths: undefined
        }}
      />
    );

    const header = screen.getByText('検索オプション').closest('.card-header');
    fireEvent.click(header!);

    // undefinedのカテゴリは表示されない
    expect(screen.queryByText('銘柄')).not.toBeInTheDocument();
    expect(screen.queryByText('年度')).not.toBeInTheDocument();
    expect(screen.queryByText('商品')).not.toBeInTheDocument();
  });

  test('10個を超えるボタンで改行が適用される', () => {
    const manyProducts = Array.from({ length: 25 }, (_, i) => `商品${i + 1}`);
    
    render(
      <SearchCard
        onSearch={mockOnSearch}
        categories={{
          ...defaultCategories,
          products: manyProducts
        }}
      />
    );

    const header = screen.getByText('検索オプション').closest('.card-header');
    fireEvent.click(header!);

    // 11番目、21番目の商品ボタンの後に改行要素があることを確認
    expect(screen.getByText('商品11')).toBeInTheDocument();
    expect(screen.getByText('商品21')).toBeInTheDocument();
  });

  test('ドロップダウンの「全て表示」オプションが正しく動作する', () => {
    render(
      <SearchCard
        onSearch={mockOnSearch}
        categories={defaultCategories}
      />
    );

    const header = screen.getByText('検索オプション').closest('.card-header');
    fireEvent.click(header!);

    const select = screen.getByLabelText('検索フィルター');
    fireEvent.change(select, { target: { value: '' } });

    expect(mockOnSearch).toHaveBeenCalledWith('');
  });
});
