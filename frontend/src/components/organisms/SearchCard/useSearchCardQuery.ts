import { useEffect, useRef, useState } from 'react';
import { SearchCategories } from '@/types/common';
import { DateInputs, DateSegment, SearchKey } from './types';
import {
    EMPTY_DATE_INPUTS,
    buildCombinedQuery,
    buildRangeQuery,
    createEmptySelectedQueries,
    getInitialDateSegment,
} from './searchQueryUtils';

interface UseSearchCardQueryArgs {
    categories?: SearchCategories;
    value?: string; // 親の検索状態と同期（外部クリア対応）
    onSearch: (query: string) => void;
}

/**
 * SearchCard の「API クエリ params state」を一元管理するフック。
 * 展開/折りたたみなどの UI 表示 state は呼び出し側（SearchCard）が持つ。
 */
export function useSearchCardQuery({ categories, value, onSearch }: UseSearchCardQueryArgs) {
    const [selectedQueries, setSelectedQueries] = useState<Record<SearchKey, string>>(
        createEmptySelectedQueries()
    );

    const [dateSegment, setDateSegment] = useState<DateSegment>(() =>
        getInitialDateSegment(categories)
    );
    const [dateInputs, setDateInputs] = useState<DateInputs>({ ...EMPTY_DATE_INPUTS });
    const [isYearPickerOpen, setIsYearPickerOpen] = useState(false);

    const categoriesRef = useRef(categories);
    categoriesRef.current = categories;

    useEffect(() => {
        if (value !== '') return;
        const cats = categoriesRef.current;
        setSelectedQueries(createEmptySelectedQueries());
        setDateInputs({ ...EMPTY_DATE_INPUTS });
        setIsYearPickerOpen(false);
        setDateSegment(getInitialDateSegment(cats));
    }, [value]);

    const effectiveSelectedQueries = value === '' ? createEmptySelectedQueries() : selectedQueries;
    const hasActiveSearch = Object.values(effectiveSelectedQueries).some(Boolean);
    const isDefaultState = !hasActiveSearch;

    const resetDateInputs = () => {
        setDateInputs({ ...EMPTY_DATE_INPUTS });
        setIsYearPickerOpen(false);
    };

    const applySelectedQueries = (nextQueries: Record<SearchKey, string>) => {
        setSelectedQueries(nextQueries);
        onSearch(buildCombinedQuery(nextQueries));
    };

    const handleQuickSearch = (val: string, searchType: 'securities' | 'years' | 'products' | 'accounts') => {
        const shouldToggleOff =
            (searchType === 'products' || searchType === 'accounts') &&
            effectiveSelectedQueries[searchType] === val;
        applySelectedQueries({
            ...effectiveSelectedQueries,
            [searchType]: val === '' || shouldToggleOff ? '' : val,
        });
    };

    const handleDateSearch = (val: string) => {
        applySelectedQueries({
            ...effectiveSelectedQueries,
            date: val,
        });
    };

    const handleSegmentChange = (segment: DateSegment) => {
        if (segment === dateSegment) return;
        setDateSegment(segment);
        if (effectiveSelectedQueries.date) {
            applySelectedQueries({
                ...effectiveSelectedQueries,
                date: '',
            });
        }
        resetDateInputs();
    };

    const handleYearOptionSelect = (val: string) => {
        setDateInputs(d => ({ ...d, yearValue: val }));
        setIsYearPickerOpen(false);
        handleDateSearch(val);
    };

    const handleMonthChange = (val: string) => {
        setDateInputs(d => ({ ...d, monthValue: val }));
        handleDateSearch(val);
    };

    const handleDateValueChange = (val: string) => {
        setDateInputs(d => ({ ...d, dateValue: val }));
        handleDateSearch(val);
    };

    const handleRangeStartChange = (val: string) => {
        setDateInputs(d => ({ ...d, rangeStart: val }));
        const query = buildRangeQuery(val, dateInputs.rangeEnd);
        handleDateSearch(query);
    };

    const handleRangeEndChange = (val: string) => {
        setDateInputs(d => ({ ...d, rangeEnd: val }));
        const query = buildRangeQuery(dateInputs.rangeStart, val);
        handleDateSearch(query);
    };

    const handleClearSearch = () => {
        setSelectedQueries(createEmptySelectedQueries());
        setDateSegment(getInitialDateSegment(categories));
        resetDateInputs();
        onSearch('');
    };

    return {
        effectiveSelectedQueries,
        hasActiveSearch,
        isDefaultState,
        dateSegment,
        dateInputs,
        isYearPickerOpen,
        toggleYearPicker: () => setIsYearPickerOpen(open => !open),
        closeYearPicker: () => setIsYearPickerOpen(false),
        handleQuickSearch,
        handleSegmentChange,
        handleYearOptionSelect,
        handleMonthChange,
        handleDateValueChange,
        handleRangeStartChange,
        handleRangeEndChange,
        handleClearSearch,
    };
}
