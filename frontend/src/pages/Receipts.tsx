import { useState } from 'react';
import { useCSVReader } from '../hooks/useCSVReader';
import { CSVFileInput } from '../components/CSVFileInput';
import { Layout } from './Layout';
import { DividendList } from './receipts/DividendList';
import { DomesticStock } from './receipts/DomesticStock';
import { MutualFund } from './receipts/MutualFund';

type ReceiptType = 'dividend' | 'domestic' | 'mutual-fund';

export const Receipts = () => {
  const [activeTab, setActiveTab] = useState<ReceiptType>('dividend');
  const { parseCSV, isLoading, error, fileName } = useCSVReader();
  const [csvData, setCsvData] = useState<any[]>([]);

  const handleFileSelect = async (file: File) => {
    try {
      const data = await parseCSV(file);
      setCsvData(data);
    } catch (err) {
      console.error("ファイル処理エラー:", err);
    }
  };

  const renderContent = () => {
    switch (activeTab) {
      case 'dividend':
        return <DividendList csvData={csvData} />;
      case 'domestic':
        return <DomesticStock csvData={csvData} />;
      case 'mutual-fund':
        return <MutualFund csvData={csvData} />;
      default:
        return <DividendList csvData={csvData} />;
    }
  };

  return (
    <Layout>
      <h2>受取金一覧</h2>

      <div className="mb-3">
        <CSVFileInput onFileSelect={handleFileSelect} />
        {isLoading && <p className="mt-2">ファイル処理中...</p>}
        {error && <p className="text-danger mt-2">{error}</p>}
      </div>

      <ul className="nav nav-tabs mb-4">
        <li className="nav-item">
          <button
            className={`nav-link ${activeTab === 'dividend' ? 'active' : ''}`}
            onClick={() => setActiveTab('dividend')}
          >
            配当金
          </button>
        </li>
        <li className="nav-item">
          <button
            className={`nav-link ${activeTab === 'domestic' ? 'active' : ''}`}
            onClick={() => setActiveTab('domestic')}
          >
            国内株式
          </button>
        </li>
        <li className="nav-item">
          <button
            className={`nav-link ${activeTab === 'mutual-fund' ? 'active' : ''}`}
            onClick={() => setActiveTab('mutual-fund')}
          >
            投資信託
          </button>
        </li>
      </ul>

      {renderContent()}
    </Layout>
  );
}
