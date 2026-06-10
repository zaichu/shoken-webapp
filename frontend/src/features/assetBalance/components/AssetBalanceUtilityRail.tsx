import { Alert } from '@/components/atoms/Alert';
import {
  DataActionRail,
  type DataActionRailProps,
} from '@/components/organisms/DataActionRail/DataActionRail';
import { SearchCard } from '@/components/organisms/SearchCard/SearchCard';
import type { AssetBalanceData } from '@/types/api';
import type { SearchCategories } from '@/types/common';
import { AssetReviewPromptCard } from './AssetReviewPromptCard';

export interface AssetBalanceUtilityRailProps {
  actionRailProps: DataActionRailProps;
  error: string | null;
  searchCardProps: {
    visible: boolean;
    categories: SearchCategories;
    value: string;
    onSearch: (query: string) => void;
  };
  reviewPromptCardProps: {
    assetBalanceData: AssetBalanceData[];
    dividendPerShareMap: Map<string, number>;
  };
}

export function AssetBalanceUtilityRail({
  actionRailProps,
  error,
  searchCardProps,
  reviewPromptCardProps,
}: AssetBalanceUtilityRailProps) {
  return (
    <>
      <DataActionRail {...actionRailProps} />

      {error && (
        <div className="px-5 py-4">
          <Alert variant="danger" role="alert" aria-live="assertive">
            <strong>エラー:</strong> {error}
          </Alert>
        </div>
      )}

      {searchCardProps.visible && (
        <SearchCard
          onSearch={searchCardProps.onSearch}
          categories={searchCardProps.categories}
          value={searchCardProps.value}
          compact
        />
      )}

      <AssetReviewPromptCard {...reviewPromptCardProps} />
    </>
  );
}
