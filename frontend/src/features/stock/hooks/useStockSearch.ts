import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { fetchStockData } from '../api';
import { StockData } from '../types';

export function useStockSearch(initialCode?: string) {
  const [stockCode, setStockCode] = useState(initialCode || '');
  const [searchQuery, setSearchQuery] = useState(initialCode || '');

  const {
    data: stockData,
    error,
    isLoading,
    isError
  } = useQuery<StockData, Error>({
    queryKey: ['stock', searchQuery],
    queryFn: () => fetchStockData(searchQuery),
    enabled: !!searchQuery,
  });

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault();
    setSearchQuery(stockCode);
  };

  // 検索を直接実行（URLパラメータからの自動検索用）
  const searchByCode = (code: string) => {
    setStockCode(code);
    setSearchQuery(code);
  };

  const resetSearch = () => {
    setStockCode('');
    setSearchQuery('');
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
