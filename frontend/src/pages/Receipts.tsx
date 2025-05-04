import { useState } from 'react';
import { Layout } from '../components/templates/Layout';
import { CSVFileInput } from '../components/molecules/CSVFileInput';
import { useCSVReader } from '../hooks/useCSVReader';
import { Dividend } from './Receipt/Dividend';
import { DomesticStock } from './Receipt/DomesticStock';
import { Mutualfund } from './Receipt/Mutualfund';

type ReceiptsType = 'dividend' | 'domesticstock' | 'mutualfund';

export function ReceiptsPage() {
  const [receiptsType, setReceiptsType] = useState<ReceiptsType>('dividend');
  const { parseCSV, isLoading, error, resetError } = useCSVReader();
  const [csvData, setCsvData] = useState<any[]>([]);

  const handleFileSelect = async (file: File) => {
    try {
      setCsvData(await parseCSV(file));
      if (error) resetError();
    } catch (e) {
      console.error('CSV処理エラー:', e);
    }
  };

  return (
    <Layout>
      <nav className="nav nav-tabs">
        <ul className="nav nav-tabs">
          <li className="nav-item">
            <button
              className={receiptsType === 'dividend' ? 'nav-link active' : 'nav-link'}
              onClick={() => setReceiptsType('dividend')}
            >
              配当金
            </button>
          </li>
          <li className="nav-item">
            <button
              className={receiptsType === 'domesticstock' ? 'nav-link active' : 'nav-link'}
              onClick={() => setReceiptsType('domesticstock')}
            >
              国内株式
            </button>
          </li>
          <li className="nav-item">
            <button
              className={receiptsType === 'mutualfund' ? 'nav-link active' : 'nav-link'}
              onClick={() => setReceiptsType('mutualfund')}
            >
              投資信託
            </button>
          </li>
        </ul>
      </nav>
      <div className="receipt-page mt-2">
        <div>
          <CSVFileInput onFileSelect={handleFileSelect} />
        </div>

        {error && (
          <div className="alert alert-danger mb-4" role="alert">
            <strong>エラー:</strong> {error}
          </div>
        )}

        {isLoading && (
          <div className="text-center my-4">
            <div className="spinner-border text-primary" role="status">
              <span className="visually-hidden">Loading...</span>
            </div>
          </div>
        )}

        {RenderReceipt(receiptsType, csvData)}

      </div>
    </Layout >
  );
}

const RenderReceipt = (type: ReceiptsType, csvData: any[]) => {
  switch (type) {
    case 'dividend':
      return <Dividend csvData={csvData} />
    case 'domesticstock':
      return <DomesticStock csvData={csvData} />
    case 'mutualfund':
      return <Mutualfund csvData={csvData} />
    default:
      return null
  }
}