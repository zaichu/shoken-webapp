import React from 'react';
import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { HoldingsPage } from '../HoldingsPage';

// 依存関係のモック
vi.mock('@/components/templates/Layout', () => ({
  Layout: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="layout">{children}</div>
  ),
}));

vi.mock('@/components/molecules/CSVFileInput', () => ({
  CSVFileInput: () => (
    <div data-testid="csv-file-input">保有株CSVファイルを選択</div>
  ),
}));

vi.mock('@/hooks/useCSVReader', () => ({
  useCSVReader: () => ({
    parseCSV: vi.fn(),
    isLoading: false,
    error: null,
    fileName: null,
    resetError: vi.fn(),
  }),
}));

vi.mock('./Receipt/Holdings', () => ({
  Holdings: ({ csvData }: { csvData: Array<unknown> }) => (
    <div data-testid="holdings-component">
      Holdings with {csvData.length} items
    </div>
  ),
}));

describe('HoldingsPage', () => {
  it('保有株ページが正しく表示される', () => {
    render(<HoldingsPage />);

    expect(screen.getByTestId('layout')).toBeInTheDocument();
    expect(screen.getByTestId('csv-file-input')).toBeInTheDocument();
    expect(screen.getByText('保有株CSVファイルを選択')).toBeInTheDocument();
  });
});
