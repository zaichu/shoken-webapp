import { useState } from 'react';
import { Layout } from '../components/templates/Layout';
import { CSVFileInput } from '../components/molecules/CSVFileInput';
import { useCSVReader } from '../hooks/useCSVReader';
import { Dividend } from './Receipt/Dividend';
import { DomesticStock } from './Receipt/DomesticStock';
import { Mutualfund } from './Receipt/Mutualfund';

type ReceiptsType = 'dividend' | 'domesticstock' | 'mutualfund';

/**
 * 明細種類ごとにCSVデータを管理するページコンポーネント
 */
export function ReceiptsPage() {
  // アクティブなタブの状態管理
  const [receiptsType, setReceiptsType] = useState<ReceiptsType>('dividend');

  // 各明細種類ごとのCSVデータを個別に管理
  const [dividendCsvData, setDividendCsvData] = useState<any[]>([]);
  const [domesticStockCsvData, setDomesticStockCsvData] = useState<any[]>([]);
  const [mutualfundCsvData, setMutualfundCsvData] = useState<any[]>([]);

  // 各明細種類ごとにCSVリーダーフックを作成
  const dividendCSV = useCSVReader();
  const domesticStockCSV = useCSVReader();
  const mutualfundCSV = useCSVReader();

  /**
   * 現在選択中のタブに応じてCSV処理を切り替える
   */
  const handleFileSelect = async (file: File) => {
    try {
      switch (receiptsType) {
        case 'dividend':
          setDividendCsvData(await dividendCSV.parseCSV(file));
          if (dividendCSV.error) dividendCSV.resetError();
          break;
        case 'domesticstock':
          setDomesticStockCsvData(await domesticStockCSV.parseCSV(file));
          if (domesticStockCSV.error) domesticStockCSV.resetError();
          break;
        case 'mutualfund':
          setMutualfundCsvData(await mutualfundCSV.parseCSV(file));
          if (mutualfundCSV.error) mutualfundCSV.resetError();
          break;
      }
    } catch (e) {
      console.error('CSV処理エラー:', e);
    }
  };

  /**
   * 現在選択中のタブに対応するエラーとローディング状態を取得
   */
  const getCurrentCSVState = () => {
    switch (receiptsType) {
      case 'dividend':
        return { isLoading: dividendCSV.isLoading, error: dividendCSV.error, fileName: dividendCSV.fileName };
      case 'domesticstock':
        return { isLoading: domesticStockCSV.isLoading, error: domesticStockCSV.error, fileName: domesticStockCSV.fileName };
      case 'mutualfund':
        return { isLoading: mutualfundCSV.isLoading, error: mutualfundCSV.error, fileName: mutualfundCSV.fileName };
    }
  };

  const { isLoading, error, fileName } = getCurrentCSVState();

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
        <div className="row">
          <div className='col'><CSVFileInput onFileSelect={handleFileSelect} selectedFileName={fileName} /></div>
        </div>

        {error && (
          <div className="alert alert-danger my-3" role="alert">
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

        {receiptsType === 'dividend' && <Dividend csvData={dividendCsvData} />}
        {receiptsType === 'domesticstock' && <DomesticStock csvData={domesticStockCsvData} />}
        {receiptsType === 'mutualfund' && <Mutualfund csvData={mutualfundCsvData} />}
      </div>
    </Layout>
  );
}
