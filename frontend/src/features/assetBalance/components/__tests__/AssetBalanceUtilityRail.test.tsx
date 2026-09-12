import { fireEvent, render, screen } from '@testing-library/react';
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

vi.mock('../AssetReviewPromptCard', () => ({
  AssetReviewPromptCard: () => <div data-testid="asset-review-prompt-card" />,
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

const reviewPromptCardProps = {
  assetBalanceData: [],
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
        reviewPromptCardProps={reviewPromptCardProps}
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
        reviewPromptCardProps={reviewPromptCardProps}
      />
    );

    expect(screen.queryByTestId('search-card')).not.toBeInTheDocument();
  });

  describe('スマホ幅のCSV折り畳み (Issue #859 追加対応、取引明細と同方式)', () => {
    const renderRail = () =>
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
          reviewPromptCardProps={reviewPromptCardProps}
        />
      );

    it('折り畳み入口を表示し、初期は折りたたまれている', () => {
      renderRail();

      const toggle = screen.getByTestId('assetbalance-csv-toggle');
      expect(toggle).toHaveAttribute('aria-expanded', 'false');
      expect(toggle).toHaveAccessibleName('CSV取り込み・削除 開く');
      // aria-controls が展開部の id を指す
      const region = screen.getByRole('region', { name: 'CSV取り込み・削除' });
      expect(toggle.getAttribute('aria-controls')).toBe(region.getAttribute('id'));
      // シェブロンは装飾として支援技術から隠す
      expect(toggle.querySelector('svg[aria-hidden="true"]')).toBeInTheDocument();
      // DataActionRail 自体はマウントされたまま (状態の二重化なし)
      expect(screen.getByTestId('data-action-rail')).toBeInTheDocument();
    });

    it('クリックで展開・折り畳みできる', () => {
      renderRail();

      const toggle = screen.getByTestId('assetbalance-csv-toggle');
      fireEvent.click(toggle);

      expect(toggle).toHaveAttribute('aria-expanded', 'true');
      expect(toggle).toHaveAccessibleName('CSV取り込み・削除 閉じる');
      expect(screen.getByTestId('data-action-rail')).toHaveTextContent('2件 全件置換で保存');

      fireEvent.click(toggle);
      expect(toggle).toHaveAttribute('aria-expanded', 'false');
      expect(toggle).toHaveAccessibleName('CSV取り込み・削除 開く');
    });
  });
});
