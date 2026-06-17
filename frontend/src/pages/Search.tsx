import { useEffect } from 'react';
import { useSearchParams } from 'react-router-dom';
import { Layout } from '../components/templates/Layout';
import { SearchForm } from '../components/organisms/SearchForm';
import { StockInfo } from '../components/organisms/StockInfo';
import { Alert } from '../components/atoms/Alert';
import { EmptyState } from '../components/atoms/EmptyState';
import { PageHeader } from '../components/atoms/PageHeader';
import { useStockSearch } from '../features/stock/hooks/useStockSearch';
import { usePageTitle } from '../hooks/usePageTitle';
import { ApiError } from '@/lib/types/api';
import { SECURITY_CODE_REGEX } from '@/lib/utils/formatters';

export function SearchPage() {
  usePageTitle('銘柄検索');

  const [searchParams] = useSearchParams();
  const codeParam = searchParams.get('code');
  const normalizedCodeParam = (() => {
    if (!codeParam) return '';
    const trimmed = codeParam.trim();
    return SECURITY_CODE_REGEX.test(trimmed) ? trimmed : '';
  })();
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
      <div className="page-surface">
        <PageHeader
          title="銘柄検索"
          eyebrow="Search"
          description="銘柄コードまたは銘柄名を入力して株式情報を検索できます。"
        />
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
            <strong>エラー:</strong>{' '}
            {error instanceof ApiError
              ? error.getUserMessage()
              : (error?.message || '銘柄情報の取得に失敗しました。')}
          </Alert>
        )}

        {stockData && !isError && (
          <StockInfo stockData={stockData} />
        )}

        {!stockData && !isError && !isLoading && (
          <>
            <EmptyState
              icon={
                <svg className="h-10 w-10" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                </svg>
              }
              title="銘柄を検索"
              description="銘柄コード（例：7203）または銘柄名を入力して検索してください。"
              className="py-10"
            />
            <div className="mt-6 rounded-xl border border-slate-950/10 bg-slate-50/80 p-4">
              <h3 className="mb-2 text-sm font-black text-slate-800">検索のヒント</h3>
              <ul className="space-y-1 text-sm font-medium text-slate-600">
                <li>4桁の銘柄コードで検索できます（例：7203, 9984）</li>
                <li>会社名の一部でも検索できます（例：トヨタ）</li>
                <li>検索結果から各種証券サイトへのリンクを確認できます</li>
              </ul>
            </div>
          </>
        )}
      </div>
    </Layout>
  );
}
