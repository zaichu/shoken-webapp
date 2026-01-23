import { useEffect } from 'react';
import { useSearchParams } from 'react-router-dom';
import { Layout } from '../components/templates/Layout';
import { SearchForm } from '../components/organisms/SearchForm';
import { StockInfo } from '../components/organisms/StockInfo';
import { useStockSearch } from '../features/stock/hooks/useStockSearch';

export function SearchPage() {
  const [searchParams] = useSearchParams();
  const codeParam = searchParams.get('code');

  const {
    stockCode,
    setStockCode,
    stockData,
    error,
    isLoading,
    isError,
    handleSearch,
    searchByCode
  } = useStockSearch(codeParam || undefined);

  // URLパラメータが変更された場合に検索を実行
  useEffect(() => {
    if (codeParam && codeParam !== stockCode) {
      searchByCode(codeParam);
    }
  }, [codeParam]);

  return (
    <Layout>
      <div className="search-page">
        <SearchForm
          stockCode={stockCode}
          onStockCodeChange={setStockCode}
          onSubmit={handleSearch}
          isLoading={isLoading}
        />

        {isError && (
          <div className="alert alert-danger" role="alert">
            <strong>エラー:</strong> {error?.message || '銘柄情報の取得に失敗しました。'}
          </div>
        )}

        {stockData && !isError && (
          <StockInfo stockData={stockData} />
        )}

        {!stockData && !isError && !isLoading && (
          <div className="text-center my-5">
            <p className="text-muted">銘柄コードを入力して検索してください。</p>
          </div>
        )}
      </div>
    </Layout>
  );
}
