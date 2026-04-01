/**
 * ReceiptsPage のテスト
 *
 * 対象:
 * - receiptsReducer: 状態遷移の単体テスト（LOGOUT, SET_CSV_DATA 等）
 * - ReceiptsPage: タブ切替・ログアウト時データクリアの統合テスト
 */
import React from 'react';
import { screen, waitFor, act, fireEvent } from '@testing-library/react';
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

// 子コンポーネント: data の長さ確認 + importResult が渡された場合は表示（回帰検知用）
vi.mock('@/pages/Receipt/Dividend', () => ({
  Dividend: ({ data, importResult, utilityRail }: { data: unknown[]; importResult?: { inserted: number }; utilityRail?: React.ReactNode }) => (
    <div data-testid="dividend-view">
      {utilityRail}
      {data.length}
      {importResult && <strong>{importResult.inserted}件登録</strong>}
    </div>
  ),
}));
vi.mock('@/pages/Receipt/DomesticStock', () => ({
  DomesticStock: ({ data, importResult, utilityRail }: { data: unknown[]; importResult?: { inserted: number }; utilityRail?: React.ReactNode }) => (
    <div data-testid="domesticstock-view">
      {utilityRail}
      {data.length}
      {importResult && <strong>{importResult.inserted}件登録</strong>}
    </div>
  ),
}));
vi.mock('@/pages/Receipt/Mutualfund', () => ({
  Mutualfund: ({ data, importResult, utilityRail }: { data: unknown[]; importResult?: { inserted: number }; utilityRail?: React.ReactNode }) => (
    <div data-testid="mutualfund-view">
      {utilityRail}
      {data.length}
      {importResult && <strong>{importResult.inserted}件登録</strong>}
    </div>
  ),
}));

// Receipt API
import * as receiptApi from '@/features/receipt/api/receiptApi';
vi.mock('@/features/receipt/api/receiptApi', () => ({
  dividendApi: {
    list: vi.fn().mockResolvedValue([]),
    previewCsv: vi.fn().mockResolvedValue({ total_rows: 0, valid_rows: 0, errors: [] }),
    uploadCsv: vi.fn().mockResolvedValue({ inserted: 0, skipped: 0, errors: [] }),
    deleteAll: vi.fn().mockResolvedValue({}),
  },
  domesticStockApi: {
    list: vi.fn().mockResolvedValue([]),
    previewCsv: vi.fn().mockResolvedValue({ total_rows: 0, valid_rows: 0, errors: [] }),
    uploadCsv: vi.fn().mockResolvedValue({ inserted: 0, skipped: 0, errors: [] }),
    deleteAll: vi.fn().mockResolvedValue({}),
  },
  mutualfundApi: {
    list: vi.fn().mockResolvedValue([]),
    previewCsv: vi.fn().mockResolvedValue({ total_rows: 0, valid_rows: 0, errors: [] }),
    uploadCsv: vi.fn().mockResolvedValue({ inserted: 0, skipped: 0, errors: [] }),
    deleteAll: vi.fn().mockResolvedValue({}),
  },
}));

