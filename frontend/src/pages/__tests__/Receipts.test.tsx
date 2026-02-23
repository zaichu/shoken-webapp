/**
 * ReceiptsPage のテスト
 *
 * 対象:
 * - receiptsReducer: 状態遷移の単体テスト（LOGOUT, SET_DB_ALL, SET_CSV_DATA 等）
 * - ReceiptsPage: タブ切替・ログアウト時データクリアの統合テスト
 */
import React from 'react';
import { render, screen, waitFor, act } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { vi, describe, it, expect, beforeEach } from 'vitest';

import { receiptsReducer, initialState } from '../receiptsReducer';
import type { ReceiptsState } from '../receiptsReducer';
import { ReceiptsPage } from '../Receipts';

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
    bulkCreate: vi.fn().mockResolvedValue({}),
    deleteAll: vi.fn().mockResolvedValue({}),
  },
  domesticStockApi: {
    list: vi.fn().mockResolvedValue([]),
    bulkCreate: vi.fn().mockResolvedValue({}),
    deleteAll: vi.fn().mockResolvedValue({}),
  },
  mutualfundApi: {
    list: vi.fn().mockResolvedValue([]),
    bulkCreate: vi.fn().mockResolvedValue({}),
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
  authLoading?: boolean;
  onLogoutCapture?: (cb: LogoutCallback) => void;
}): UseAuthReturn {
  const { isAuthenticated = false, authLoading = false, onLogoutCapture } = opts;
  return {
    user: null,
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
    expect(initialState.dbData.dividend).toHaveLength(0);
    expect(initialState.dbLoading).toBe(false);
    expect(initialState.dbError).toBeNull();
  });

  it('SET_RECEIPTS_TYPE: タブを切り替える', () => {
    const next = receiptsReducer(initialState, {
      type: 'SET_RECEIPTS_TYPE',
      payload: 'domesticstock',
    });
    expect(next.receiptsType).toBe('domesticstock');
  });

  it('SET_DB_ALL: 3 種類のDBデータを一括セットする', () => {
    const mockDividend = [{ id: '1' }] as unknown as ReceiptsState['dbData']['dividend'];
    const mockDomesticstock = [{ id: '2' }] as unknown as ReceiptsState['dbData']['domesticstock'];
    const mockMutualfund = [{ id: '3' }] as unknown as ReceiptsState['dbData']['mutualfund'];

    const next = receiptsReducer(initialState, {
      type: 'SET_DB_ALL',
      dividend: mockDividend,
      domesticstock: mockDomesticstock,
      mutualfund: mockMutualfund,
    });

    expect(next.dbData.dividend).toBe(mockDividend);
    expect(next.dbData.domesticstock).toBe(mockDomesticstock);
    expect(next.dbData.mutualfund).toBe(mockMutualfund);
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

  it('LOGOUT: csvData / dbData をすべて空にし dbError をクリアする', () => {
    const dirtyState: ReceiptsState = {
      ...initialState,
      csvData: {
        dividend: [{ x: 1 }],
        domesticstock: [{ x: 2 }],
        mutualfund: [{ x: 3 }],
      },
      dbData: {
        dividend: [{ id: 'a' } as unknown as ReceiptsState['dbData']['dividend'][number]],
        domesticstock: [{ id: 'b' } as unknown as ReceiptsState['dbData']['domesticstock'][number]],
        mutualfund: [{ id: 'c' } as unknown as ReceiptsState['dbData']['mutualfund'][number]],
      },
      dbError: 'なんらかのエラー',
    };

    const next = receiptsReducer(dirtyState, { type: 'LOGOUT' });

    expect(next.csvData.dividend).toHaveLength(0);
    expect(next.csvData.domesticstock).toHaveLength(0);
    expect(next.csvData.mutualfund).toHaveLength(0);
    expect(next.dbData.dividend).toHaveLength(0);
    expect(next.dbData.domesticstock).toHaveLength(0);
    expect(next.dbData.mutualfund).toHaveLength(0);
    expect(next.dbError).toBeNull();
    // receiptsType は変更されない
    expect(next.receiptsType).toBe(dirtyState.receiptsType);
  });

  it('SET_DB_ERROR / SET_DB_LOADING が正しく反映される', () => {
    const s1 = receiptsReducer(initialState, { type: 'SET_DB_LOADING', payload: true });
    expect(s1.dbLoading).toBe(true);

    const s2 = receiptsReducer(s1, { type: 'SET_DB_ERROR', payload: 'エラーメッセージ' });
    expect(s2.dbError).toBe('エラーメッセージ');
    expect(s2.dbLoading).toBe(true); // 他フィールドは不変

    const s3 = receiptsReducer(s2, { type: 'SET_DB_LOADING', payload: false });
    expect(s3.dbLoading).toBe(false);
    expect(s3.dbError).toBe('エラーメッセージ'); // クリアされない
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

    render(<ReceiptsPage />);

    // 配当金タブが aria-selected=true
    const tab = screen.getByRole('tab', { name: '配当金' });
    expect(tab).toHaveAttribute('aria-selected', 'true');

    // Dividend コンポーネントが表示される
    expect(screen.getByTestId('dividend-view')).toBeInTheDocument();
    expect(screen.queryByTestId('domesticstock-view')).not.toBeInTheDocument();
  });

  it('タブ切替: 国内株式タブをクリックすると DomesticStock が表示される', async () => {
    vi.mocked(authHook.useAuth).mockReturnValue(makeAuthMock({}));

    render(<ReceiptsPage />);
    const user = userEvent.setup();

    await user.click(screen.getByRole('tab', { name: '国内株式' }));

    expect(screen.getByTestId('domesticstock-view')).toBeInTheDocument();
    expect(screen.queryByTestId('dividend-view')).not.toBeInTheDocument();
  });

  it('タブ切替: 投資信託タブをクリックすると Mutualfund が表示される', async () => {
    vi.mocked(authHook.useAuth).mockReturnValue(makeAuthMock({}));

    render(<ReceiptsPage />);
    const user = userEvent.setup();

    await user.click(screen.getByRole('tab', { name: '投資信託' }));

    expect(screen.getByTestId('mutualfund-view')).toBeInTheDocument();
    expect(screen.queryByTestId('dividend-view')).not.toBeInTheDocument();
  });

  it('CSV読込→保存: bulkCreate API が呼ばれ CSV データがクリアされる', async () => {
    // 未認証時は保存ボタンが出ないため、認証済みとする
    vi.mocked(authHook.useAuth).mockReturnValue(
      makeAuthMock({ isAuthenticated: true })
    );
    // 初期DBデータなし
    vi.mocked(receiptApi.dividendApi.list).mockResolvedValue([]);
    vi.mocked(receiptApi.domesticStockApi.list).mockResolvedValue([]);
    vi.mocked(receiptApi.mutualfundApi.list).mockResolvedValue([]);
    // bulkCreate 後の再フェッチもなし
    vi.mocked(receiptApi.dividendApi.bulkCreate).mockResolvedValue({ inserted: 0, skipped: 0 });

    // parseCSV が 2 件を返すように設定
    const mockRows = [{ '入金日': '2023/01/01' }, { '入金日': '2023/02/01' }];
    mockParseCSV.mockResolvedValue(mockRows);

    render(<ReceiptsPage />);

    // DB フェッチ完了を待つ
    await waitFor(() => {
      expect(screen.queryByRole('status')).not.toBeInTheDocument();
    });

    // CSV 読込ボタンをクリック
    const user = userEvent.setup();
    await user.click(screen.getByTestId('csv-file-input'));

    // 保存ボタンが現れるまで待つ
    await waitFor(() => {
      expect(screen.getByText('保存')).toBeInTheDocument();
    });

    // 保存ボタンをクリック
    await user.click(screen.getByText('保存'));

    // bulkCreate が呼ばれたことを確認
    await waitFor(() => {
      expect(receiptApi.dividendApi.bulkCreate).toHaveBeenCalled();
    });

    // 保存後は CSV データがクリアされ保存ボタンが消える
    await waitFor(() => {
      expect(screen.queryByText('保存')).not.toBeInTheDocument();
    });
  });

  it('全削除: 確認モーダル経由で deleteAll API が呼ばれデータがクリアされる', async () => {
    vi.mocked(authHook.useAuth).mockReturnValue(
      makeAuthMock({ isAuthenticated: true })
    );

    // 初期 DB に 1 件のデータを返す
    const mockDbRow = { id: '1', payment_date: '2023-01-01' } as unknown as ReceiptsState['dbData']['dividend'][number];
    vi.mocked(receiptApi.dividendApi.list).mockResolvedValue([mockDbRow]);
    vi.mocked(receiptApi.domesticStockApi.list).mockResolvedValue([]);
    vi.mocked(receiptApi.mutualfundApi.list).mockResolvedValue([]);
    vi.mocked(receiptApi.dividendApi.deleteAll).mockResolvedValue({});

    render(<ReceiptsPage />);

    // DB データ読み込み完了まで待つ
    await waitFor(() => {
      expect(screen.queryByRole('status')).not.toBeInTheDocument();
    });

    // 「全件削除」ボタンが表示されるまで待つ
    await waitFor(() => {
      expect(screen.getByText(/全件削除/)).toBeInTheDocument();
    });

    // 削除ボタンをクリック → 確認モーダルが出る
    const user = userEvent.setup();
    await user.click(screen.getByText(/全件削除/));
    expect(screen.getByTestId('confirm-modal')).toBeInTheDocument();

    // モーダルの確認ボタンをクリック
    await user.click(screen.getByTestId('confirm-delete'));

    // deleteAll API が呼ばれたことを確認
    await waitFor(() => {
      expect(receiptApi.dividendApi.deleteAll).toHaveBeenCalled();
    });

    // 削除後は全件削除ボタンが消える（データ 0 件）
    await waitFor(() => {
      expect(screen.queryByText(/全件削除/)).not.toBeInTheDocument();
    });
  });

  it('ログアウト: onLogout コールバック実行で csvData / dbData がクリアされる', async () => {
    let capturedLogoutCallback: LogoutCallback | null = null;

    vi.mocked(authHook.useAuth).mockReturnValue(
      makeAuthMock({
        isAuthenticated: true,
        onLogoutCapture: (cb) => { capturedLogoutCallback = cb; },
      })
    );

    // DB に 1 件のデータを返す
    const mockDbRow = { id: '1', payment_date: '2023-01-01' } as unknown as ReceiptsState['dbData']['dividend'][number];
    vi.mocked(receiptApi.dividendApi.list).mockResolvedValue([mockDbRow]);
    vi.mocked(receiptApi.domesticStockApi.list).mockResolvedValue([]);
    vi.mocked(receiptApi.mutualfundApi.list).mockResolvedValue([]);

    render(<ReceiptsPage />);

    // ログアウトコールバックが登録されるまで待つ
    await waitFor(() => expect(capturedLogoutCallback).not.toBeNull());

    // DB フェッチ完了を待つ
    await waitFor(() => {
      expect(screen.queryByRole('status')).not.toBeInTheDocument();
    });

    // ログアウト前: 全件削除ボタンが存在する（DB データあり）
    await waitFor(() => {
      expect(screen.getByText(/全件削除/)).toBeInTheDocument();
    });

    // ログアウトコールバックを実行
    act(() => { capturedLogoutCallback!(); });

    // ログアウト後: データがクリアされ全件削除ボタンが消える
    await waitFor(() => {
      expect(screen.queryByText(/全件削除/)).not.toBeInTheDocument();
    });
  });
});
