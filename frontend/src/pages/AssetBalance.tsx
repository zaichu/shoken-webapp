import React, { useEffect, useState, useMemo } from 'react';
import { Layout } from '../components/templates/Layout';
import { CSVFileInput } from '../components/molecules/CSVFileInput';
import { useAuth } from '@/features/auth/hooks/useAuth';
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
import { renderSecurityCode } from '@/components/atoms/SecurityCodeLink';
import { assetBalanceApi } from '@/features/receipt/api/receiptApi';
import { useReceiptDataSource } from '@/hooks/common/useReceiptDataSource';


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
 * 保有銘柄データを表示するコンポーネント
 */
export const AssetBalanceInfo: React.FC<AssetBalanceProps> = ({ assetBalanceData }) => {
  const [searchQuery, setSearchQuery] = useState('');

  // 検索オプションの生成
  const searchCategories = useMemo(() => ({
    securities: createSearchOptions(assetBalanceData, 'security_code', 'security_name', true)
  }), [assetBalanceData]);

  // 検索クエリに基づくフィルタリング
  const filteredData = useMemo(() => filterDataBySearchQuery(
    assetBalanceData,
    searchQuery,
    ['security_code', 'security_name']
  ), [assetBalanceData, searchQuery]);

  // テーブルカラムの定義
  const columns: TableColumnConfig[] = [
    {
      key: 'security_code',
      header: '銘柄コード',
      width: '90px',
      textAlign: 'center',
      format: renderSecurityCode
    },
    { key: 'security_name', header: '銘柄名', width: '200px' },
    { key: 'shares', header: '保有数量', width: '80px', textAlign: 'right', format: formatNumber },
    { key: 'average_purchase_price', header: '平均取得価額', width: '100px', textAlign: 'right', format: formatCurrency },
    { key: 'total_purchase_amount', header: '取得総額', width: '100px', textAlign: 'right', format: formatCurrency },
  ];

  return (
    <ReceiptTemplate
      title="保有銘柄"
      onSearch={(query: string) => setSearchQuery(query)}
      searchCategories={searchCategories}
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

/**
 * 保有銘柄管理ページコンポーネント
 */
export function AssetBalancePage() {
  const { isAuthenticated, isLoading: authLoading, login } = useAuth();

  // 共通フック
  const {
    dbData,
    csvData,
    loading,
    error,
    saving,
    deleting,
    csvReader,
    hasCsvData,
    hasDbData,
    handleFileSelect,
    handleSaveToDB,
    handleDeleteAll,
  } = useReceiptDataSource<AssetBalanceData>({
    api: assetBalanceApi,
    parseCsvItem,
    csvReaderOptions: { skipHeaderRows: 6 },
    deleteConfirmMessage: '保存された保有銘柄データを削除しますか？',
  });

  // CSVデータの変換（空の銘柄コードをフィルタ）
  const tmpAssetBalanceData = useReceiptData(csvData, parseCsvItem, sortBySecurityCode);
  const [assetBalanceData, setAssetBalanceData] = useState<AssetBalanceData[]>([]);

  // CSVデータが読み込まれたらフィルタして設定
  useEffect(() => {
    if (csvData.length > 0) {
      setAssetBalanceData(tmpAssetBalanceData.filter(item => item.security_code !== ''));
    } else if (dbData.length > 0) {
      setAssetBalanceData(dbData);
    } else {
      setAssetBalanceData([]);
    }
  }, [csvData, tmpAssetBalanceData, dbData]);

  const isProcessing = loading || saving || deleting || csvReader.isLoading || authLoading;

  return (
    <Layout>
      <div className="asset-balance-page mt-2" aria-busy={isProcessing}>
        {/* 認証確認中 */}
        {authLoading && (
          <div className="text-center my-4" role="status" aria-live="polite">
            <div className="spinner-border text-primary" aria-hidden="true" />
            <p className="mt-2 text-muted">認証状態を確認しています...</p>
          </div>
        )}

        {/* 未ログイン時のログイン誘導 */}
        {!authLoading && !isAuthenticated && (
          <div className="alert alert-info my-3" role="status" aria-live="polite">
            <p className="mb-2">保有銘柄データを管理するにはログインが必要です。</p>
            <button
              className="btn btn-primary btn-sm"
              onClick={() => login()}
              aria-label="Googleアカウントでログイン"
            >
              ログイン
            </button>
          </div>
        )}

        {/* ログイン済みの場合のメインコンテンツ */}
        {!authLoading && isAuthenticated && (
          <>
            <div className="d-flex align-items-center gap-2 flex-wrap">
              <div className="page-control-panel">
                <CSVFileInput
                  onFileSelect={handleFileSelect}
                  selectedFileName={csvReader.fileName || ''}
                  disabled={loading || saving || deleting}
                />
              </div>
              <div className="btn-group" role="group" aria-label="データ操作">
                {hasCsvData && (
                  <button
                    className="btn btn-primary btn-sm"
                    onClick={handleSaveToDB}
                    disabled={saving || deleting}
                    aria-disabled={saving || deleting}
                  >
                    {saving ? '保存中...' : '保存'}
                  </button>
                )}
                {hasDbData && (
                  <button
                    className="btn btn-outline-danger btn-sm"
                    onClick={handleDeleteAll}
                    disabled={saving || deleting}
                    aria-disabled={saving || deleting}
                  >
                    {deleting ? '削除中...' : '削除'}
                  </button>
                )}
              </div>
            </div>

            {(csvReader.error || error) && (
              <div className="alert alert-danger my-3" role="alert" aria-live="assertive">
                <strong>エラー:</strong> {csvReader.error || error}
              </div>
            )}

            <div aria-live="polite" aria-atomic="true">
              {(loading || saving || deleting || csvReader.isLoading) && (
                <div className="text-center my-4" role="status">
                  <div className="spinner-border text-primary" aria-hidden="true" />
                  <p className="mt-2 text-muted">
                    {loading && 'データを読み込んでいます...'}
                    {saving && 'データを保存しています...'}
                    {deleting && 'データを削除しています...'}
                    {csvReader.isLoading && 'CSVファイルを処理しています...'}
                  </p>
                </div>
              )}
            </div>

            {assetBalanceData.length > 0 && <AssetBalanceInfo assetBalanceData={assetBalanceData} />}
          </>
        )}
      </div>
    </Layout>
  );
}
