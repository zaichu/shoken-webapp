import { SearchCategories } from '@/types/common';
import { DateInputs, DateSegment, SearchKey } from './types';

export const SEARCH_ORDER = ['date', 'securities', 'years', 'products', 'accounts'] as const satisfies readonly SearchKey[];

export const EMPTY_DATE_INPUTS: DateInputs = {
    yearValue: '',
    monthValue: '',
    dateValue: '',
    rangeStart: '',
    rangeEnd: '',
};

export function createEmptySelectedQueries(): Record<SearchKey, string> {
    return {
        securities: '',
        years: '',
        products: '',
        accounts: '',
        date: '',
    };
}

export function buildRangeQuery(start: string, end: string): string {
    if (!start && !end) return '';
    return `${start}..${end}`;
}

export function formatQueryToken(value: string): string {
    const trimmed = value.trim();
    if (!/\s/.test(trimmed)) return trimmed;
    return `"${trimmed.replace(/"/g, '\\"')}"`;
}

export function buildCombinedQuery(queries: Record<SearchKey, string>): string {
    return SEARCH_ORDER
        .map(key => queries[key].trim())
        .filter(Boolean)
        .map(formatQueryToken)
        .join(' ');
}

export function getInitialDateSegment(cats: SearchCategories | undefined): DateSegment {
    if ((cats?.years?.length ?? 0) > 0) return '年';
    if (cats?.dates) return '月';
    return '年';
}

export const formatDateLabel = (value: string): string => value.replace(/-/g, "/");
