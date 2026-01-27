import React, { useEffect, useState, useCallback, useRef } from 'react';
import { Layout } from '../components/templates/Layout';
import { CSVFileInput } from '../components/molecules/CSVFileInput';
import { useCSVReader } from '../hooks/useCSVReader';
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
import { assetBalanceApi } from '@/features/receipt/api/receiptApi';


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
  const searchCategories = {
    securities: createSearchOptions(assetBalanceData, 'security_code', 'security_name', true)
  };

  // 検索クエリに基づくフィルタリング
  const filteredData = filterDataBySearchQuery(
    assetBalanceData,
    searchQuery,
    ['security_code', 'security_name']
  );

  // テーブルカラムの定義
  const columns: TableColumnConfig[] = [
    { key: 'security_code', header: '銘柄コード', width: '90px' },
    { key: 'security_name', header: '銘柄名', width: '200px' },
    { key: 'shares', header: '保有数量', width: '80px', textAlign: 'right', format: formatNumber },
    // { key: 'executing_shares', header: '執行中', width: '60px', textAlign: 'right', format: formatNumber },
    { key: 'average_purchase_price', header: '平均取得価額', width: '100px', textAlign: 'right', format: formatCurrency },
    { key: 'total_purchase_amount', header: '取得総額', width: '100px', textAlign: 'right', format: formatCurrency },
    // { key: 'current_price', header: '現在値', width: '80px', textAlign: 'right', format: formatCurrency },
    // { key: 'daily_change', header: '前日比', width: '80px', textAlign: 'right', format: formatCurrency },
    // { key: 'market_value', header: '時価評価額', width: '100px', textAlign: 'right', format: formatCurrency },
    // { key: 'profit_loss_rate', header: '評価損益率', width: '90px', textAlign: 'right', format: (value: number) => `${formatNumber(value)}%` },
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
  const { isAuthenticated, isLoading: authLoading, login, onLogout } = useAuth();
  const [assetBalanceCsvData, setAssetBalanceCsvData] = useState<Record<string, unknown>[]>([]);
  const [assetBalanceData, setAssetBalanceData] = useState<AssetBalanceData[]>([]);
  const [savedData, setSavedData] = useState<AssetBalanceData[]>([]);
  const [dbLoading, setDbLoading] = useState(false);
  const [dbError, setDbError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [deleting, setDeleting] = useState(false);
  const tmpAssetBalanceData = useReceiptData(assetBalanceCsvData, parseCsvItem, sortBySecurityCode);
  const options = {
    skipHeaderRows: 6,
  };
  const assetBalanceCSV = useCSVReader(options);
  // フェッチ済みフラグ（多重実行防止）
  const hasFetched = useRef(false);

  // DBから保有銘柄データを取得
  const fetchAssetBalances = useCallback(async (force = false) => {
    // 認証状態が確定していない場合は待機
    if (authLoading) return;
    // 未認証の場合はスキップ
    if (!isAuthenticated) return;
    // 既にフェッチ済みで強制更新でない場合はスキップ
    if (hasFetched.current && !force) return;

    setDbLoading(true);
    setDbError(null);
    try {
      const data = await assetBalanceApi.list();
      setSavedData(data || []);
      setAssetBalanceData(data || []);
      hasFetched.current = true;
    } catch (error) {
      const message = error instanceof Error ? error.message : 'データ取得に失敗しました';
      setDbError(message);
    } finally {
      setDbLoading(false);
    }
  }, [isAuthenticated, authLoading]);

  // 認証状態が確定したらDBからデータを取得
  useEffect(() => {
    if (!authLoading) {
      fetchAssetBalances();
    }
  }, [authLoading, fetchAssetBalances]);

  // ログアウト時に全データをクリア
  useEffect(() => {
    return onLogout(() => {
      setSavedData([]);
      setAssetBalanceData([]);
      setAssetBalanceCsvData([]);
      setDbError(null);
      hasFetched.current = false;
      assetBalanceCSV.reset();
    });
  }, [onLogout, assetBalanceCSV]);

  const handleFileSelect = async (file: File) => {
    try {
      setAssetBalanceCsvData(await assetBalanceCSV.parseCSV(file));
      if (assetBalanceCSV.error) assetBalanceCSV.resetError();
    } catch (error) {
      console.error('AssetBalance CSV処理エラー:', error);
    }
  };

  useEffect(() => {
    if (assetBalanceCsvData.length > 0) {
      setAssetBalanceData(tmpAssetBalanceData.filter(item => item.security_code !== ''));
    }
  }, [tmpAssetBalanceData, assetBalanceCsvData]);

  const handleSaveToDb = async () => {
    if (!isAuthenticated) return;
    if (assetBalanceCsvData.length === 0) return;
    setSaving(true);
    setDbError(null);
    try {
      await assetBalanceApi.bulkCreate(assetBalanceData);
      setAssetBalanceCsvData([]);
      await fetchAssetBalances(true);
    } catch (error) {
      const message = error instanceof Error ? error.message : '保存に失敗しました';
      setDbError(message);
    } finally {
      setSaving(false);
    }
  };

  const handleDeleteAll = async () => {
    if (!isAuthenticated) return;
    if (!window.confirm('保存された保有銘柄データを削除しますか？')) return;
    setDeleting(true);
    setDbError(null);
    try {
      await assetBalanceApi.deleteAll();
      setSavedData([]);
      setAssetBalanceData([]);
      setAssetBalanceCsvData([]);
      assetBalanceCSV.reset();
    } catch (error) {
      const message = error instanceof Error ? error.message : '削除に失敗しました';
      setDbError(message);
    } finally {
      setDeleting(false);
    }
  };

  const hasCsvData = assetBalanceCsvData.length > 0;
  const hasDbData = savedData.length > 0;

  const isProcessing = dbLoading || saving || deleting || assetBalanceCSV.isLoading || authLoading;

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
              <div style={{ width: '400px' }}>
                <CSVFileInput
                  onFileSelect={handleFileSelect}
                  selectedFileName={assetBalanceCSV.fileName || ''}
                  disabled={dbLoading || saving || deleting}
                />
              </div>
              <div className="btn-group" role="group" aria-label="データ操作">
                {hasCsvData && (
                  <button
                    className="btn btn-primary btn-sm"
                    onClick={handleSaveToDb}
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

            {(assetBalanceCSV.error || dbError) && (
              <div className="alert alert-danger my-3" role="alert" aria-live="assertive">
                <strong>エラー:</strong> {assetBalanceCSV.error || dbError}
              </div>
            )}

            <div aria-live="polite" aria-atomic="true">
              {(dbLoading || saving || deleting || assetBalanceCSV.isLoading) && (
                <div className="text-center my-4" role="status">
                  <div className="spinner-border text-primary" aria-hidden="true" />
                  <p className="mt-2 text-muted">
                    {dbLoading && 'データを読み込んでいます...'}
                    {saving && 'データを保存しています...'}
                    {deleting && 'データを削除しています...'}
                    {assetBalanceCSV.isLoading && 'CSVファイルを処理しています...'}
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
