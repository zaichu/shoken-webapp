/**
 * ReceiptsPage のテスト
 *
 * 対象:
 * - receiptsReducer: 状態遷移の単体テスト（LOGOUT, SET_CSV_DATA 等）
 * - ReceiptsPage: タブ切替・ログアウト時データクリアの統合テスト
 */
import React from 'react';
import { screen, waitFor, act } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import { makeQueryClient, renderWithQuery, waitOpts } from '@/test/utils';

import { receiptsReducer, initialState } from '../receiptsReducer';
import type { ReceiptsState } from '../receiptsReducer';
import { ReceiptsPage } from '../Receipts';
import { receiptQueryKeys } from '@/features/receipt/queryKeys';

// ────────────────────────────────────────────────────────
// モック定義
// ────────────────────────────────────────────────────────

// Layout / PageHeader は描画を単純化
vi.mock('@/components/templates/Layout', () => ({
  Layout: ({ children }: { children: React.ReactNode }) => <div data-testid="layout">{children}</div>,
}));
vi.mock('@/components/atoms/PageHeader', () => ({
  PageHeader: ({ title }: { title: string }) => <h1>{title}</h1>,
}));

// Alert / Spinner は実物でも問題ないが念のため簡略化
vi.mock('@/components/atoms/Alert', () => ({
  Alert: ({ children }: { children: React.ReactNode }) => <div role="alert">{children}</div>,
}));
vi.mock('@/components/atoms/Spinner', () => ({
  Spinner: () => <span role="status">loading</span>,
}));

// CSVFileInput: ボタンでファイル選択を擬似的にトリガー
vi.mock('@/components/molecules/CSVFileInput', () => ({
  CSVFileInput: ({ onFileSelect }: { onFileSelect: (file: File) => Promise<void> }) => (
    <button
      data-testid="csv-file-input"
      onClick={() => onFileSelect(new File(['dummy'], 'test.csv'))}
    >
      CSV読込
    </button>
  ),
}));

// ConfirmDeleteModal: 簡略化したモーダル
vi.mock('@/components/molecules/ConfirmDeleteModal/ConfirmDeleteModal', () => ({
  ConfirmDeleteModal: ({
    isOpen,
    onConfirm,
    onCancel,
  }: {
    isOpen: boolean;
    onConfirm: () => void;
    onCancel: () => void;
  }) =>
    isOpen ? (
      <div data-testid="confirm-modal">
        <button data-testid="confirm-delete" onClick={onConfirm}>
          削除する
        </button>
        <button data-testid="cancel-delete" onClick={onCancel}>
          キャンセル
        </button>
      </div>
    ) : null,
}));

// 子コンポーネント: csvData の長さだけ確認できる最小表示
vi.mock('@/pages/Receipt/Dividend', () => ({
  Dividend: ({ csvData }: { csvData: unknown[] }) => (
    <div data-testid="dividend-view">{csvData.length}</div>
  ),
}));
vi.mock('@/pages/Receipt/DomesticStock', () => ({
  DomesticStock: ({ csvData }: { csvData: unknown[] }) => (
    <div data-testid="domesticstock-view">{csvData.length}</div>
  ),
}));
vi.mock('@/pages/Receipt/Mutualfund', () => ({
  Mutualfund: ({ csvData }: { csvData: unknown[] }) => (
    <div data-testid="mutualfund-view">{csvData.length}</div>
  ),
}));

// Receipt API
import * as receiptApi from '@/features/receipt/api/receiptApi';
vi.mock('@/features/receipt/api/receiptApi', () => ({
  dividendApi: {
    list: vi.fn().mockResolvedValue([]),
    bulkCreate: vi.fn().mockResolvedValue({ inserted: 0, skipped: 0 }),
    deleteAll: vi.fn().mockResolvedValue({}),
  },
  domesticStockApi: {
    list: vi.fn().mockResolvedValue([]),
    bulkCreate: vi.fn().mockResolvedValue({ inserted: 0, skipped: 0 }),
    deleteAll: vi.fn().mockResolvedValue({}),
  },
  mutualfundApi: {
    list: vi.fn().mockResolvedValue([]),
    bulkCreate: vi.fn().mockResolvedValue({ inserted: 0, skipped: 0 }),
    deleteAll: vi.fn().mockResolvedValue({}),
  },
}));

