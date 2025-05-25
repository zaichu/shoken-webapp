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
  formatNumber,
  parseNumber
} from '@/lib/utils/formatters';
import { useReceiptData } from '@/hooks/receipt/useReceiptData';
import { useHoldingsStorage } from '@/hooks/common/useHoldingsStorage';

// CSVアイテムをHoldingsDataに変換
const parseCsvItem = (item: Record<string, unknown>): HoldingsData => ({
  security_code: String(item['銘柄コード'] || '').replace(/"/g, ''),
  security_name: String(item['銘柄名'] || ''),
  shares: parseNumber(item['保有数量［株］']),
  executing_shares: parseNumber(item['執行中［株］']),
  average_purchase_price: parseNumber(item['平均取得価額［円］']),
  total_purchase_amount: parseNumber(item['取得総額［円］']),
  current_price: parseNumber(item['現在値［円］']),
  daily_change: parseNumber(item['現在値（前日比）［円］']),
  market_value: parseNumber(item['時価評価額［円］']),
  profit_loss_rate: parseNumber(item['評価損益［％］']),
});

// 銘柄コードでソート
const sortBySecurityCode = (data: HoldingsData[]): HoldingsData[] => {
  return [...data].sort((a, b) => a.security_code.localeCompare(b.security_code));
};

interface HoldingsProps {
  csvData: Record<string, unknown>[];
}

/**
 * 保有株データを表示するコンポーネント
 */
export const Holdings: React.FC<HoldingsProps> = ({ csvData }) => {
  const [searchQuery, setSearchQuery] = useState('');
  const onSearch = useCallback((query: string) => setSearchQuery(query), []);

  const { saveHoldings, clearHoldings, lastUpdated } = useHoldingsStorage();

  // CSVデータを保有株データ形式に変換
  const holdingsData = useReceiptData(csvData, parseCsvItem, sortBySecurityCode);

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

  // ローカルストレージに保存
  const handleSaveToStorage = useCallback(() => {
    if (holdingsData.length > 0) {
      saveHoldings(holdingsData);
      alert('保有株データをローカルストレージに保存しました');
    }
  }, [holdingsData, saveHoldings]);

  // ローカルストレージをクリア
  const handleClearStorage = useCallback(() => {
    if (window.confirm('保存された保有株データを削除しますか？')) {
      clearHoldings();
      alert('保有株データを削除しました');
    }
  }, [clearHoldings]);

  // テーブルカラムの定義
  const columns = useMemo<TableColumnConfig[]>(() => [
    { key: 'security_code', header: '銘柄コード', width: '100px' },
    { key: 'security_name', header: '銘柄名', width: '250px' },
    { key: 'shares', header: '保有数量', width: '100px', textAlign: 'right', format: formatNumber },
    { key: 'average_purchase_price', header: '平均取得価額', width: '120px', textAlign: 'right', format: formatCurrency },
    { key: 'total_purchase_amount', header: '取得総額', width: '120px', textAlign: 'right', format: formatCurrency },
    { key: 'current_price', header: '現在値', width: '100px', textAlign: 'right', format: formatCurrency },
    { key: 'daily_change', header: '前日比', width: '80px', textAlign: 'right', format: formatCurrency },
    { key: 'market_value', header: '時価評価額', width: '120px', textAlign: 'right', format: formatCurrency },
    {
      key: 'profit_loss_rate',
      header: '評価損益率',
      width: '100px',
      textAlign: 'right',
      format: (value: unknown) => `${typeof value === 'number' ? value.toFixed(2) : '0.00'}%`
    },
  ], []);

  return (
    <ReceiptTemplate
      title="保有株一覧"
      searchQuery={searchQuery}
      onSearch={onSearch}
      searchOptions={searchOptions}
    >
      <div className="mb-3 d-flex gap-2">
        <button
          className="btn btn-primary"
          onClick={handleSaveToStorage}
          disabled={holdingsData.length === 0}
        >
          ローカルストレージに保存
        </button>
        <button
          className="btn btn-outline-danger"
          onClick={handleClearStorage}
        >
          保存データを削除
        </button>
        {lastUpdated && (
          <span className="align-self-center text-muted ms-3">
            最終更新: {new Date(lastUpdated).toLocaleString('ja-JP')}
          </span>
        )}
      </div>

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
