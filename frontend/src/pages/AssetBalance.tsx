import { useCallback, useEffect, useMemo, useState } from 'react';
import { Layout } from '../components/templates/Layout';
import { CSVFileInput } from '../components/molecules/CSVFileInput';
import { useCSVReader } from '../hooks/useCSVReader';
import { useAssetBalanceStorage } from '@/hooks/common/useAssetBalanceStorage';
import { AssetBalanceData } from '@/lib/interfaces/assetBalance';
import { parseNumber } from '@/lib/utils/formatters';
import { useReceiptData } from '@/hooks/receipt/useReceiptData';
import { ReceiptTemplate } from '@/components/templates/ReceiptTemplate';
import { ReceiptTable } from '@/components/organisms/ReceiptTable/ReceiptTable';
import { TableColumnConfig } from '@/lib/interfaces/receipt';
import {
  createSearchOptions,
  filterDataBySearchQuery,
} from '@/lib/utils/dataTransformer';
import {
  formatCurrency,
  formatNumber
} from '@/lib/utils/formatters';


// CSVアイテムをAssetBalanceDataに変換
const parseCsvItem = (item: Record<string, unknown>): AssetBalanceData => ({
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
const sortBySecurityCode = (data: AssetBalanceData[]): AssetBalanceData[] => {
  return [...data].sort((a, b) => a.security_code.localeCompare(b.security_code));
};


interface AssetBalanceProps {
  assetBalanceData: AssetBalanceData[];
}

/**
 * 保有株データを表示するコンポーネント
 */
export const AssetBalanceInfo: React.FC<AssetBalanceProps> = ({ assetBalanceData }) => {
  const [searchQuery, setSearchQuery] = useState('');
  const onSearch = useCallback((query: string) => setSearchQuery(query), []);

  // 検索オプションの生成
  const searchOptions = useMemo(() =>
    createSearchOptions(
      assetBalanceData,
      'security_code',
      'security_name',
      true
    ),
    [assetBalanceData]
  );

  // 検索クエリに基づくフィルタリング
  const filteredData = useMemo(() =>
    filterDataBySearchQuery(
      assetBalanceData,
      searchQuery,
      ['security_code', 'security_name']
    ),
    [assetBalanceData, searchQuery]
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

AssetBalanceInfo.displayName = 'AssetBalanceInfo';

/**
 * 保有株管理ページコンポーネント
 */
export function AssetBalancePage() {
  const [assetBalanceCsvData, setAssetBalanceCsvData] = useState<Record<string, unknown>[]>([]);
  const { assetBalanceStorageData, saveAssetBalance, clearAssetBalance, lastUpdated } = useAssetBalanceStorage();
  const [assetBalanceData, setAssetBalanceData] = useState<AssetBalanceData[]>(assetBalanceStorageData);
  const tmpAssetBalanceData = useReceiptData(assetBalanceCsvData, parseCsvItem, sortBySecurityCode);
  const options = {
    skipHeaderRows: 6,
  };
  const assetBalanceCSV = useCSVReader(options);

  const handleFileSelect = async (file: File) => {
    try {
      setAssetBalanceCsvData(await assetBalanceCSV.parseCSV(file));
      if (assetBalanceCSV.error) assetBalanceCSV.resetError();
    } catch (e) {
      console.error('CSV処理エラー:', e);
    }
  };

  useEffect(() => {
    if (assetBalanceCsvData.length > 0) {
      setAssetBalanceData(tmpAssetBalanceData.filter(item => item.security_code !== ''));
    }
  }, [tmpAssetBalanceData, assetBalanceCsvData]);

  const handleSaveToStorage = useCallback(() => {
    if (assetBalanceData.length > 0) {
      saveAssetBalance(assetBalanceData);
      // alert('保有株データをローカルストレージに保存しました');
    }
  }, [assetBalanceData, saveAssetBalance]);

  const handleClearStorage = useCallback(() => {
    if (window.confirm('保存された保有株データを削除しますか？')) {
      clearAssetBalance();
      setAssetBalanceData([]);
      setAssetBalanceCsvData([]);
      assetBalanceCSV.reset();
      // alert('保有株データを削除しました');
    }
  }, [assetBalanceCSV, clearAssetBalance]);

  return (
    <Layout>
      <div className="asset-balance-page">
        <div className="row">
          <div className='col'>
            <CSVFileInput onFileSelect={handleFileSelect} selectedFileName={assetBalanceCSV.fileName || ''} />
          </div>
        </div>
        {assetBalanceCSV.error && (
          <div className="alert alert-danger my-3" role="alert">
            <strong>エラー:</strong> {assetBalanceCSV.error}
          </div>
        )}

        {assetBalanceCSV.isLoading && (
          <div className="text-center my-4">
            <div className="spinner-border text-primary" role="status">
              <span className="visually-hidden">Loading...</span>
            </div>
          </div>
        )}

        <div className="mt-2 d-flex gap-2">
          <button className="btn btn-primary" onClick={handleSaveToStorage} disabled={assetBalanceData.length === 0}>
            保存
          </button>
          <button className="btn btn-outline-danger" onClick={handleClearStorage} disabled={assetBalanceStorageData.length === 0}>
            保存データを削除
          </button>
          {lastUpdated && (
            <span className="align-self-center text-muted ms-3">
              最終更新: {lastUpdated}
            </span>
          )}
        </div>

        {assetBalanceData.length > 0 && <AssetBalanceInfo assetBalanceData={assetBalanceData} />}
      </div>
    </Layout>
  );
}