// parsers: 入力をそのまま返す
vi.mock('@/features/receipt/parsers', () => ({
  parseDividendCsvItem: vi.fn((item: unknown) => item),
  parseDomesticStockCsvItem: vi.fn((item: unknown) => item),
  parseMutualfundCsvItem: vi.fn((item: unknown) => item),
  transformDBDividend: vi.fn((item: unknown) => item),
  transformDBDomesticStock: vi.fn((item: unknown) => item),
  transformDBMutualfund: vi.fn((item: unknown) => item),
}));

// useCSVReader: parseCSV はデフォルト空配列を返す（テスト内で上書き可能）
const mockParseCSV = vi.fn().mockResolvedValue([]);
vi.mock('@/hooks/useCSVReader', () => ({
  useCSVReader: () => ({
    parseCSV: mockParseCSV,
    isLoading: false,
    error: null,
    fileName: null,
    resetError: vi.fn(),
    reset: vi.fn(),
  }),
}));

// useAuth: ログアウトコールバックをキャプチャ可能にする
import * as authHook from '@/features/auth/hooks/useAuth';
vi.mock('@/features/auth/hooks/useAuth');

// ────────────────────────────────────────────────────────
// ユーティリティ: useAuth のデフォルトモック生成
// ────────────────────────────────────────────────────────
type LogoutCallback = () => void;
type UseAuthReturn = ReturnType<typeof authHook.useAuth>;

function makeAuthMock(opts: {
  isAuthenticated?: boolean;
  userId?: string;
  authLoading?: boolean;
  onLogoutCapture?: (cb: LogoutCallback) => void;
}): UseAuthReturn {
  const { isAuthenticated = false, userId = 'user-1', authLoading = false, onLogoutCapture } = opts;
  return {
    user: isAuthenticated ? { id: userId, email: 'test@example.com' } : null,
    setUser: vi.fn(),
    login: vi.fn(),
    logout: vi.fn().mockResolvedValue(undefined),
    deleteAccount: vi.fn().mockResolvedValue(undefined),
    isAuthenticated,
    isLoading: authLoading,
    onLogout: (cb: LogoutCallback) => {
      onLogoutCapture?.(cb);
      return () => {};
    },
  };
}


// ────────────────────────────────────────────────────────
// receiptsReducer 単体テスト
// ────────────────────────────────────────────────────────

describe('receiptsReducer', () => {
  it('initialState が正しく定義されている', () => {
    expect(initialState.receiptsType).toBe('dividend');
    expect(initialState.csvData.dividend).toHaveLength(0);
    expect(initialState.showDeleteConfirm).toBe(false);
  });

  it('SET_RECEIPTS_TYPE: タブを切り替える', () => {
    const next = receiptsReducer(initialState, {
      type: 'SET_RECEIPTS_TYPE',
      payload: 'domesticstock',
    });
    expect(next.receiptsType).toBe('domesticstock');
  });

  it('SET_CSV_DATA: 指定タブの CSV データのみ更新する', () => {
    const mockRows = [{ col: 'a' }, { col: 'b' }];
    const next = receiptsReducer(initialState, {
      type: 'SET_CSV_DATA',
      receiptsType: 'mutualfund',
      payload: mockRows,
    });
    expect(next.csvData.mutualfund).toBe(mockRows);
    // 他のタブは変更されない
    expect(next.csvData.dividend).toHaveLength(0);
    expect(next.csvData.domesticstock).toHaveLength(0);
  });

  it('LOGOUT: csvData をすべて空にする', () => {
    const dirtyState: ReceiptsState = {
      ...initialState,
      csvData: {
        dividend: [{ x: 1 }],
        domesticstock: [{ x: 2 }],
        mutualfund: [{ x: 3 }],
      },
    };

    const next = receiptsReducer(dirtyState, { type: 'LOGOUT' });

    expect(next.csvData.dividend).toHaveLength(0);
    expect(next.csvData.domesticstock).toHaveLength(0);
    expect(next.csvData.mutualfund).toHaveLength(0);
    // receiptsType は変更されない
    expect(next.receiptsType).toBe(dirtyState.receiptsType);
  });

  it('SET_SHOW_DELETE_CONFIRM が正しく反映される', () => {
    const s1 = receiptsReducer(initialState, { type: 'SET_SHOW_DELETE_CONFIRM', payload: true });
    expect(s1.showDeleteConfirm).toBe(true);

    const s2 = receiptsReducer(s1, { type: 'SET_SHOW_DELETE_CONFIRM', payload: false });
    expect(s2.showDeleteConfirm).toBe(false);
  });
});

