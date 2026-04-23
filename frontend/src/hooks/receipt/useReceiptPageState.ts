import { useState } from 'react';
import { filterByConfig, FilterConfig } from '@/lib/utils/searchUtils';

/**
 * 受取金ページ共通のフィルタ状態管理フック
 * @param data フィルタ対象のデータ配列
 * @param filterConfig フィルタ設定
 */
export function useReceiptPageState<T>(
    data: T[],
    filterConfig: FilterConfig<T>
) {
    const [searchQuery, setSearchQuery] = useState('');
    const filteredData = filterByConfig(data, searchQuery, filterConfig);
    return { searchQuery, setSearchQuery, filteredData } as const;
}
