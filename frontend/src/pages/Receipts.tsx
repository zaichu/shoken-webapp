import { useState } from 'react';
import { Layout } from './Layout';
import { ReceiptTemplate } from '../components/templates/ReceiptTemplate';
import { CSVFileInput } from '../components/molecules/CSVFileInput';
import { useCSVReader } from '../hooks/useCSVReader';

/**
 * 受取金管理ページコンポーネント
 */
export function ReceiptsPage() {
  const { parseCSV, isLoading, error, resetError } = useCSVReader();
  const [dividendItems, setDividendItems] = useState<any[]>([]);
  const [stockCodes, setStockCodes] = useState<string[]>([]);
  const [filter, setFilter] = useState('');
  
  /**
   * CSVファイル選択時のハンドラ
   */
  const handleFileSelect = async (file: File) => {
    try {
      const data = await parseCSV(file);
      
      // 銘柄コードのセット
      const codes = Array.from(
        new Set(
          data
            .filter(item => item['銘柄コード'])
            .map(item => item['銘柄コード'])
        )
      );
      
      setDividendItems(data);
      setStockCodes(codes);
      
      // エラーがあればリセット
      if (error) resetError();
    } catch (e) {
      console.error('CSV処理エラー:', e);
    }
  };
  
  /**
   * 検索オプションの生成
   */
  const searchOptions = stockCodes.map(code => ({
    value: code,
    label: code
  }));
  
  /**
   * フィルタリングされた配当データの取得
   */
  const filteredData = filter
    ? dividendItems.filter(item => item['銘柄コード'] === filter)
    : dividendItems;
  
  return (
    <Layout>
      <div className="receipt-page">
        <h2 className="mb-4">受取金管理</h2>
        
        <div className="mb-4">
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
        
        {dividendItems.length > 0 && !isLoading && (
          <ReceiptTemplate
            title="配当金情報"
            searchQuery={filter}
            onSearch={setFilter}
            searchOptions={searchOptions}
          >
            <div className="table-responsive">
              <table className="table table-striped mb-0">
                <thead className="bg-light">
                  <tr>
                    <th>入金日</th>
                    <th>銘柄コード</th>
                    <th>銘柄名</th>
                    <th>単価</th>
                    <th>数量</th>
                    <th>受取金額</th>
                  </tr>
                </thead>
                <tbody>
                  {filteredData.map((item, index) => (
                    <tr key={index}>
                      <td>{item['入金日']}</td>
                      <td>{item['銘柄コード']}</td>
                      <td>{item['銘柄']}</td>
                      <td>{item['単価[円/現地通貨]']}</td>
                      <td>{item['数量[株/口]']}</td>
                      <td>{item['受取金額[円/現地通貨]']}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </ReceiptTemplate>
        )}
        
        {dividendItems.length === 0 && !isLoading && !error && (
          <div className="text-center my-5">
            <p className="text-muted">CSVファイルをアップロードして受取金情報を表示します。</p>
          </div>
        )}
      </div>
    </Layout>
  );
}
