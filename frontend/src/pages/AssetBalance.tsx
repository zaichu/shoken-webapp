import React, { useEffect, useState, Suspense, lazy } from 'react';
import { Layout } from '../components/templates/Layout';
import { PageHeader } from '../components/atoms/PageHeader';
import { CSVFileInput } from '../components/molecules/CSVFileInput';
import { Alert } from '@/components/atoms/Alert';
import { Button } from '@/components/atoms/Button';
import { Spinner } from '@/components/atoms/Spinner';
import { useAuth } from '@/features/auth/hooks/useAuth';
import { AssetBalanceData } from '@/lib/interfaces/assetBalance';
import { parseNumber } from '@/lib/utils/formatters';
import { useReceiptData } from '@/hooks/receipt/useReceiptData';
import { assetBalanceApi } from '@/features/receipt/api/receiptApi';
import { useReceiptDataSource } from '@/hooks/common/useReceiptDataSource';

// rechartsを含むコンポーネントを遅延読み込み（バンドルサイズ最適化）
const AssetPortfolioSummary = lazy(() =>
  import('@/components/organisms/AssetPortfolioSummary').then(module => ({
    default: module.AssetPortfolioSummary
  }))
);


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
 * 保有銘柄データを表示するコンポーネント（概要重視）
 */
export const AssetBalanceInfo: React.FC<AssetBalanceProps> = ({ assetBalanceData }) => {
  return (
    <Suspense fallback={<div className="h-64 flex items-center justify-center"><Spinner size="md" /></div>}>
      <AssetPortfolioSummary assetBalanceData={assetBalanceData} />
    </Suspense>
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
    filterCsvItem: (item) => item.security_code !== '',
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
      <PageHeader
        title="保有銘柄"
        description="保有している銘柄の一覧と評価額を確認できます。"
      />
      <div className="mt-2" aria-busy={isProcessing}>
        {/* 認証確認中 */}
        {authLoading && (
          <div className="status-message" role="status" aria-live="polite">
            <Spinner size="md" className="text-primary" />
            <p className="text-sm text-secondary">認証状態を確認しています...</p>
          </div>
        )}

        {/* 未ログイン時のログイン誘導 */}
        {!authLoading && !isAuthenticated && (
          <Alert variant="info" className="my-3" role="status" aria-live="polite">
            <p className="mb-2 text-sm">保有銘柄データを管理するにはログインが必要です。</p>
            <Button
              variant="primary"
              size="sm"
              onClick={() => login()}
              aria-label="Googleアカウントでログイン"
            >
              ログイン
            </Button>
          </Alert>
        )}

        {/* ログイン済みの場合のメインコンテンツ */}
        {!authLoading && isAuthenticated && (
          <>
            <div className="action-toolbar">
              <div className="form-input-container">
                <CSVFileInput
                  onFileSelect={handleFileSelect}
                  selectedFileName={csvReader.fileName || ''}
                  disabled={loading || saving || deleting}
                />
              </div>
              <div className="action-button-group" role="group" aria-label="データ操作">
                {hasCsvData && (
                  <Button
                    variant="primary"
                    size="sm"
                    onClick={handleSaveToDB}
                    disabled={saving || deleting}
                    aria-disabled={saving || deleting}
                  >
                    {saving ? '保存中...' : '保存'}
                  </Button>
                )}
                {hasDbData && (
                  <Button
                    variant="outline-danger"
                    size="sm"
                    onClick={handleDeleteAll}
                    disabled={saving || deleting || loading}
                    aria-disabled={saving || deleting || loading}
                  >
                    {deleting ? '削除中...' : `全件削除 (${dbData.length}件)`}
                  </Button>
                )}
              </div>
            </div>

            {(csvReader.error || error) && (
              <Alert variant="danger" className="my-3" role="alert" aria-live="assertive">
                <strong>エラー:</strong> {csvReader.error || error}
              </Alert>
            )}

            <div aria-live="polite" aria-atomic="true">
              {(loading || saving || deleting || csvReader.isLoading) && (
                <div className="status-message" role="status">
                  <Spinner size="md" className="text-primary" />
                  <p className="text-sm text-secondary">
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
