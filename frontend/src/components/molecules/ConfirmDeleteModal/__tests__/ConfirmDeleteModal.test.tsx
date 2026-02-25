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

  it('Tab キーでフォーカスがモーダル内を循環する', async () => {
    const user = userEvent.setup();
    render(<ConfirmDeleteModal {...defaultProps} />);
    // autoFocus により最初のフォーカスはキャンセルボタン
    const cancelBtn = screen.getByRole('button', { name: 'キャンセル' });
    expect(document.activeElement).toBe(cancelBtn);

    // Tab でフォーカスが次のボタンへ移動
    await user.tab();
    expect(document.activeElement).not.toBe(cancelBtn);
    // モーダル内の要素であること
    expect(screen.getByRole('dialog')).toContainElement(document.activeElement as HTMLElement);
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
