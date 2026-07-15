import { useCallback, useState } from 'react';

export interface UseAssetBalanceDeleteConfirmResult {
  showDeleteConfirm: boolean;
  openDeleteConfirm: () => void;
  closeDeleteConfirm: () => void;
  confirmDeleteAll: () => Promise<void>;
  resetDeleteConfirm: () => void;
}

/**
 * 全件削除確認モーダルの開閉と実行を管理するフック
 */
export function useAssetBalanceDeleteConfirm(
  deleteAll: () => Promise<void>
): UseAssetBalanceDeleteConfirmResult {
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);

  const openDeleteConfirm = useCallback(() => {
    setShowDeleteConfirm(true);
  }, []);

  const closeDeleteConfirm = useCallback(() => {
    setShowDeleteConfirm(false);
  }, []);

  const confirmDeleteAll = useCallback(async () => {
    setShowDeleteConfirm(false);
    await deleteAll();
  }, [deleteAll]);

  const resetDeleteConfirm = useCallback(() => {
    setShowDeleteConfirm(false);
  }, []);

  return {
    showDeleteConfirm,
    openDeleteConfirm,
    closeDeleteConfirm,
    confirmDeleteAll,
    resetDeleteConfirm,
  };
}
