import { SearchCategories } from '@/types/common';

export type SearchKey = 'securities' | 'years' | 'products' | 'accounts' | 'date';
export type ActiveSearchType = SearchKey | null;
export type DateSegment = '年' | '月' | '日' | '範囲';
export type DateInputs = {
    yearValue: string;
    monthValue: string;
    dateValue: string;
    rangeStart: string;
    rangeEnd: string;
};

export interface SearchCardProps {
    onSearch: (query: string) => void;
    categories?: SearchCategories;
    onExpandToggle?: (isExpanded: boolean) => void; // 展開状態変更の通知
    value?: string; // 親の検索状態と同期（外部クリア対応）
    initialExpanded?: boolean; // 初期展開状態（デフォルト: true）
    compact?: boolean; // コンパクトモード: aside などで lg:grid-cols-4 を抑制する
}
