import { render, screen } from '@testing-library/react';
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
});
