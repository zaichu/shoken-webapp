import { Layout } from './Layout';
import { useStockSearch } from '../hooks/useStockSearch';
import { SearchForm } from '../components/organisms/SearchForm';
import { StockInfo } from '../components/organisms/StockInfo';
import { ErrorPage } from '../components/templates/ErrorPage';

export const Search = () => {
  const {
    stockCode,
    setStockCode,
    stockData,
    error,
    isLoading,
    handleSearch,
    resetSearch,
  } = useStockSearch();

  return (
    <Layout>
      <SearchForm
        stockCode={stockCode}
        onStockCodeChange={setStockCode}
        onSubmit={handleSearch}
        isLoading={isLoading}
      />

      {error && (
        <div className="mb-4">
          <ErrorPage
            title="検索エラー"
            message={`エラーが発生しました: ${error.message}`}
            onRetry={resetSearch}
          />
        </div>
      )}

      {stockData && <StockInfo data={stockData} />}
    </Layout>
  );
}
