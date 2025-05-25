import { ReceiptTemplate } from '@/components/templates/ReceiptTemplate';
import { ReceiptTable } from '@/components/organisms/ReceiptTable/ReceiptTable';
import React, { useMemo, useState, useCallback } from 'react';
import { HoldingsData } from '@/lib/interfaces/holdings';
import { TableColumnConfig } from '@/lib/interfaces/receipt';
import {
  createSearchOptions,
  filterDataBySearchQuery,
} from '@/lib/utils/dataTransformer';
import {
  formatCurrency,
  formatNumber
} from '@/lib/utils/formatters';

interface HoldingsProps {
  holdingsData: HoldingsData[];
}

/**
 * 保有株データを表示するコンポーネント
 */
export const Holdings: React.FC<HoldingsProps> = ({ holdingsData }) => {
  const [searchQuery, setSearchQuery] = useState('');
  const onSearch = useCallback((query: string) => setSearchQuery(query), []);

  // 検索オプションの生成
  const searchOptions = useMemo(() =>
    createSearchOptions(
      holdingsData,
      'security_code',
      'security_name',
      true
    ),
    [holdingsData]
  );

  // 検索クエリに基づくフィルタリング
  const filteredData = useMemo(() =>
    filterDataBySearchQuery(
      holdingsData,
      searchQuery,
      ['security_code', 'security_name']
    ),
    [holdingsData, searchQuery]
  );

  // テーブルカラムの定義
  const columns = useMemo<TableColumnConfig[]>(() => [
    { key: 'security_code', header: '銘柄コード', width: '1px' },
    { key: 'security_name', header: '銘柄名', width: '150px' },
    { key: 'shares', header: '保有数量', width: '1px', textAlign: 'right', format: formatNumber },
    { key: 'average_purchase_price', header: '平均取得価額', width: '1px', textAlign: 'right', format: formatCurrency },
    { key: 'total_purchase_amount', header: '取得総額', width: '1px', textAlign: 'right', format: formatCurrency },
  ], []);

  return (
    <ReceiptTemplate
      title="保有株一覧"
      searchQuery={searchQuery}
      onSearch={onSearch}
      searchOptions={searchOptions}
    >
      <ReceiptTable
        data={filteredData}
        summary={[]}
        columns={columns}
        summaryColumns={[]}
        getGroupKey={() => ''}
      />
    </ReceiptTemplate>
  );
};