// ────────────────────────────────────────────────────────
// ReceiptsPage 統合テスト
// ────────────────────────────────────────────────────────

describe('ReceiptsPage', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockParseCSV.mockResolvedValue([]);
  });

  it('初期表示: 配当金タブが選択されている', () => {
    vi.mocked(authHook.useAuth).mockReturnValue(makeAuthMock({}));

    renderWithQuery(<ReceiptsPage />);

    // 配当金タブが aria-selected=true
    const tab = screen.getByRole('tab', { name: '配当金' });
    expect(tab).toHaveAttribute('aria-selected', 'true');

    // Dividend コンポーネントが表示される
    expect(screen.getByTestId('dividend-view')).toBeInTheDocument();
    expect(screen.queryByTestId('domesticstock-view')).not.toBeInTheDocument();
  });

  it('タブ切替: 国内株式タブをクリックすると DomesticStock が表示される', async () => {
    vi.mocked(authHook.useAuth).mockReturnValue(makeAuthMock({}));

    renderWithQuery(<ReceiptsPage />);
    const user = userEvent.setup();

    await user.click(screen.getByRole('tab', { name: '国内株式' }));

    expect(screen.getByTestId('domesticstock-view')).toBeInTheDocument();
    expect(screen.queryByTestId('dividend-view')).not.toBeInTheDocument();
  });

  it('タブ切替: 投資信託タブをクリックすると Mutualfund が表示される', async () => {
    vi.mocked(authHook.useAuth).mockReturnValue(makeAuthMock({}));

    renderWithQuery(<ReceiptsPage />);
    const user = userEvent.setup();

    await user.click(screen.getByRole('tab', { name: '投資信託' }));

    expect(screen.getByTestId('mutualfund-view')).toBeInTheDocument();
    expect(screen.queryByTestId('dividend-view')).not.toBeInTheDocument();
  });

  it('CSV読込→保存: bulkCreate API が呼ばれ CSV データがクリアされる', async () => {
    vi.mocked(authHook.useAuth).mockReturnValue(
      makeAuthMock({ isAuthenticated: true })
    );
    vi.mocked(receiptApi.dividendApi.list).mockResolvedValue([]);
    vi.mocked(receiptApi.domesticStockApi.list).mockResolvedValue([]);
    vi.mocked(receiptApi.mutualfundApi.list).mockResolvedValue([]);
    vi.mocked(receiptApi.dividendApi.bulkCreate).mockResolvedValue({ inserted: 0, skipped: 0 });

    const mockRows = [{ '入金日': '2023/01/01' }, { '入金日': '2023/02/01' }];
    mockParseCSV.mockResolvedValue(mockRows);

    renderWithQuery(<ReceiptsPage />);

    // DB フェッチ完了を待つ
    await waitFor(() => {
      expect(screen.queryByRole('status')).not.toBeInTheDocument();
    }, waitOpts);

    const user = userEvent.setup();
    await user.click(screen.getByTestId('csv-file-input'));

    await waitFor(() => {
      expect(screen.getByText('保存')).toBeInTheDocument();
    }, waitOpts);

    await user.click(screen.getByText('保存'));

    await waitFor(() => {
      expect(receiptApi.dividendApi.bulkCreate).toHaveBeenCalled();
    }, waitOpts);

    // 保存後は CSV データがクリアされ保存ボタンが消える
    await waitFor(() => {
      expect(screen.queryByText('保存')).not.toBeInTheDocument();
    }, waitOpts);
  }, 20000);

  it('全削除: 確認モーダル経由で deleteAll API が呼ばれデータがクリアされる', async () => {
    vi.mocked(authHook.useAuth).mockReturnValue(
      makeAuthMock({ isAuthenticated: true })
    );

    const mockDbRow = { id: '1', payment_date: '2023-01-01' };
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    vi.mocked(receiptApi.dividendApi.list).mockResolvedValue([mockDbRow] as any);
    vi.mocked(receiptApi.domesticStockApi.list).mockResolvedValue([]);
    vi.mocked(receiptApi.mutualfundApi.list).mockResolvedValue([]);
    vi.mocked(receiptApi.dividendApi.deleteAll).mockResolvedValue({});

    renderWithQuery(<ReceiptsPage />);

    await waitFor(() => {
      expect(screen.queryByRole('status')).not.toBeInTheDocument();
    }, waitOpts);

    await waitFor(() => {
      expect(screen.getByText(/全件削除/)).toBeInTheDocument();
    }, waitOpts);

    const user = userEvent.setup();
    await user.click(screen.getByText(/全件削除/));
    expect(screen.getByTestId('confirm-modal')).toBeInTheDocument();

    await user.click(screen.getByTestId('confirm-delete'));

    await waitFor(() => {
      expect(receiptApi.dividendApi.deleteAll).toHaveBeenCalled();
    }, waitOpts);

    await waitFor(() => {
      expect(screen.queryByText(/全件削除/)).not.toBeInTheDocument();
    }, waitOpts);
  }, 20000);

  it('ログアウト: onLogout コールバック実行で csvData / dbData がクリアされ Query キャッシュが除去される', async () => {
    // AuthContext は複数コールバックをすべて発火する。配列で収集して一括発火することで実際の動作を再現する
    const capturedCallbacks: LogoutCallback[] = [];
    const qc = makeQueryClient();

    vi.mocked(authHook.useAuth).mockReturnValue(
      makeAuthMock({
        isAuthenticated: true,
        onLogoutCapture: (cb) => { capturedCallbacks.push(cb); },
      })
    );

    const mockDbRow = { id: '1', payment_date: '2023-01-01' };
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    vi.mocked(receiptApi.dividendApi.list).mockResolvedValue([mockDbRow] as any);
    vi.mocked(receiptApi.domesticStockApi.list).mockResolvedValue([]);
    vi.mocked(receiptApi.mutualfundApi.list).mockResolvedValue([]);

    renderWithQuery(<ReceiptsPage />, qc);

    await waitFor(() => expect(capturedCallbacks.length).toBeGreaterThan(0));

    await waitFor(() => {
      expect(screen.queryByRole('status')).not.toBeInTheDocument();
    });

    await waitFor(() => {
      expect(screen.getByText(/全件削除/)).toBeInTheDocument();
    });

    act(() => {
      capturedCallbacks.forEach(cb => cb());
      vi.mocked(authHook.useAuth).mockReturnValue(makeAuthMock({ isAuthenticated: false }));
    });

    await waitFor(() => {
      expect(screen.queryByText(/全件削除/)).not.toBeInTheDocument();
    });
    expect(qc.getQueryData(receiptQueryKeys.dividend('user-1'))).toBeUndefined();
    expect(qc.getQueryData(receiptQueryKeys.domesticstock('user-1'))).toBeUndefined();
    expect(qc.getQueryData(receiptQueryKeys.mutualfund('user-1'))).toBeUndefined();
  });
});
