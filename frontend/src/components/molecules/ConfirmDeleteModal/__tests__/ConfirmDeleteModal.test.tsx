import { render, screen, fireEvent } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { vi } from 'vitest';
import { ConfirmDeleteModal } from '../ConfirmDeleteModal';

const defaultProps = {
  isOpen: true,
  onConfirm: vi.fn(),
  onCancel: vi.fn(),
  title: '削除確認',
  description: 'データを削除します',
  itemCount: 3,
};

beforeEach(() => {
  vi.clearAllMocks();
});

describe('ConfirmDeleteModal', () => {
  it('isOpen=false のとき何も描画しない', () => {
    render(<ConfirmDeleteModal {...defaultProps} isOpen={false} />);
    expect(screen.queryByRole('dialog')).toBeNull();
  });

  it('isOpen=true のとき描画される', () => {
    render(<ConfirmDeleteModal {...defaultProps} />);
    expect(screen.getByRole('dialog')).toBeInTheDocument();
  });

  it('Escape キーでモーダルが閉じる（autoFocus のキャンセルボタンにフォーカス中）', () => {
    render(<ConfirmDeleteModal {...defaultProps} />);
    const cancelBtn = screen.getByRole('button', { name: 'キャンセル' });
    cancelBtn.focus();
    fireEvent.keyDown(cancelBtn, { key: 'Escape' });
    expect(defaultProps.onCancel).toHaveBeenCalledTimes(1);
  });

  it('Escape キーでモーダルが閉じる（削除ボタンにフォーカス中）', () => {
    render(<ConfirmDeleteModal {...defaultProps} />);
    const confirmBtn = screen.getByRole('button', { name: '削除する' });
    confirmBtn.focus();
    fireEvent.keyDown(confirmBtn, { key: 'Escape' });
    expect(defaultProps.onCancel).toHaveBeenCalledTimes(1);
  });

  it('Tab キーでフォーカスがモーダル内を循環する（末尾→先頭のラップ）', async () => {
    const user = userEvent.setup();
    render(<ConfirmDeleteModal {...defaultProps} />);
    // 末尾の削除ボタンにフォーカスを移動
    const confirmBtn = screen.getByRole('button', { name: '削除する' });
    confirmBtn.focus();
    expect(document.activeElement).toBe(confirmBtn);

    // Tab で先頭要素（✕閉じるボタン）へラップ
    await user.tab();
    const closeBtn = screen.getByRole('button', { name: '閉じる' });
    expect(document.activeElement).toBe(closeBtn);
  });

  it('Shift+Tab キーで逆循環する（先頭→末尾のラップ）', async () => {
    const user = userEvent.setup();
    render(<ConfirmDeleteModal {...defaultProps} />);
    // 先頭の✕閉じるボタンにフォーカス
    const closeBtn = screen.getByRole('button', { name: '閉じる' });
    closeBtn.focus();
    expect(document.activeElement).toBe(closeBtn);

    // Shift+Tab で末尾要素（削除ボタン）へラップ
    await user.tab({ shift: true });
    const confirmBtn = screen.getByRole('button', { name: '削除する' });
    expect(document.activeElement).toBe(confirmBtn);
  });

  it('dialog 自体にフォーカスがある状態で Tab を押すと先頭要素へ移動する', async () => {
    const user = userEvent.setup();
    render(<ConfirmDeleteModal {...defaultProps} />);
    // dialog 要素（tabIndex=-1）に直接フォーカス（トラップ対象外）
    const dialog = screen.getByRole('dialog');
    dialog.focus();
    expect(document.activeElement).toBe(dialog);

    await user.tab();
    // Tab → 先頭要素（✕閉じるボタン）へ
    expect(document.activeElement).toBe(screen.getByRole('button', { name: '閉じる' }));
  });

  it('dialog 自体にフォーカスがある状態で Shift+Tab を押すと末尾要素へ移動する', async () => {
    const user = userEvent.setup();
    render(<ConfirmDeleteModal {...defaultProps} />);
    const dialog = screen.getByRole('dialog');
    dialog.focus();
    expect(document.activeElement).toBe(dialog);

    await user.tab({ shift: true });
    // Shift+Tab → 末尾要素（削除ボタン）へ
    expect(document.activeElement).toBe(screen.getByRole('button', { name: '削除する' }));
  });

  it('キャンセルボタンクリックで onCancel が呼ばれる', () => {
    render(<ConfirmDeleteModal {...defaultProps} />);
    fireEvent.click(screen.getByRole('button', { name: 'キャンセル' }));
    expect(defaultProps.onCancel).toHaveBeenCalledTimes(1);
  });

  it('削除ボタンクリックで onConfirm が呼ばれる', () => {
    render(<ConfirmDeleteModal {...defaultProps} />);
    fireEvent.click(screen.getByRole('button', { name: '削除する' }));
    expect(defaultProps.onConfirm).toHaveBeenCalledTimes(1);
  });

  it('loading=true のとき削除ボタンが disabled になる', () => {
    render(<ConfirmDeleteModal {...defaultProps} loading={true} />);
    // loading=true 時は Button が「読み込み中...」テキストに切り替わる
    expect(screen.getByRole('button', { name: /読み込み中/ })).toBeDisabled();
  });

  it('confirmLabel を変更できる', () => {
    render(<ConfirmDeleteModal {...defaultProps} confirmLabel="実行する" />);
    expect(screen.getByRole('button', { name: '実行する' })).toBeInTheDocument();
  });
});
