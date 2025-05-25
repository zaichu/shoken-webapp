import { useState } from 'react';
import { Layout } from '../components/templates/Layout';
import { CSVFileInput } from '../components/molecules/CSVFileInput';
import { useCSVReader } from '../hooks/useCSVReader';
import { Holdings } from './Receipt/Holdings';

/**
 * 保有株管理ページコンポーネント
 */
export function HoldingsPage() {
  // 保有株CSVデータの状態管理
  const [holdingsCsvData, setHoldingsCsvData] = useState<Record<string, unknown>[]>([]);

  // CSVリーダーフック
  const holdingsCSV = useCSVReader();

  /**
   * CSVファイル選択時の処理
   */
  const handleFileSelect = async (file: File) => {
    try {
      setHoldingsCsvData(await holdingsCSV.parseCSV(file));
      if (holdingsCSV.error) holdingsCSV.resetError();
    } catch (e) {
      console.error('CSV処理エラー:', e);
    }
  };

  return (
    <Layout>
      <div className="holdings-page">
        <div className="row mb-3">
          <div className='col'>
            <CSVFileInput
              onFileSelect={handleFileSelect}
              selectedFileName={holdingsCSV.fileName || ''}
            />
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

        {holdingsCsvData.length > 0 && <Holdings csvData={holdingsCsvData} />}
      </div>
    </Layout>
  );
}
