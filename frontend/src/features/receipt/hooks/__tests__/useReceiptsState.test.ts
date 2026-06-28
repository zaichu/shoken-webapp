/**
 * useReceiptsState: 状態管理・ユーザー操作のユニットテスト
 */
import { renderHook, act } from '@testing-library/react';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import { useReceiptsState } from '../useReceiptsState';
import { useAuth } from '@/features/auth/hooks/useAuth';
import { useReceiptsData } from '../useReceiptsData';

// ────────────────────────────────────────────────────────
// モック
// ────────────────────────────────────────────────────────

vi.mock('@/features/auth/hooks/useAuth');
vi.mock('../useReceiptsData');

// useReceiptsState が useEffect 内で onLogout を呼ぶため、
// クリーンアップ関数を返すモックを用意する
function makeOnLogoutMock() {
  return vi.fn((_cb: () => void) => () => {}); // eslint-disable-line @typescript-eslint/no-unused-vars
}

const mockUploadCsv = vi.fn();
const mockPreviewCsv = vi.fn();
const mockDeleteAll = vi.fn();

function setupMocks(isAuthenticated = true) {
  vi.mocked(useAuth).mockReturnValue({
    user: isAuthenticated ? { id: 'user-1', email: 'test@example.com' } : null,
    setUser: vi.fn(),
    login: vi.fn(),
    logout: vi.fn().mockResolvedValue(undefined),
    deleteAccount: vi.fn().mockResolvedValue(undefined),
    isAuthenticated,
    isLoading: false,
    onLogout: makeOnLogoutMock(),
  });

  vi.mocked(useReceiptsData).mockReturnValue({
    dividendData: [],
    domesticstockData: [],
    mutualfundData: [],
    dbLoading: false,
    dbError: null,
    saving: false,
    deleting: false,
    previewing: false,
    uploadCsv: mockUploadCsv,
    previewCsv: mockPreviewCsv,
    deleteAll: mockDeleteAll,
  });
}

// ────────────────────────────────────────────────────────
// テスト
// ────────────────────────────────────────────────────────

describe('useReceiptsState', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    setupMocks();
  });

  it('初期状態: receiptsType が dividend で showDeleteConfirm が false', () => {
    const { result } = renderHook(() => useReceiptsState());

    expect(result.current.receiptsType).toBe('dividend');
    expect(result.current.showDeleteConfirm).toBe(false);
  });

  it('setReceiptsType: 呼び出すと receiptsType が変わる', () => {
    const { result } = renderHook(() => useReceiptsState());

    act(() => {
      result.current.setReceiptsType('domesticstock');
    });

    expect(result.current.receiptsType).toBe('domesticstock');

    act(() => {
      result.current.setReceiptsType('mutualfund');
    });

    expect(result.current.receiptsType).toBe('mutualfund');
  });

  it('handleTabKeyDown ArrowRight: 次のタブに移動する', () => {
    const { result } = renderHook(() => useReceiptsState());

    // 初期は dividend (index=0)
    act(() => {
      result.current.handleTabKeyDown({
        key: 'ArrowRight',
        preventDefault: vi.fn(),
      } as unknown as React.KeyboardEvent<HTMLButtonElement>);
    });

    // dividend → domesticstock (index=1)
    expect(result.current.receiptsType).toBe('domesticstock');
  });

  it('handleTabKeyDown ArrowLeft: 前のタブ（循環）に移動する', () => {
    const { result } = renderHook(() => useReceiptsState());

    // 初期は dividend (index=0), ArrowLeft で末尾 mutualfund (index=2) に循環
    act(() => {
      result.current.handleTabKeyDown({
        key: 'ArrowLeft',
        preventDefault: vi.fn(),
      } as unknown as React.KeyboardEvent<HTMLButtonElement>);
    });

    expect(result.current.receiptsType).toBe('mutualfund');
  });

  it('openDeleteConfirm / closeDeleteConfirm: showDeleteConfirm のトグル', () => {
    const { result } = renderHook(() => useReceiptsState());

    // utilityRailProps.actionRailProps.onDeleteRequest が openDeleteConfirm に対応
    act(() => {
      result.current.utilityRailProps.actionRailProps.onDeleteRequest();
    });

    expect(result.current.showDeleteConfirm).toBe(true);

    act(() => {
      result.current.closeDeleteConfirm();
    });

    expect(result.current.showDeleteConfirm).toBe(false);
  });

  it('confirmDeleteAll: 認証済みなら deleteAll が呼ばれる', () => {
    const { result } = renderHook(() => useReceiptsState());

    act(() => {
      result.current.confirmDeleteAll();
    });

    expect(mockDeleteAll).toHaveBeenCalledWith(
      'dividend',
      expect.objectContaining({ onSuccess: expect.any(Function) }),
    );
  });

  it('confirmDeleteAll: 未認証なら deleteAll が呼ばれない', () => {
    setupMocks(false);
    const { result } = renderHook(() => useReceiptsState());

    act(() => {
      result.current.confirmDeleteAll();
    });

    expect(mockDeleteAll).not.toHaveBeenCalled();
  });

  it('handleFileSelect: previewCsv が呼ばれ onSuccess で csvPreview がセットされる', () => {
    const { result } = renderHook(() => useReceiptsState());

    const file = new File([''], 'test.csv');

    act(() => {
      result.current.utilityRailProps.actionRailProps.onFileSelect(file);
    });

    // previewCsv が呼ばれていること
    expect(mockPreviewCsv).toHaveBeenCalledWith(
      expect.objectContaining({
        type: 'dividend',
        file,
        onSuccess: expect.any(Function),
      }),
    );

    // onSuccess を手動で呼び出して csvPreview がセットされることを確認
    const onSuccess = vi.mocked(mockPreviewCsv).mock.calls[0][0].onSuccess as (
      result: { totalRows: number; validRows: number; errors: unknown[]; rows: unknown[] },
    ) => void;

    const csvPreviewData = {
      totalRows: 3,
      validRows: 2,
      errors: [],
      rows: [{ id: '1' }, { id: '2' }],
    };

    act(() => {
      onSuccess(csvPreviewData);
    });

    // utilityRailProps.alertsProps.csvPreview にプレビューがセットされている
    expect(result.current.utilityRailProps.alertsProps.csvPreview).toEqual(csvPreviewData);
  });
});
