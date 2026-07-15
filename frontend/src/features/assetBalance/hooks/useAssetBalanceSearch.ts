import { useState } from 'react';
import { createSearchOptions } from '@/lib/utils/dataTransformer';
import { filterByConfig, type FilterConfig } from '@/lib/utils/searchUtils';
import type { AssetBalanceData } from '@/types/api';
import type { SearchCategories } from '@/types/common';
import type { AssetBalanceSearchFacets } from './useAssetBalanceDataSource';

const filterConfig: FilterConfig<AssetBalanceData> = {
  partialStringFields: [
    (item) => item.security_code,
    (item) => item.security_name,
  ],
};

export interface UseAssetBalanceSearchResult {
  searchQuery: string;
  handleSearch: (query: string) => void;
  clearSearch: () => void;
  resetSearch: () => void;
  filteredData: AssetBalanceData[];
  searchCategories: SearchCategories;
}

/**
 * 保有銘柄の検索クエリと検索候補を管理するフック
 * Receipt 側（useReceiptBaseData）と同様、CSV取込/削除確認の状態とは分離した
 * コンポーネントローカルな state として持つ
 */
export function useAssetBalanceSearch(
  assetBalanceData: AssetBalanceData[],
  facets: AssetBalanceSearchFacets | undefined,
  hasCsvFile: boolean
): UseAssetBalanceSearchResult {
  const [searchQuery, setSearchQuery] = useState('');

  const handleSearch = (query: string) => {
    setSearchQuery(query);
  };

  const clearSearch = () => {
    setSearchQuery('');
  };

  const resetSearch = () => {
    setSearchQuery('');
  };

  const filteredData = filterByConfig(assetBalanceData, searchQuery, filterConfig);

  // facets は保存済み DB データ（一覧 API）由来のため、CSV プレビュー中は使わない
  const searchCategories: SearchCategories = !hasCsvFile && facets?.securities
    ? {
      securities: facets.securities.map((option) => ({ value: option.value, label: option.label })),
    }
    : {
      securities: createSearchOptions(assetBalanceData, 'security_code', 'security_name', true),
    };

  return {
    searchQuery,
    handleSearch,
    clearSearch,
    resetSearch,
    filteredData,
    searchCategories,
  };
}
