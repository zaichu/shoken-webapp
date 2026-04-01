import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { ApiError, ApiErrorType } from '@/lib/types/api';
import { SearchPage } from '../Search';

const { mockUseStockSearch } = vi.hoisted(() => ({
  mockUseStockSearch: vi.fn(),
}));

vi.mock('@/components/templates/Layout', () => ({
  Layout: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
}));

vi.mock('@/components/organisms/SearchForm', () => ({
  SearchForm: () => <div>SearchForm</div>,
}));

vi.mock('@/components/organisms/StockInfo', () => ({
  StockInfo: () => <div>StockInfo</div>,
}));

vi.mock('@/components/atoms/EmptyState', () => ({
  EmptyState: ({ title, description }: { title: string; description?: string }) => (
    <div>
      <div>{title}</div>
      {description && <div>{description}</div>}
    </div>
  ),
}));

vi.mock('@/components/atoms/PageHeader', () => ({
  PageHeader: ({ title, description }: { title: string; description?: string }) => (
    <div>
      <h1>{title}</h1>
      {description && <p>{description}</p>}
    </div>
  ),
}));

vi.mock('@/components/atoms/Alert', () => ({
  Alert: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
}));

vi.mock('@/features/stock/hooks/useStockSearch', () => ({
  useStockSearch: (...args: unknown[]) => mockUseStockSearch(...args),
}));

vi.mock('@/hooks/usePageTitle', () => ({
  usePageTitle: vi.fn(),
}));

describe('SearchPage', () => {
  beforeEach(() => {
    vi.clearAllMocks();

    mockUseStockSearch.mockReturnValue({
      stockCode: '',
      setStockCode: vi.fn(),
      stockData: undefined,
      error: null,
      isLoading: false,
      isError: false,
      handleSearch: vi.fn(),
      searchByCode: vi.fn(),
    });
  });

  it('有効な ?code= パラメータがある場合に searchByCode を呼ぶ', () => {
    const searchByCode = vi.fn();

    mockUseStockSearch.mockReturnValue({
      stockCode: '',
      setStockCode: vi.fn(),
      stockData: undefined,
      error: null,
      isLoading: false,
      isError: false,
      handleSearch: vi.fn(),
      searchByCode,
    });

    render(
      <MemoryRouter initialEntries={['/search?code=7203']}>
        <SearchPage />
      </MemoryRouter>
    );

    expect(searchByCode).toHaveBeenCalledWith('7203');
  });

  it('無効な ?code= パラメータの場合は警告を表示し searchByCode を呼ばない', () => {
    const searchByCode = vi.fn();

    mockUseStockSearch.mockReturnValue({
      stockCode: '',
      setStockCode: vi.fn(),
      stockData: undefined,
      error: null,
      isLoading: false,
      isError: false,
      handleSearch: vi.fn(),
      searchByCode,
    });

    render(
      <MemoryRouter initialEntries={['/search?code=12*4']}>
        <SearchPage />
      </MemoryRouter>
    );

    expect(screen.getByText('不正な銘柄コードが指定されています。')).toBeInTheDocument();
    expect(searchByCode).not.toHaveBeenCalled();
  });

  it('ApiError のユーザー向けメッセージを表示する', () => {
    mockUseStockSearch.mockReturnValue({
      stockCode: '',
      setStockCode: vi.fn(),
      stockData: undefined,
      error: new ApiError(
        ApiErrorType.DESERIALIZATION_ERROR,
        'J-Quants API の応答形式が変更された可能性があります'
      ),
      isLoading: false,
      isError: true,
      handleSearch: vi.fn(),
      searchByCode: vi.fn(),
    });

    render(
      <MemoryRouter>
        <SearchPage />
      </MemoryRouter>
    );

    expect(
      screen.getByText('銘柄情報の取得に失敗しました。時間をおいて再度お試しください。')
    ).toBeInTheDocument();
    expect(
      screen.queryByText('J-Quants API の応答形式が変更された可能性があります')
    ).not.toBeInTheDocument();
  });
});
