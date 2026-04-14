import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { AssetBalanceUtilityRail } from '../AssetBalanceUtilityRail';

vi.mock('@/components/organisms/DataActionRail/DataActionRail', () => ({
  DataActionRail: ({ saveLabel }: { saveLabel: string }) => (
    <div data-testid="data-action-rail">{saveLabel}</div>
  ),
}));

vi.mock('@/components/organisms/SearchCard/SearchCard', () => ({
  SearchCard: ({ value }: { value: string }) => <div data-testid="search-card">{value}</div>,
}));

const actionRailProps = {
  onFileSelect: vi.fn(),
  selectedFileName: 'assetbalance.csv',
  fileInputDisabled: false,
  hasCsvFile: true,
  saveLabel: '2件 全件置換で保存',
  onSave: vi.fn(),
  saveDisabled: false,
  hasDbData: true,
  deleteLabel: '全件削除 (2件)',
  onDeleteRequest: vi.fn(),
  deleteDisabled: false,
  saveResult: null,
  saveModeLabel: '全件置換',
};

describe('AssetBalanceUtilityRail', () => {
  it('データ操作、エラー、検索カードを表示する', () => {
    render(
      <AssetBalanceUtilityRail
        actionRailProps={actionRailProps}
        error="取得失敗"
        searchCardProps={{
          visible: true,
          categories: {
            securities: [{ value: '7203', label: '7203: トヨタ自動車' }],
          },
          value: '7203',
          onSearch: vi.fn(),
        }}
      />
    );

    expect(screen.getByTestId('data-action-rail')).toHaveTextContent('2件 全件置換で保存');
    expect(screen.getByRole('alert')).toHaveTextContent('取得失敗');
    expect(screen.getByTestId('search-card')).toHaveTextContent('7203');
  });

  it('検索対象がないときは SearchCard を表示しない', () => {
    render(
      <AssetBalanceUtilityRail
        actionRailProps={actionRailProps}
        error={null}
        searchCardProps={{
          visible: false,
          categories: {},
          value: '',
          onSearch: vi.fn(),
        }}
      />
    );

    expect(screen.queryByTestId('search-card')).not.toBeInTheDocument();
  });
});
