import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { fetchStockData } from '../api';
import { StockData } from '../types';

export function useStockSearch(initialCode?: string) {
  const [stockCode, setStockCode] = useState(initialCode || '');
  const [submittedCode, setSubmittedCode] = useState(initialCode || '');

  const {
    data: stockData,
    error,
    isLoading,
    isError
  } = useQuery<StockData, Error>({
    queryKey: ['stock', submittedCode],
    queryFn: () => fetchStockData(submittedCode),
    enabled: !!submittedCode,
    refetchOnWindowFocus: false,
  });

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault();
    setSubmittedCode(stockCode);
  };

  // 検索を直接実行（URLパラメータからの自動検索用）
  const searchByCode = (code: string) => {
    setStockCode(code);
    setSubmittedCode(code);
  };

  const resetSearch = () => {
    setStockCode('');
    setSubmittedCode('');
  };

  return {
    stockCode,
    setStockCode,
    stockData,
    error,
    isLoading,
    isError,
    handleSearch,
    searchByCode,
    resetSearch
  };
}
