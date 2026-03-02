import React, { useEffect, useRef } from 'react';
import { Button } from '@/components/atoms/Button';

interface ConfirmDeleteModalProps {
  isOpen: boolean;
  onConfirm: () => void;
  onCancel: () => void;
  title: string;
  description: string;
  itemCount: number;
  confirmLabel?: string;
  /** 削除処理中フラグ（多重実行防止） */
  loading?: boolean;
}

/**
 * 削除確認モーダル
 * 破壊的操作の実行前に影響範囲を明示し、誤操作を防止する
 */
export const ConfirmDeleteModal: React.FC<ConfirmDeleteModalProps> = ({
  isOpen,
  onConfirm,
  onCancel,
  title,
  description,
  itemCount,
  confirmLabel = '削除する',
  loading = false,
}) => {
  const dialogRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (isOpen) {
      dialogRef.current?.focus();
    }
  }, [isOpen]);

  if (!isOpen) return null;

  return (
    <div
      ref={dialogRef}
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
      onClick={onCancel}
      onKeyDown={(e) => {
        if (e.key === 'Escape') {
          onCancel();
          return;
        }
        // Tab フォーカストラップ: dialog 全体で一元管理し、フォーカス逸脱を防ぐ
        if (e.key === 'Tab') {
          const focusable = e.currentTarget.querySelectorAll<HTMLElement>(
            'button:not([disabled]), [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
          );
          if (focusable.length === 0) return;
          const first = focusable[0];
          const last = focusable[focusable.length - 1];
          const active = document.activeElement as HTMLElement;
          const isInTrap = Array.from(focusable).includes(active);
          if (!isInTrap) {
            // dialog 自体にフォーカスがある等、トラップ対象外の場合: Shift+Tab は末尾、Tab は先頭へ
            e.preventDefault();
            (e.shiftKey ? last : first).focus();
          } else if (e.shiftKey ? active === first : active === last) {
            e.preventDefault();
            (e.shiftKey ? last : first).focus();
          }
        }
      }}
      role="dialog"
      aria-modal="true"
      aria-labelledby="confirm-delete-title"
      aria-describedby="confirm-delete-desc"
      tabIndex={-1}
    >
      <div
        className="w-full max-w-md"
        role="presentation"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="rounded-lg bg-white shadow-lg">
          <div className="flex items-center justify-between border-b border-border px-4 py-3">
            <h5 id="confirm-delete-title" className="text-danger font-semibold">
              {title}
            </h5>
            <button
              type="button"
              className="text-gray-500 hover:text-gray-700 focus:outline-none focus:ring-2 focus:ring-primary/50 rounded"
              onClick={onCancel}
              aria-label="閉じる"
            >
              <span aria-hidden="true">✕</span>
            </button>
          </div>
          <div id="confirm-delete-desc" className="px-4 py-4 text-base text-dark">
            <p>{description}</p>
            <p className="mt-2 text-sm text-secondary">
              対象: <strong className="text-danger">{itemCount}件</strong>のデータ
            </p>
            <p className="mt-3 rounded-md bg-danger/10 px-3 py-2 text-sm text-danger">
              <strong>⚠ この操作は取り消せません。</strong>削除されたデータは復元できません。
            </p>
          </div>
          <div className="flex justify-end gap-2 border-t border-border px-4 py-3">
            <Button
              variant="secondary"
              onClick={onCancel}
            >
              キャンセル
            </Button>
            <Button
              variant="outline-danger"
              size="sm"
              onClick={onConfirm}
              disabled={loading}
              loading={loading}
            >
              {confirmLabel}
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
};
