import { Alert } from '@/components/atoms/Alert';
import {
  DataActionRail,
  type DataActionRailProps,
} from '@/components/organisms/DataActionRail/DataActionRail';
import { SearchCard } from '@/components/organisms/SearchCard/SearchCard';
import type { SearchCategories } from '@/types/common';

export interface AssetBalanceUtilityRailProps {
  actionRailProps: DataActionRailProps;
  error: string | null;
  searchCardProps: {
    visible: boolean;
    categories: SearchCategories;
    value: string;
    onSearch: (query: string) => void;
  };
}

export function AssetBalanceUtilityRail({
  actionRailProps,
  error,
  searchCardProps,
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
    </>
  );
}
