import { useCallback, useEffect, useState } from 'react';
import { Layout } from '../components/templates/Layout';
import { CSVFileInput } from '../components/molecules/CSVFileInput';
import { useCSVReader } from '../hooks/useCSVReader';
import { Holdings } from './Receipt/Holdings';
import { useHoldingsStorage } from '@/hooks/common/useHoldingsStorage';
import { HoldingsData } from '@/lib/interfaces/holdings';
import { parseNumber } from '@/lib/utils/formatters';
import { useReceiptData } from '@/hooks/receipt/useReceiptData';

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

/**
 * 保有株管理ページコンポーネント
 */
export function HoldingsPage() {
  const [holdingsCsvData, setHoldingsCsvData] = useState<Record<string, unknown>[]>([]);
  const { holdingsStorageData, saveHoldings, clearHoldings, lastUpdated } = useHoldingsStorage();
  const [holdingsData, setHoldingsData] = useState<HoldingsData[]>(holdingsStorageData);
  const tmpHoldingsData = useReceiptData(holdingsCsvData, parseCsvItem, sortBySecurityCode);
  const holdingsCSV = useCSVReader();

  const handleFileSelect = async (file: File) => {
    try {
      setHoldingsCsvData(await holdingsCSV.parseCSV(file));
      if (holdingsCSV.error) holdingsCSV.resetError();
    } catch (e) {
      console.error('CSV処理エラー:', e);
    }
  };

  useEffect(() => {
    if (holdingsCsvData.length > 0) {
      setHoldingsData(tmpHoldingsData.filter(item => item.security_code !== ''));
    }
  }, [holdingsCsvData]);

  const handleSaveToStorage = useCallback(() => {
    if (holdingsData.length > 0) {
      saveHoldings(holdingsData);
      // alert('保有株データをローカルストレージに保存しました');
    }
  }, [holdingsData, saveHoldings]);

  const handleClearStorage = useCallback(() => {
    if (window.confirm('保存された保有株データを削除しますか？')) {
      clearHoldings();
      setHoldingsData([]);
      setHoldingsCsvData([]);
      holdingsCSV.reset();
      // alert('保有株データを削除しました');
    }
  }, [clearHoldings]);

  return (
    <Layout>
      <div className="holdings-page">
        <div className="row">
          <div className='col'>
            <CSVFileInput onFileSelect={handleFileSelect} selectedFileName={holdingsCSV.fileName || ''} />
          </div>
        </div>
        {holdingsCSV.error && (
          <div className="alert alert-danger my-3" role="alert">
            <strong>エラー:</strong> {holdingsCSV.error}
          </div>
        )}

        {holdingsCSV.isLoading && (
          <div className="text-center my-4">
            <div className="spinner-border text-primary" role="status">
              <span className="visually-hidden">Loading...</span>
            </div>
          </div>
        )}

        <div className="mt-2 d-flex gap-2">
          <button className="btn btn-primary" onClick={handleSaveToStorage} disabled={holdingsData.length === 0}>
            保存
          </button>
          <button className="btn btn-outline-danger" onClick={handleClearStorage} disabled={holdingsStorageData.length === 0}>
            保存データを削除
          </button>
          {lastUpdated && (
            <span className="align-self-center text-muted ms-3">
              最終更新: {lastUpdated}
            </span>
          )}
        </div>

        {holdingsData.length > 0 && <Holdings holdingsData={holdingsData} />}
      </div>
    </Layout>
  );
}
