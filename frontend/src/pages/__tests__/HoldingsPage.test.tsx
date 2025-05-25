import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { HoldingsPage } from '../HoldingsPage';

// 依存関係のモック
vi.mock('@/components/templates/Layout', () => ({
  Layout: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="layout">{children}</div>
  ),
}));

vi.mock('@/components/molecules/CSVFileInput', () => ({
  CSVFileInput: ({
    onFileSelect,
    selectedFileName
  }: {
    onFileSelect: (file: File) => void;
    selectedFileName: string;
  }) => (
    <div data-testid="csv-file-input">
      <button onClick={() => {
        const file = new File(['test'], 'test.csv', { type: 'text/csv' });
        onFileSelect(file);
      }}>
        保有株CSVファイルを選択
      </button>
      {selectedFileName && <span>{selectedFileName}</span>}
    </div>
  ),
}));

const mockParseCSV = vi.fn();
const mockReset = vi.fn();
const mockResetError = vi.fn();

vi.mock('@/hooks/useCSVReader', () => ({
  useCSVReader: () => ({
    parseCSV: mockParseCSV,
    isLoading: false,
    error: null,
    fileName: 'test.csv',
    reset: mockReset,
    resetError: mockResetError,
  }),
}));

vi.mock('./Receipt/Holdings', () => ({
  Holdings: ({ holdingsData }: { holdingsData: Array<unknown> }) => (
    <div data-testid="holdings-component">
      Holdings with {holdingsData.length} items
    </div>
  ),
}));

const mockSaveHoldings = vi.fn();
const mockClearHoldings = vi.fn();

vi.mock('@/hooks/common/useHoldingsStorage', () => ({
  useHoldingsStorage: () => ({
    holdingsStorageData: [],
    saveHoldings: mockSaveHoldings,
    clearHoldings: mockClearHoldings,
    getHoldingByCode: vi.fn(),
    lastUpdated: '2024-01-01T00:00:00.000Z',
  }),
}));

vi.mock('@/hooks/receipt/useReceiptData', () => ({
  useReceiptData: (data: unknown) => {
    return data.map((item: unknown) => ({
      security_code: item['銘柄コード'] || '',
      security_name: item['銘柄名'] || '',
      shares: 100,
      executing_shares: 0,
      average_purchase_price: 2500,
      total_purchase_amount: 250000,
      current_price: 2600,
      daily_change: 50,
      market_value: 260000,
      profit_loss_rate: 4.0,
    }));
  },
}));

describe('HoldingsPage', () => {

  it('保有株ページが正しく表示される', () => {
    render(<HoldingsPage />);

    expect(screen.getByTestId('layout')).toBeInTheDocument();
    expect(screen.getByTestId('csv-file-input')).toBeInTheDocument();
    expect(screen.getByText('保有株CSVファイルを選択')).toBeInTheDocument();
  });

  beforeEach(() => {
    vi.clearAllMocks();
    window.confirm = vi.fn();
  });

  it('CSVファイルを選択して読み込める', async () => {
    mockParseCSV.mockResolvedValue([
      {
        '銘柄コード': '7203',
        '銘柄名': 'トヨタ自動車',
        '保有数量［株］': '100',
      },
    ]);

    render(<HoldingsPage />);

    const fileButton = screen.getByText('保有株CSVファイルを選択');
    fireEvent.click(fileButton);

    await waitFor(() => {
      expect(mockParseCSV).toHaveBeenCalled();
    });

  });

  it('保有株データを保存できる', async () => {
    mockParseCSV.mockResolvedValue([
      {
        '銘柄コード': '7203',
        '銘柄名': 'トヨタ自動車',
      },
    ]);

    render(<HoldingsPage />);

    const fileButton = screen.getByText('保有株CSVファイルを選択');
    fireEvent.click(fileButton);

    setTimeout(() => {
      const saveButton = screen.getByText('保存');
      expect(saveButton).not.toBeDisabled();
      fireEvent.click(saveButton);
    }, 100);

    setTimeout(() => {
      expect(mockSaveHoldings).toHaveBeenCalledWith([
        expect.objectContaining({
          security_code: '7203',
          security_name: 'トヨタ自動車',
        }),
      ]);
    }, 100);

  });

  it('保存データを削除できる', async () => {
    vi.mocked(window.confirm).mockReturnValue(true);

    // 保存データがある状態でモック
    vi.mock('@/hooks/common/useHoldingsStorage', () => ({
      useHoldingsStorage: () => ({
        holdingsStorageData: [{ security_code: '7203' }],
        saveHoldings: mockSaveHoldings,
        clearHoldings: mockClearHoldings,
        getHoldingByCode: vi.fn(),
        lastUpdated: null,
      }),
    }));

    render(<HoldingsPage />);

    const deleteButton = screen.getByText('保存データを削除');
    fireEvent.click(deleteButton);

    expect(window.confirm).toHaveBeenCalledWith('保存された保有株データを削除しますか？');
    expect(mockClearHoldings).toHaveBeenCalled();
    expect(mockReset).toHaveBeenCalled();
  });

  it('削除確認でキャンセルした場合は削除されない', async () => {
    vi.mocked(window.confirm).mockReturnValue(false);

    render(<HoldingsPage />);

    const deleteButton = screen.getByText('保存データを削除');
    fireEvent.click(deleteButton);

    expect(window.confirm).toHaveBeenCalledWith('保存された保有株データを削除しますか？');
    expect(mockClearHoldings).not.toHaveBeenCalled();
  });
});
