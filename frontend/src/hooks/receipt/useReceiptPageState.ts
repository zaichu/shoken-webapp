import { useState } from 'react';
import { filterByConfig, FilterConfig } from '@/lib/utils/searchUtils';

export function useReceiptPageState<T>(
    data: T[],
    filterConfig: FilterConfig<T>
) {
    const [searchQuery, setSearchQuery] = useState('');
    const filteredData = filterByConfig(data, searchQuery, filterConfig);
    return { searchQuery, setSearchQuery, filteredData } as const;
}
