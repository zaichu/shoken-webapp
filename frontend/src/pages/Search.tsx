import { useEffect, useMemo } from 'react';
import { useSearchParams } from 'react-router-dom';
import { Layout } from '../components/templates/Layout';
import { SearchForm } from '../components/organisms/SearchForm';
import { StockInfo } from '../components/organisms/StockInfo';
import { useStockSearch } from '../features/stock/hooks/useStockSearch';

export function SearchPage() {
  const [searchParams] = useSearchParams();
  const codeParam = searchParams.get('code');
  const normalizedCodeParam = useMemo(() => {
    if (!codeParam) return '';
    const trimmed = codeParam.trim();
    return /^[0-9A-Za-z]+$/.test(trimmed) ? trimmed : '';
  }, [codeParam]);
  const hasInvalidCodeParam = !!codeParam && !normalizedCodeParam;

  const {
    stockCode,
    setStockCode,
    stockData,
    error,
    isLoading,
    isError,
    handleSearch,
    searchByCode
  } = useStockSearch(normalizedCodeParam || undefined);

  // URLパラメータが変更された場合に検索を実行
  useEffect(() => {
    if (normalizedCodeParam && normalizedCodeParam !== stockCode) {
      searchByCode(normalizedCodeParam);
    }
  }, [normalizedCodeParam, stockCode, searchByCode]);

  return (
    <Layout>
      <div className="search-page">
        <SearchForm
          stockCode={stockCode}
          onStockCodeChange={setStockCode}
          onSubmit={handleSearch}
          isLoading={isLoading}
        />

        {hasInvalidCodeParam && (
          <div className="alert alert-warning" role="alert">
            不正な銘柄コードが指定されています。英数字で入力してください。
          </div>
        )}

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
