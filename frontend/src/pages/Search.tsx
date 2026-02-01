import { useEffect, useMemo } from 'react';
import { useSearchParams } from 'react-router-dom';
import { Layout } from '../components/templates/Layout';
import { SearchForm } from '../components/organisms/SearchForm';
import { StockInfo } from '../components/organisms/StockInfo';
import { Alert } from '../components/atoms/Alert';
import { EmptyState } from '../components/atoms/EmptyState';
import { useStockSearch } from '../features/stock/hooks/useStockSearch';
import { SECURITY_CODE_REGEX } from '@/lib/utils/formatters';

export function SearchPage() {
  const [searchParams] = useSearchParams();
  const codeParam = searchParams.get('code');
  const normalizedCodeParam = useMemo(() => {
    if (!codeParam) return '';
    const trimmed = codeParam.trim();
    return SECURITY_CODE_REGEX.test(trimmed) ? trimmed : '';
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
      <div>
        <SearchForm
          stockCode={stockCode}
          onStockCodeChange={setStockCode}
          onSubmit={handleSearch}
          isLoading={isLoading}
        />

        {hasInvalidCodeParam && (
          <Alert variant="warning">
            不正な銘柄コードが指定されています。
          </Alert>
        )}

        {isError && (
          <Alert variant="danger">
            <strong>エラー:</strong> {error?.message || '銘柄情報の取得に失敗しました。'}
          </Alert>
        )}

        {stockData && !isError && (
          <StockInfo stockData={stockData} />
        )}

        {!stockData && !isError && !isLoading && (
          <EmptyState
            icon={
              <svg className="h-12 w-12" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
              </svg>
            }
            title="銘柄を検索"
            description="銘柄コード（例：7203）を入力して検索してください。日本株の情報を表示します。"
            className="py-16"
          />
        )}
      </div>
    </Layout>
  );
}