// parsers: sort 関数はそのまま通す、transformDB は identity
vi.mock('@/features/receipt/parsers', () => ({
  sortDividendBySettlementDate: vi.fn((data: unknown[]) => data),
  sortDomesticStockByTradeDate: vi.fn((data: unknown[]) => data),
  sortMutualfundByTradeDate: vi.fn((data: unknown[]) => data),
  transformDBDividend: vi.fn((item: unknown) => item),
  transformDBDomesticStock: vi.fn((item: unknown) => item),
  transformDBMutualfund: vi.fn((item: unknown) => item),
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
    expect(initialState.rawFiles.dividend).toBeNull();
    expect(initialState.showDeleteConfirm).toBe(false);
  });

  it('SET_RECEIPTS_TYPE: タブを切り替える', () => {
    const next = receiptsReducer(initialState, {
      type: 'SET_RECEIPTS_TYPE',
      payload: 'domesticstock',
    });
    expect(next.receiptsType).toBe('domesticstock');
  });

  it('SET_RAW_FILE: 指定タブのファイルのみ更新し、取り込み結果をクリアする', () => {
    const file = new File([''], 'test.csv');
    const next = receiptsReducer(initialState, {
      type: 'SET_RAW_FILE',
      receiptsType: 'mutualfund',
      payload: file,
    });
    expect(next.rawFiles.mutualfund).toBe(file);
    expect(next.rawFiles.dividend).toBeNull();
    expect(next.lastImportResults.mutualfund).toBeNull();
  });

  it('LOGOUT: rawFiles と lastImportResults をリセットする', () => {
    const file = new File([''], 'test.csv');
    const dirtyState: ReceiptsState = {
      ...initialState,
      rawFiles: { dividend: file, domesticstock: null, mutualfund: null },
      lastImportResults: {
        dividend: { inserted: 1, skipped: 0, errors: [] },
        domesticstock: null,
        mutualfund: null,
      },
    };

    const next = receiptsReducer(dirtyState, { type: 'LOGOUT' });

    expect(next.rawFiles.dividend).toBeNull();
    expect(next.lastImportResults.dividend).toBeNull();
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
  });

  async function setupKeyboardNavigationTest() {
    vi.mocked(authHook.useAuth).mockReturnValue(makeAuthMock({ isAuthenticated: true }));
    vi.mocked(receiptApi.dividendApi.list).mockResolvedValue([] as never[]);
    vi.mocked(receiptApi.domesticStockApi.list).mockResolvedValue([] as never[]);
    vi.mocked(receiptApi.mutualfundApi.list).mockResolvedValue([] as never[]);

    renderWithQuery(<ReceiptsPage />);

    await waitFor(() => {
      expect(receiptApi.dividendApi.list).toHaveBeenCalled();
    }, waitOpts);

    return {
      user: userEvent.setup(),
      getDividendTab: () => screen.getByRole('tab', { name: /^配当金/ }),
      getDomesticStockTab: () => screen.getByRole('tab', { name: /^国内株式/ }),
      getMutualfundTab: () => screen.getByRole('tab', { name: /^投資信託/ }),
    };
  }

  it('初期表示: 配当金タブが選択されている', () => {
    vi.mocked(authHook.useAuth).mockReturnValue(makeAuthMock({}));

    renderWithQuery(<ReceiptsPage />);

    // 配当金タブが aria-selected=true
    const tab = screen.getByRole('tab', { name: /^配当金/ });
    expect(tab).toHaveAttribute('aria-selected', 'true');

    // Dividend コンポーネントが表示される
    expect(screen.getByTestId('dividend-view')).toBeInTheDocument();
    expect(screen.queryByTestId('domesticstock-view')).not.toBeInTheDocument();
  });

  it('タブ切替: 国内株式タブをクリックすると DomesticStock が表示される', async () => {
    vi.mocked(authHook.useAuth).mockReturnValue(makeAuthMock({}));

    renderWithQuery(<ReceiptsPage />);
    const user = userEvent.setup();

    await user.click(screen.getByRole('tab', { name: /^国内株式/ }));

    expect(screen.getByTestId('domesticstock-view')).toBeInTheDocument();
    expect(screen.queryByTestId('dividend-view')).not.toBeInTheDocument();
  });

  it('タブ切替: 投資信託タブをクリックすると Mutualfund が表示される', async () => {
    vi.mocked(authHook.useAuth).mockReturnValue(makeAuthMock({}));

    renderWithQuery(<ReceiptsPage />);
    const user = userEvent.setup();

    await user.click(screen.getByRole('tab', { name: /^投資信託/ }));

    expect(screen.getByTestId('mutualfund-view')).toBeInTheDocument();
    expect(screen.queryByTestId('dividend-view')).not.toBeInTheDocument();
  });

  describe('タブキーボードナビゲーション', () => {
    it('ArrowRight でタブが次に移動する', async () => {
      const { getDividendTab, getDomesticStockTab } = await setupKeyboardNavigationTest();

      const dividendTab = getDividendTab();
      dividendTab.focus();
      fireEvent.keyDown(dividendTab, { key: 'ArrowRight' });

      await waitFor(() => {
        expect(screen.getByRole('tab', { selected: true, name: /^国内株式/ })).toBeInTheDocument();
        expect(getDomesticStockTab()).toHaveFocus();
      }, waitOpts);

      expect(screen.getByTestId('domesticstock-view')).toBeInTheDocument();
      expect(screen.queryByTestId('dividend-view')).not.toBeInTheDocument();
    });

    it('ArrowLeft でタブが前に移動する', async () => {
      const { user, getDividendTab, getDomesticStockTab } = await setupKeyboardNavigationTest();

      await user.click(getDomesticStockTab());
      await waitFor(() => {
        expect(screen.getByRole('tab', { selected: true, name: /^国内株式/ })).toBeInTheDocument();
      }, waitOpts);

      const domesticStockTab = getDomesticStockTab();
      domesticStockTab.focus();
      fireEvent.keyDown(domesticStockTab, { key: 'ArrowLeft' });

      await waitFor(() => {
        expect(screen.getByRole('tab', { selected: true, name: /^配当金/ })).toBeInTheDocument();
        expect(getDividendTab()).toHaveFocus();
      }, waitOpts);

      expect(screen.getByTestId('dividend-view')).toBeInTheDocument();
      expect(screen.queryByTestId('domesticstock-view')).not.toBeInTheDocument();
    });

    it('ArrowRight が最後のタブでラップアラウンドする', async () => {
      const { user, getDividendTab, getMutualfundTab } = await setupKeyboardNavigationTest();

      await user.click(getMutualfundTab());
      await waitFor(() => {
        expect(screen.getByRole('tab', { selected: true, name: /^投資信託/ })).toBeInTheDocument();
      }, waitOpts);

      const mutualfundTab = getMutualfundTab();
      mutualfundTab.focus();
      fireEvent.keyDown(mutualfundTab, { key: 'ArrowRight' });

      await waitFor(() => {
        expect(screen.getByRole('tab', { selected: true, name: /^配当金/ })).toBeInTheDocument();
        expect(getDividendTab()).toHaveFocus();
      }, waitOpts);

      expect(screen.getByTestId('dividend-view')).toBeInTheDocument();
      expect(screen.queryByTestId('mutualfund-view')).not.toBeInTheDocument();
    });

    it('ArrowLeft が最初のタブでラップアラウンドする', async () => {
      const { getDividendTab, getMutualfundTab } = await setupKeyboardNavigationTest();

      const dividendTab = getDividendTab();
      dividendTab.focus();
      fireEvent.keyDown(dividendTab, { key: 'ArrowLeft' });

      await waitFor(() => {
        expect(screen.getByRole('tab', { selected: true, name: /^投資信託/ })).toBeInTheDocument();
        expect(getMutualfundTab()).toHaveFocus();
      }, waitOpts);

      expect(screen.getByTestId('mutualfund-view')).toBeInTheDocument();
      expect(screen.queryByTestId('dividend-view')).not.toBeInTheDocument();
    });

    it('Home キーで最初のタブに移動する', async () => {
      const { user, getDividendTab, getMutualfundTab } = await setupKeyboardNavigationTest();

      await user.click(getMutualfundTab());
      await waitFor(() => {
        expect(screen.getByRole('tab', { selected: true, name: /^投資信託/ })).toBeInTheDocument();
      }, waitOpts);

      const mutualfundTab = getMutualfundTab();
      mutualfundTab.focus();
      fireEvent.keyDown(mutualfundTab, { key: 'Home' });

      await waitFor(() => {
        expect(screen.getByRole('tab', { selected: true, name: /^配当金/ })).toBeInTheDocument();
        expect(getDividendTab()).toHaveFocus();
      }, waitOpts);
    });

    it('End キーで最後のタブに移動する', async () => {
      const { getDividendTab, getMutualfundTab } = await setupKeyboardNavigationTest();

      const dividendTab = getDividendTab();
      dividendTab.focus();
      fireEvent.keyDown(dividendTab, { key: 'End' });

      await waitFor(() => {
        expect(screen.getByRole('tab', { selected: true, name: /^投資信託/ })).toBeInTheDocument();
        expect(getMutualfundTab()).toHaveFocus();
      }, waitOpts);
    });

    it('関係ないキーではタブが変わらない', async () => {
      const { getDividendTab } = await setupKeyboardNavigationTest();

      const dividendTab = getDividendTab();
      dividendTab.focus();
      fireEvent.keyDown(dividendTab, { key: 'Space' });

      await waitFor(() => {
        expect(screen.getByRole('tab', { selected: true, name: /^配当金/ })).toBeInTheDocument();
      }, waitOpts);

      expect(getDividendTab()).toHaveFocus();
      expect(screen.getByTestId('dividend-view')).toBeInTheDocument();
      expect(screen.queryByTestId('domesticstock-view')).not.toBeInTheDocument();
      expect(screen.queryByTestId('mutualfund-view')).not.toBeInTheDocument();
    });
  });

  it('CSV読込→保存: uploadCsv API が呼ばれ保存ボタンが消える', async () => {
    vi.mocked(authHook.useAuth).mockReturnValue(
      makeAuthMock({ isAuthenticated: true })
    );
    vi.mocked(receiptApi.dividendApi.list).mockResolvedValue([]);
    vi.mocked(receiptApi.domesticStockApi.list).mockResolvedValue([]);
    vi.mocked(receiptApi.mutualfundApi.list).mockResolvedValue([]);
    vi.mocked(receiptApi.dividendApi.uploadCsv).mockResolvedValue({ inserted: 0, skipped: 0, errors: [] });

    renderWithQuery(<ReceiptsPage />);

    await waitFor(() => {
      expect(receiptApi.dividendApi.list).toHaveBeenCalled();
    }, waitOpts);

    await waitFor(() => {
      expect(screen.getByTestId('csv-file-input')).toBeInTheDocument();
    }, waitOpts);

    const user = userEvent.setup();
    await user.click(screen.getByTestId('csv-file-input'));

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /追加で保存/ })).toBeInTheDocument();
    }, waitOpts);

    await user.click(screen.getByRole('button', { name: /追加で保存/ }));

    await waitFor(() => {
      expect(receiptApi.dividendApi.uploadCsv).toHaveBeenCalled();
    }, waitOpts);

    // 保存後は CSV データがクリアされ保存ボタンが消える
    await waitFor(() => {
      expect(screen.queryByRole('button', { name: /追加で保存/ })).not.toBeInTheDocument();
    }, waitOpts);
  }, 20000);

  it('取込結果: right rail に軽い confirmation strip が1箇所だけ表示される', async () => {
    vi.mocked(authHook.useAuth).mockReturnValue(
      makeAuthMock({ isAuthenticated: true })
    );
    vi.mocked(receiptApi.dividendApi.list).mockResolvedValue([]);
    vi.mocked(receiptApi.domesticStockApi.list).mockResolvedValue([]);
    vi.mocked(receiptApi.mutualfundApi.list).mockResolvedValue([]);
    vi.mocked(receiptApi.dividendApi.uploadCsv).mockResolvedValue({ inserted: 3, skipped: 2, errors: [] });

    renderWithQuery(<ReceiptsPage />);

    await waitFor(() => {
      expect(receiptApi.dividendApi.list).toHaveBeenCalled();
    }, waitOpts);

    await waitFor(() => {
      expect(screen.getByTestId('csv-file-input')).toBeInTheDocument();
    }, waitOpts);

    const user = userEvent.setup();
    await user.click(screen.getByTestId('csv-file-input'));

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /追加で保存/ })).toBeInTheDocument();
    }, waitOpts);

    await user.click(screen.getByRole('button', { name: /追加で保存/ }));

    await waitFor(() => {
      expect(receiptApi.dividendApi.uploadCsv).toHaveBeenCalled();
    }, waitOpts);

    await waitFor(() => {
      expect(screen.getByText('保存しました')).toBeInTheDocument();
    }, waitOpts);

    expect(screen.getByText('3件反映')).toBeInTheDocument();
    expect(screen.getByText('追加保存')).toBeInTheDocument();
    expect(screen.getByText('2件スキップ')).toBeInTheDocument();
    expect(screen.getAllByText('保存しました')).toHaveLength(1);
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
      expect(receiptApi.dividendApi.list).toHaveBeenCalled();
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
      expect(receiptApi.dividendApi.list).toHaveBeenCalled();
    }, waitOpts);

    await waitFor(() => {
      expect(screen.getByText(/全件削除/)).toBeInTheDocument();
    }, waitOpts);

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

  it('desktop向けworkspaceレイアウトとタブ件数が表示される', async () => {
    vi.mocked(authHook.useAuth).mockReturnValue(makeAuthMock({ isAuthenticated: true }));
    vi.mocked(receiptApi.dividendApi.list).mockResolvedValue([
      { id: 'd1', payment_date: '2025-01-01' },
      { id: 'd2', payment_date: '2025-02-01' },
    ] as never[]);
    vi.mocked(receiptApi.domesticStockApi.list).mockResolvedValue([
      { id: 's1', trade_date: '2025-01-01' },
    ] as never[]);
    vi.mocked(receiptApi.mutualfundApi.list).mockResolvedValue([] as never[]);

    renderWithQuery(<ReceiptsPage />);

    await waitFor(() => {
      expect(receiptApi.dividendApi.list).toHaveBeenCalled();
    }, waitOpts);

    expect(screen.getByTestId('receipts-workspace')).toBeInTheDocument();
    await waitFor(() => {
      expect(screen.getByTestId('tab-count-dividend')).toHaveTextContent('2');
      expect(screen.getByTestId('tab-count-domesticstock')).toHaveTextContent('1');
      expect(screen.getByTestId('tab-count-mutualfund')).toHaveTextContent('0');
    }, waitOpts);

    expect(screen.getByRole('tablist').className).toContain('border-b');
    expect(screen.getByRole('tab', { name: /^配当金/ }).className).toContain('rounded-t-2xl');
  });
});
