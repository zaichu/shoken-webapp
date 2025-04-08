import { useState } from 'react';
import { useCSVReader } from '../hooks/useCSVReader';
import { CSVFileInput } from '../components/molecules/CSVFileInput';
import { Layout } from './Layout';
import { DividendList } from './receipts/DividendList';
import { DomesticStock } from './receipts/DomesticStock';
import { MutualFund } from './receipts/MutualFund';
import { Button } from '../components/atoms/Button';

type ReceiptType = 'Dividend' | 'DomesticStock' | 'MutualFund';

export const Receipts = () => {
  const [activeTab, setActiveTab] = useState<ReceiptType>('Dividend');
  const { parseCSV, isLoading, error, resetError } = useCSVReader();
  const [csvData, setCsvData] = useState<any[]>([]);

  const handleFileSelect = async (file: File) => {
    try {
      const data = await parseCSV(file);
      setCsvData(data);
      resetError();
    } catch (err) {
      console.error("ファイル処理エラー:", err);
    }
  };

  const renderContent = () => {
    switch (activeTab) {
      case 'Dividend':
        return <DividendList csvData={csvData} />;
      case 'DomesticStock':
        return <DomesticStock csvData={csvData} />;
      case 'MutualFund':
        return <MutualFund csvData={csvData} />;
    }
  };

  return (
    <Layout>
      <div className="mb-3">
        <CSVFileInput onFileSelect={handleFileSelect} />
        {isLoading && (
          <div className="d-flex align-items-center mt-2">
            <div className="spinner-border spinner-border-sm me-2" role="status">
              <span className="visually-hidden">読み込み中...</span>
            </div>
            <span>ファイル処理中...</span>
          </div>
        )}
        {error && (
          <div className="alert alert-danger d-flex align-items-center mt-2">
            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor"
              className="bi bi-exclamation-triangle-fill flex-shrink-0 me-2" viewBox="0 0 16 16">
              <path d="M8.982 1.566a1.13 1.13 0 0 0-1.96 0L.165 13.233c-.457.778.091 1.767.98 1.767h13.713c.889 0 1.438-.99.98-1.767L8.982 1.566zM8 5c.535 0 .954.462.9.995l-.35 3.507a.552.552 0 0 1-1.1 0L7.1 5.995A.905.905 0 0 1 8 5zm.002 6a1 1 0 1 1 0 2 1 1 0 0 1 0-2z" />
            </svg>
            <div>
              {error}
              <Button
                size="sm"
                className="ms-2"
                onClick={resetError}
              >
                閉じる
              </Button>
            </div>
          </div>
        )}
      </div>

      <ul className="nav nav-tabs mb-4">
        <li className="nav-item">
          <button
            className={`nav-link ${activeTab === 'Dividend' ? 'active' : ''}`}
            onClick={() => setActiveTab('Dividend')}
          >
            配当金
          </button>
        </li>
        <li className="nav-item">
          <button
            className={`nav-link ${activeTab === 'DomesticStock' ? 'active' : ''}`}
            onClick={() => setActiveTab('DomesticStock')}
          >
            国内株式
          </button>
        </li>
        <li className="nav-item">
          <button
            className={`nav-link ${activeTab === 'MutualFund' ? 'active' : ''}`}
            onClick={() => setActiveTab('MutualFund')}
          >
            投資信託
          </button>
        </li>
      </ul>

      {csvData.length > 0 ? (
        renderContent()
      ) : (
        <div className="alert alert-info">
          データを表示するにはCSVファイルをアップロードしてください。
        </div>
      )}
    </Layout>
  );
}
