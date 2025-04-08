import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { fetchStockData } from '../services/api';
import { StockData } from '../data/stock';

export function useStockSearch() {
  const [stockCode, setStockCode] = useState('');
  const [searchQuery, setSearchQuery] = useState('');

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
    resetSearch
  };
}
