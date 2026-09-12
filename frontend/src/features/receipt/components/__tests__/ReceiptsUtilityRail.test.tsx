import { fireEvent, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { ReceiptsUtilityRail } from '../ReceiptsUtilityRail';

vi.mock('@/components/organisms/DataActionRail/DataActionRail', () => ({
  DataActionRail: ({ saveLabel }: { saveLabel: string }) => (
    <div data-testid="data-action-rail">{saveLabel}</div>
  ),
}));

const actionRailProps = {
  onFileSelect: vi.fn(),
  selectedFileName: 'receipts.csv',
  fileInputDisabled: false,
  hasCsvFile: true,
  saveLabel: '3件 追加で保存',
  onSave: vi.fn(),
  saveDisabled: false,
  hasDbData: true,
  deleteLabel: '全件削除 (3件)',
  onDeleteRequest: vi.fn(),
  deleteDisabled: false,
  saveResult: null,
  saveModeLabel: '追加',
};

const alertsProps = {
  dbError: null,
  dbWarning: null,
  hasCsvFile: false,
  previewing: false,
  csvPreview: null,
};

describe('ReceiptsUtilityRail', () => {
  it('DataActionRail が描画される', () => {
    render(
      <ReceiptsUtilityRail
        actionRailProps={actionRailProps}
        alertsProps={alertsProps}
        authLoading={false}
        dbLoading={false}
      />
    );

    expect(screen.getByTestId('data-action-rail')).toHaveTextContent('3件 追加で保存');
  });

  it('authLoading=true のとき Spinner と認証確認メッセージが表示される', () => {
    render(
      <ReceiptsUtilityRail
        actionRailProps={actionRailProps}
        alertsProps={alertsProps}
        authLoading
        dbLoading={false}
      />
    );

    expect(screen.getByLabelText('読み込み中...')).toBeInTheDocument();
    expect(screen.getByText('認証状態を確認しています...')).toBeInTheDocument();
  });

  it('dbLoading=true のときデータ読み込みメッセージが表示される', () => {
    render(
      <ReceiptsUtilityRail
        actionRailProps={actionRailProps}
        alertsProps={alertsProps}
        authLoading={false}
        dbLoading
      />
    );

    expect(screen.getByText('データを読み込んでいます...')).toBeInTheDocument();
    expect(screen.getByLabelText('読み込み中...')).toBeInTheDocument();
  });

  it('authLoading=false かつ dbLoading=false のとき Spinner は表示されない', () => {
    render(
      <ReceiptsUtilityRail
        actionRailProps={actionRailProps}
        alertsProps={alertsProps}
        authLoading={false}
        dbLoading={false}
      />
    );

    expect(screen.queryByLabelText('読み込み中...')).not.toBeInTheDocument();
    expect(screen.queryByText('認証状態を確認しています...')).not.toBeInTheDocument();
    expect(screen.queryByText('データを読み込んでいます...')).not.toBeInTheDocument();
  });

  describe('スマホ幅のCSV折り畳み (Issue #837)', () => {
    it('折り畳み入口を表示し、初期は折りたたまれている', () => {
      render(
        <ReceiptsUtilityRail
          actionRailProps={actionRailProps}
          alertsProps={alertsProps}
          authLoading={false}
          dbLoading={false}
        />
      );

      const toggle = screen.getByTestId('receipt-csv-toggle');
      expect(toggle).toHaveAttribute('aria-expanded', 'false');
      expect(toggle).toHaveAccessibleName('CSV取り込み・削除 開く');
      // 展開部は max-sm:hidden で畳まれる (PC幅では表示される)
      const region = screen.getByRole('region', { name: 'CSV取り込み・削除' });
      expect(region).toHaveClass('max-sm:hidden');
      // aria-controls が展開部の id を指す
      expect(toggle.getAttribute('aria-controls')).toBe(region.getAttribute('id'));
      // シェブロンは装飾として支援技術から隠す
      expect(toggle.querySelector('svg[aria-hidden="true"]')).toBeInTheDocument();
      // DataActionRail 自体はマウントされたまま (状態の二重化なし)
      expect(screen.getByTestId('data-action-rail')).toBeInTheDocument();
    });

    it('クリックで展開・折り畳みできる', () => {
      render(
        <ReceiptsUtilityRail
          actionRailProps={actionRailProps}
          alertsProps={alertsProps}
          authLoading={false}
          dbLoading={false}
        />
      );

      const toggle = screen.getByTestId('receipt-csv-toggle');
      fireEvent.click(toggle);

      expect(toggle).toHaveAttribute('aria-expanded', 'true');
      expect(toggle).toHaveAccessibleName('CSV取り込み・削除 閉じる');
      const region = screen.getByRole('region', { name: 'CSV取り込み・削除' });
      expect(region).not.toHaveClass('max-sm:hidden');
      expect(screen.getByTestId('data-action-rail')).toHaveTextContent('3件 追加で保存');

      fireEvent.click(toggle);
      expect(toggle).toHaveAttribute('aria-expanded', 'false');
      expect(screen.getByRole('region', { name: 'CSV取り込み・削除' })).toHaveClass(
        'max-sm:hidden'
      );
    });

    it('キーボード（Tab / Enter / Space）で展開・折り畳みできる', async () => {
      const user = userEvent.setup();
      render(
        <ReceiptsUtilityRail
          actionRailProps={actionRailProps}
          alertsProps={alertsProps}
          authLoading={false}
          dbLoading={false}
        />
      );

      const toggle = screen.getByTestId('receipt-csv-toggle');
      await user.tab();
      expect(toggle).toHaveFocus();

      await user.keyboard('{Enter}');
      expect(toggle).toHaveAttribute('aria-expanded', 'true');
      expect(
        screen.getByRole('region', { name: 'CSV取り込み・削除' })
      ).not.toHaveClass('max-sm:hidden');

      await user.keyboard(' ');
      expect(toggle).toHaveAttribute('aria-expanded', 'false');
    });
  });
});
