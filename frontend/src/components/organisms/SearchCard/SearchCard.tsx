import React, { useState, useRef, useEffect } from 'react';
import { SearchCategories } from '@/types/common';
import { Button } from '@/components/atoms/Button';
import { Card, CardBody, CardHeader } from '@/components/atoms/Card';
import { cn } from '@/lib/utils/classNames';

type SearchKey = 'securities' | 'years' | 'products' | 'accounts' | 'date';
type ActiveSearchType = SearchKey | null;
type DateSegment = '年' | '月' | '日' | '範囲';

const DATE_SEGMENTS: DateSegment[] = ['年', '月', '日', '範囲'];
const SEARCH_ORDER = ['date', 'securities', 'years', 'products', 'accounts'] as const satisfies readonly SearchKey[];

function createEmptySelectedQueries(): Record<SearchKey, string> {
    return {
        securities: '',
        years: '',
        products: '',
        accounts: '',
        date: '',
    };
}

function buildRangeQuery(start: string, end: string): string {
    if (!start && !end) return '';
    return `${start}..${end}`;
}

function formatQueryToken(value: string): string {
    const trimmed = value.trim();
    if (!/\s/.test(trimmed)) return trimmed;
    return `"${trimmed.replace(/"/g, '\\"')}"`;
}

function buildCombinedQuery(queries: Record<SearchKey, string>): string {
    return SEARCH_ORDER
        .map(key => queries[key].trim())
        .filter(Boolean)
        .map(formatQueryToken)
        .join(' ');
}

interface QuickSearchDropdownProps {
    items: { value: string; label: string }[];
    id: string;
    searchType: 'securities' | 'years';
    activeSearchType: ActiveSearchType;
    searchQuery: string;
    onSearch: (value: string, searchType: 'securities' | 'years') => void;
}

const QuickSearchDropdown: React.FC<QuickSearchDropdownProps> = ({
    items, id, searchType, activeSearchType, searchQuery, onSearch
}) => {
    const displayValue = activeSearchType === searchType ? searchQuery : '';
    const isSelected = activeSearchType === searchType && displayValue !== '';
    return (
        <div className="relative">
            <select
                id={id}
                className={cn(
                    'w-full rounded-md border px-3 py-2 text-sm transition-colors focus:outline-none focus:ring-2',
                    isSelected
                        ? 'border-amber-500 bg-amber-50 text-amber-900 font-bold ring-2 ring-amber-500/30'
                        : 'border-slate-300 bg-white text-dark hover:border-slate-500 focus:border-amber-600 focus:ring-amber-500/25'
                )}
                value={displayValue}
                onChange={(e) => onSearch(e.target.value, searchType)}
            >
                <option value="">全て表示</option>
                {items.map((option) => (
                    <option key={option.value} value={option.value}>
                        {option.label}
                    </option>
                ))}
            </select>
            {isSelected && (
                <span className="absolute right-8 top-1/2 -translate-y-1/2 text-amber-700 pointer-events-none">
                    <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                        <path fillRule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clipRule="evenodd" />
                    </svg>
                </span>
            )}
        </div>
    );
};

interface QuickSearchButtonsProps {
    items: string[];
    searchType: 'products' | 'accounts';
    activeSearchType: ActiveSearchType;
    searchQuery: string;
    onSearch: (value: string, searchType: 'products' | 'accounts') => void;
}

const QuickSearchButtons: React.FC<QuickSearchButtonsProps> = ({
    items, searchType, activeSearchType, searchQuery, onSearch
}) => {
    if (!items || items.length === 0) return null;
    const hasSelection = activeSearchType === searchType;
    return (
        <>
            {items.map((item, index) => {
                const isSelected = activeSearchType === searchType && searchQuery === item;
                return (
                    <React.Fragment key={item}>
                        <Button
                            type="button"
                            variant={isSelected ? 'primary' : 'outline-secondary'}
                            size="sm"
                            onClick={() => onSearch(item, searchType)}
                            className={isSelected
                                ? 'ring-2 ring-amber-500 ring-offset-1 font-bold shadow-md'
                                : `${hasSelection ? 'opacity-50' : 'opacity-80'} hover:opacity-100 hover:bg-slate-100 hover:ring-1 hover:ring-slate-300 focus-visible:ring-2 focus-visible:ring-amber-500/50 focus-visible:outline-none`
                            }
                            aria-pressed={isSelected}
                            aria-label={isSelected ? `${item}（選択中）` : item}
                        >
                            {isSelected && (
                                <svg className="w-3.5 h-3.5 mr-1 inline-block" fill="currentColor" viewBox="0 0 20 20" aria-hidden="true">
                                    <path fillRule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clipRule="evenodd" />
                                </svg>
                            )}
                            {item}
                        </Button>
                        {index % 10 === 9 && <div className="mt-1" />}
                    </React.Fragment>
                );
            })}
        </>
    );
};

type DropdownSearchType = 'securities' | 'years';
type ButtonsSearchType = 'products' | 'accounts';

const DROPDOWN_CONFIGS: Array<{ key: DropdownSearchType; label: string }> = [
    { key: 'securities', label: '銘柄' },
    { key: 'years', label: '西暦' },
];

const BUTTONS_CONFIGS: Array<{ key: ButtonsSearchType; label: string }> = [
    { key: 'products', label: '商品' },
    { key: 'accounts', label: '口座' },
];

interface SearchFieldsGridProps {
    categories: SearchCategories;
    gridClassName: string;
    selectedQueries: Record<SearchKey, string>;
    onSearch: (value: string, searchType: 'securities' | 'years' | 'products' | 'accounts') => void;
    hasData: (data: unknown[] | undefined) => boolean;
}

const SearchFieldsGrid: React.FC<SearchFieldsGridProps> = ({
    categories, gridClassName, selectedQueries, onSearch, hasData,
}) => (
    <div className={cn('grid', gridClassName)}>
        {DROPDOWN_CONFIGS.map(({ key, label }) =>
            // yearsはdatesブロックが有効な場合は期間ブロック側で表示するため除外
            hasData(categories[key]) && !(key === 'years' && Boolean(categories.dates)) && (
                <div key={key} className="space-y-1">
                    <label htmlFor={`${key}-search`} className="text-sm font-bold text-slate-800">{label}</label>
                    <QuickSearchDropdown
                        items={categories[key]!}
                        id={`${key}-search`}
                        searchType={key}
                        activeSearchType={selectedQueries[key] ? key : null}
                        searchQuery={selectedQueries[key]}
                        onSearch={onSearch}
                    />
                </div>
            )
        )}
        {BUTTONS_CONFIGS.map(({ key, label }) =>
            hasData(categories[key]) && (
                <div key={key} className="space-y-1">
                    <div className="text-sm font-bold text-slate-800">{label}</div>
                    <div className="flex flex-wrap gap-1">
                        <QuickSearchButtons
                            items={categories[key]!}
                            searchType={key}
                            activeSearchType={selectedQueries[key] ? key : null}
                            searchQuery={selectedQueries[key]}
                            onSearch={onSearch}
                        />
                    </div>
                </div>
            )
        )}
    </div>
);

type DateInputs = { yearValue: string; monthValue: string; dateValue: string; rangeStart: string; rangeEnd: string };
const EMPTY_DATE_INPUTS: DateInputs = { yearValue: '', monthValue: '', dateValue: '', rangeStart: '', rangeEnd: '' };

function getInitialDateSegment(cats: SearchCategories | undefined): DateSegment {
    if ((cats?.years?.length ?? 0) > 0) return '年';
    if (cats?.dates) return '月';
    return '年';
}

const formatDateLabel = (value: string): string => value.replace(/-/g, "/");

interface CalendarDateButtonProps {
    label: string;
    value: string;
    inputType?: 'date' | 'month';
    onChange: (value: string) => void;
}

const CalendarDateButton: React.FC<CalendarDateButtonProps> = ({ label, value, inputType = 'date', onChange }) => {
    const inputRef = useRef<HTMLInputElement>(null);

    const handleButtonClick = () => {
        const input = inputRef.current;
        if (!input) return;
        try {
            const pickerInput = input as HTMLInputElement & { showPicker?: () => void };
            if (typeof pickerInput.showPicker === 'function') {
                pickerInput.showPicker();
                return;
            }
        } catch {
            // showPicker が失敗した場合はクリックフォールバックへ
        }
        input.focus();
        input.click();
    };

    return (
        <div className="relative">
            <button
                type="button"
                onClick={handleButtonClick}
                className={cn(
                    'w-full flex items-center gap-2 rounded-md border px-3 py-2 text-sm transition-colors focus:outline-none focus:ring-2 focus:border-amber-600 focus:ring-amber-500/25',
                    value
                        ? 'border-amber-500 bg-amber-50 text-amber-900 font-semibold'
                        : 'border-slate-300 bg-white text-slate-500 hover:border-slate-400'
                )}
            >
                <svg className="w-3.5 h-3.5 shrink-0 text-slate-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                </svg>
                <span className="flex-1 text-left">
                    {value ? formatDateLabel(value) : label}
                </span>
            </button>
            <input
                ref={inputRef}
                type={inputType}
                value={value}
                onChange={e => onChange(e.target.value)}
                aria-hidden="true"
                tabIndex={-1}
                className="absolute opacity-0 pointer-events-none w-px h-px overflow-hidden"
            />
        </div>
    );
};

interface DatePeriodBlockProps {
    dateSegment: DateSegment;
    years: { value: string; label: string }[];
    yearValue: string;
    monthValue: string;
    dateValue: string;
    rangeStart: string;
    rangeEnd: string;
    isYearPickerOpen: boolean;
    onSegmentChange: (segment: DateSegment) => void;
    onToggleYearPicker: () => void;
    onYearOptionSelect: (value: string) => void;
    onMonthChange: (value: string) => void;
    onDateValueChange: (value: string) => void;
    onRangeStartChange: (value: string) => void;
    onRangeEndChange: (value: string) => void;
    onClose?: () => void;
}

const DatePeriodBlock: React.FC<DatePeriodBlockProps> = ({
    dateSegment, years, yearValue, monthValue, dateValue, rangeStart, rangeEnd, isYearPickerOpen,
    onSegmentChange, onToggleYearPicker, onYearOptionSelect, onMonthChange, onDateValueChange, onRangeStartChange, onRangeEndChange, onClose,
}) => {
    const availableSegments = years.length > 0 ? DATE_SEGMENTS : DATE_SEGMENTS.filter(seg => seg !== '年');
    const visibleDateSegment = years.length === 0 && dateSegment === '年' ? '月' : dateSegment;

    const triggerRef = useRef<HTMLButtonElement>(null);
    const listboxRef = useRef<HTMLDivElement>(null);

    const getListboxOptions = () => {
        if (!listboxRef.current) return [];
        return Array.from(listboxRef.current.querySelectorAll<HTMLButtonElement>('[role="option"]'));
    };

    const handleListboxKeyDown = (e: React.KeyboardEvent) => {
        const options = getListboxOptions();
        if (options.length === 0) return;
        const currentIndex = options.indexOf(document.activeElement as HTMLButtonElement);
        switch (e.key) {
            case 'ArrowDown':
            case 'ArrowRight': {
                e.preventDefault();
                const next = currentIndex >= 0 && currentIndex < options.length - 1 ? options[currentIndex + 1] : options[0];
                next.focus();
                break;
            }
            case 'ArrowUp':
            case 'ArrowLeft': {
                e.preventDefault();
                const prev = currentIndex > 0 ? options[currentIndex - 1] : options[options.length - 1];
                prev.focus();
                break;
            }
            case 'Home':
                e.preventDefault();
                options[0].focus();
                break;
            case 'End':
                e.preventDefault();
                options[options.length - 1].focus();
                break;
            case 'Enter':
            case ' ': {
                e.preventDefault();
                if (currentIndex >= 0) options[currentIndex].click();
                break;
            }
            case 'Escape':
                e.preventDefault();
                onClose?.();
                triggerRef.current?.focus();
                break;
        }
    };

    const handleTriggerKeyDown = (e: React.KeyboardEvent<HTMLButtonElement>) => {
        if (!isYearPickerOpen) return;
        const options = getListboxOptions();
        if (e.key === 'ArrowDown' || e.key === 'ArrowRight') {
            e.preventDefault();
            options[0]?.focus();
        } else if (e.key === 'ArrowUp' || e.key === 'ArrowLeft') {
            e.preventDefault();
            options[options.length - 1]?.focus();
        } else if (e.key === 'Escape') {
            e.preventDefault();
            onClose?.();
        }
    };

    return (
    <div className="space-y-2">
        <div className="text-sm font-bold text-slate-800">期間</div>
        <div className="flex gap-1">
            {availableSegments.map(seg => (
                <button
                    key={seg}
                    type="button"
                    aria-pressed={visibleDateSegment === seg}
                    onClick={() => onSegmentChange(seg)}
                    className={cn(
                        'flex-1 rounded px-2 py-1 text-xs font-semibold transition-colors',
                        visibleDateSegment === seg
                            ? 'bg-slate-950 text-white'
                            : 'bg-slate-100 text-slate-600 hover:bg-slate-200'
                    )}
                >
                    {seg}
                </button>
            ))}
        </div>
        {visibleDateSegment === '年' && (
            <div className="relative">
                <button
                    ref={triggerRef}
                    type="button"
                    aria-label="年を選択"
                    aria-haspopup="listbox"
                    aria-expanded={isYearPickerOpen}
                    onClick={onToggleYearPicker}
                    onKeyDown={handleTriggerKeyDown}
                    className={cn(
                        'w-full flex items-center gap-2 border px-3 py-2 text-sm transition-colors focus:outline-none focus:ring-2 focus:border-amber-600 focus:ring-amber-500/25',
                        isYearPickerOpen ? 'rounded-t-md rounded-b-none' : 'rounded-md',
                        yearValue
                            ? 'border-amber-500 bg-amber-50 text-amber-900 font-semibold'
                            : 'border-slate-300 bg-white text-slate-500 hover:border-slate-400'
                    )}
                >
                    <svg className="w-3.5 h-3.5 shrink-0 text-slate-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                    </svg>
                    <span className="flex-1 text-left">
                        {yearValue ? (years.find(y => y.value === yearValue)?.label ?? yearValue) : '年を選択'}
                    </span>
                    <svg
                        className={cn('w-4 h-4 shrink-0 text-slate-400 transition-transform duration-150', isYearPickerOpen && 'rotate-180')}
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                        aria-hidden="true"
                    >
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
                    </svg>
                </button>
                {isYearPickerOpen && (
                    <div
                        ref={listboxRef}
                        role="listbox"
                        aria-label="年候補"
                        onKeyDown={handleListboxKeyDown}
                        className="absolute z-10 w-full grid grid-cols-3 gap-1 rounded-b-md border border-t-0 border-slate-300 bg-white px-2 pb-2 pt-1.5"
                    >
                        {years.map(year => (
                            <button
                                key={year.value}
                                type="button"
                                role="option"
                                aria-selected={yearValue === year.value}
                                onClick={() => onYearOptionSelect(year.value)}
                                className={
                                    yearValue === year.value
                                        ? 'rounded px-1 py-1.5 text-sm font-semibold text-center whitespace-nowrap bg-amber-50 text-amber-900 ring-1 ring-inset ring-amber-400 transition-colors'
                                        : 'rounded px-1 py-1.5 text-sm text-center whitespace-nowrap text-slate-600 hover:bg-slate-100 hover:text-slate-900 transition-colors'
                                }
                            >
                                {year.label}
                            </button>
                        ))}
                    </div>
                )}
            </div>
        )}
        {visibleDateSegment === '月' && (
            <CalendarDateButton
                label="月を選択"
                value={monthValue}
                inputType="month"
                onChange={onMonthChange}
            />
        )}
        {visibleDateSegment === '日' && (
            <CalendarDateButton
                label="日を選択"
                value={dateValue}
                onChange={onDateValueChange}
            />
        )}
        {visibleDateSegment === '範囲' && (
            <div className="flex flex-col gap-2">
                <CalendarDateButton
                    label="開始日"
                    value={rangeStart}
                    onChange={onRangeStartChange}
                />
                <CalendarDateButton
                    label="終了日"
                    value={rangeEnd}
                    onChange={onRangeEndChange}
                />
            </div>
        )}
    </div>
    );
};

interface SearchCardProps {
    onSearch: (query: string) => void;
    categories?: SearchCategories;
    onExpandToggle?: (isExpanded: boolean) => void; // 展開状態変更の通知
    value?: string; // 親の検索状態と同期（外部クリア対応）
    initialExpanded?: boolean; // 初期展開状態（デフォルト: true）
    compact?: boolean; // コンパクトモード: aside などで lg:grid-cols-4 を抑制する
}

/**
 * 検索カードコンポーネント
 * 複数の検索方法を提供する使いやすいUI
 */
export const SearchCard: React.FC<SearchCardProps> = ({
    onSearch,
    categories,
    onExpandToggle,
    value,
    initialExpanded = true,
    compact = false,
}) => {

    const [isExpanded, setIsExpanded] = useState(initialExpanded);
    // layout 切り替えなどで initialExpanded が変化したときに展開状態を同期する
    useEffect(() => {
        setIsExpanded(initialExpanded);
    }, [initialExpanded]);

    const [selectedQueries, setSelectedQueries] = useState<Record<SearchKey, string>>(
        createEmptySelectedQueries()
    );

    // 日付検索用ステート（5フィールドをひとつのオブジェクトで管理）
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

    // データが存在するかチェック
    const hasData = (data: unknown[] | undefined): boolean =>
        Boolean(data && data.length > 0);

    // 描画対象カテゴリが1つ以上あるか（毎レンダーで再評価されないよう定数化）
    const hasAnyCategories = categories != null && (
        hasData(categories.securities) ||
        hasData(categories.products) ||
        hasData(categories.accounts) ||
        hasData(categories.years) ||
        Boolean(categories.dates)
    );

    // 展開状態の切り替え処理
    const handleToggleExpanded = () => {
        const newExpandedState = !isExpanded;
        setIsExpanded(newExpandedState);
        onExpandToggle?.(newExpandedState);
    };

    const handleToggleKeyDown = (e: React.KeyboardEvent<HTMLButtonElement>) => {
        if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            handleToggleExpanded();
        }
    };

    const resetDateInputs = () => {
        setDateInputs({ ...EMPTY_DATE_INPUTS });
        setIsYearPickerOpen(false);
    };

    const applySelectedQueries = (nextQueries: Record<SearchKey, string>) => {
        setSelectedQueries(nextQueries);
        onSearch(buildCombinedQuery(nextQueries));
    };

    const handleQuickSearch = (value: string, searchType: 'securities' | 'years' | 'products' | 'accounts') => {
        const shouldToggleOff =
            (searchType === 'products' || searchType === 'accounts') &&
            effectiveSelectedQueries[searchType] === value;
        applySelectedQueries({
            ...effectiveSelectedQueries,
            [searchType]: value === '' || shouldToggleOff ? '' : value,
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

    // 検索条件が初期状態かどうか
    const isDefaultState = !hasActiveSearch;

    // 検索条件をクリア
    const handleClearSearch = () => {
        setSelectedQueries(createEmptySelectedQueries());
        setDateSegment(getInitialDateSegment(categories));
        resetDateInputs();
        onSearch('');
    };

    // カテゴリが何もない場合は SearchCard 自体を非表示
    if (!hasAnyCategories) {
        return null;
    }

    const datePeriodBlock = categories?.dates ? (
        <div className="mb-3.5">
            <DatePeriodBlock
                dateSegment={dateSegment}
                years={categories?.years ?? []}
                yearValue={dateInputs.yearValue}
                monthValue={dateInputs.monthValue}
                dateValue={dateInputs.dateValue}
                rangeStart={dateInputs.rangeStart}
                rangeEnd={dateInputs.rangeEnd}
                isYearPickerOpen={isYearPickerOpen}
                onSegmentChange={handleSegmentChange}
                onToggleYearPicker={() => setIsYearPickerOpen(open => !open)}
                onYearOptionSelect={handleYearOptionSelect}
                onMonthChange={handleMonthChange}
                onDateValueChange={handleDateValueChange}
                onRangeStartChange={handleRangeStartChange}
                onRangeEndChange={handleRangeEndChange}
                onClose={() => setIsYearPickerOpen(false)}
            />
        </div>
    ) : null;

    if (compact) {
        return (
            <section className="px-4 py-4" data-testid="search-card-compact">
                <div
                    className="flex items-center justify-between gap-2"
                >
                    <button
                        type="button"
                        className="flex min-w-0 items-center gap-2.5 text-left select-none cursor-pointer"
                        onClick={handleToggleExpanded}
                        onKeyDown={handleToggleKeyDown}
                        aria-expanded={isExpanded}
                        aria-controls="search-options-body"
                        aria-label={`検索オプション ${isExpanded ? '閉じる' : '開く'}`}
                        data-testid="search-card-header"
                    >
                        <span className="inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-md bg-slate-950 text-white">
                            <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z" />
                            </svg>
                        </span>
                        <div className="min-w-0">
                            <p className="text-[10px] font-black uppercase tracking-[0.18em] text-slate-500">Filter</p>
                            <h5 className="whitespace-nowrap text-sm font-black text-slate-950">検索オプション</h5>
                            {!isExpanded && hasActiveSearch && (
                                <span className="mt-1 inline-flex whitespace-nowrap rounded-md border border-amber-200 bg-amber-50 px-2 py-0.5 text-[11px] font-bold text-amber-800">
                                    適用中
                                </span>
                            )}
                        </div>
                    </button>
                    <div className="flex shrink-0 items-center gap-1.5">
                        <Button
                            type="button"
                            variant="outline-secondary"
                            size="sm"
                            onClick={(e: React.MouseEvent) => {
                                e.stopPropagation();
                                handleClearSearch();
                            }}
                            onKeyDown={(e: React.KeyboardEvent) => {
                                if (e.key === 'Enter' || e.key === ' ') {
                                    e.stopPropagation();
                                }
                            }}
                            className={cn('whitespace-nowrap rounded-md border-slate-300 bg-white px-2 py-1 text-xs text-slate-700 hover:bg-slate-50 transition-opacity', isDefaultState && 'opacity-0 pointer-events-none')}
                            aria-label="検索条件をクリア"
                            aria-hidden={isDefaultState}
                            tabIndex={isDefaultState ? -1 : 0}
                            data-testid="search-clear-button"
                        >
                            解除
                        </Button>
                        <span className="flex h-8 w-8 items-center justify-center rounded-md border border-slate-300 bg-white text-slate-700" aria-hidden="true">
                            <svg
                                className={cn('h-4 w-4 text-slate-500 transition-transform duration-200', isExpanded && 'rotate-180')}
                                fill="none"
                                stroke="currentColor"
                                viewBox="0 0 24 24"
                            >
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
                            </svg>
                        </span>
                    </div>
                </div>
                {isExpanded && categories && (
                    <div id="search-options-body" className="pt-4">
                        {datePeriodBlock}
                        <SearchFieldsGrid
                            categories={categories}
                            gridClassName="grid-cols-1 gap-3.5"
                            selectedQueries={effectiveSelectedQueries}
                            onSearch={handleQuickSearch}
                            hasData={hasData}
                        />
                    </div>
                )}
            </section>
        );
    }

    return (
        <Card className="mt-1 overflow-hidden">
            <CardHeader
                variant="secondary"
                className={cn(
                    'flex items-center justify-between transition-colors focus-within:ring-2 focus-within:ring-white/50 focus-within:ring-inset',
                    isExpanded
                        ? 'bg-slate-950 hover:bg-slate-900 border-b border-amber-500'
                        : 'bg-slate-800 hover:bg-slate-900'
                )}
            >
                <button
                    type="button"
                    className="flex items-center gap-2 text-left select-none cursor-pointer"
                    onClick={handleToggleExpanded}
                    onKeyDown={handleToggleKeyDown}
                    aria-expanded={isExpanded}
                    aria-controls="search-options-body"
                    aria-label={`検索オプション ${isExpanded ? '閉じる' : '開く'}`}
                    data-testid="search-card-header"
                >
                    <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z" />
                    </svg>
                    <h5 className="text-sm font-semibold">検索オプション</h5>
                    {!isExpanded && hasActiveSearch && (
                        <span className="text-xs px-2 py-0.5 rounded bg-white/20">
                            フィルタ適用中
                        </span>
                    )}
                </button>
                <div className="flex items-center gap-6">
                    {/* 条件クリアボタン（ヘッダー内・常にレンダリングし高さを固定） */}
                    <Button
                        type="button"
                        variant="outline-danger"
                        size="sm"
                        onClick={(e: React.MouseEvent) => {
                            e.stopPropagation();
                            handleClearSearch();
                        }}
                        onKeyDown={(e: React.KeyboardEvent) => {
                            if (e.key === 'Enter' || e.key === ' ') {
                                e.stopPropagation();
                            }
                        }}
                        className={cn(
                            'text-xs px-2 py-0.5 transition-opacity',
                            isDefaultState
                                ? 'opacity-0 pointer-events-none border-transparent text-transparent'
                                : 'opacity-100 border-white text-white bg-white/20 font-semibold hover:bg-white/30 hover:border-white'
                        )}
                        aria-label="検索条件をクリア"
                        aria-hidden={isDefaultState}
                        tabIndex={isDefaultState ? -1 : 0}
                        data-testid="search-clear-button"
                    >
                        <svg className="w-3.5 h-3.5 inline-block" fill="none" stroke="currentColor" strokeWidth={2.5} viewBox="0 0 24 24" aria-hidden="true">
                            <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                        <span className="ml-1">絞り込み解除</span>
                    </Button>
                    {/* シェブロンアイコン: 回転で開閉状態を表現 */}
                    <span
                        className="flex items-center gap-1.5 rounded-md border border-white/25 bg-white/10 px-2 py-0.5"
                        aria-hidden="true"
                    >
                        <span className="text-xs font-semibold whitespace-nowrap text-white">
                            {isExpanded ? '閉じる' : '開く'}
                        </span>
                        <svg
                            className={cn('w-4 h-4 transition-transform duration-200', isExpanded && 'rotate-180')}
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                        >
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
                        </svg>
                    </span>
                </div>
            </CardHeader>
            {isExpanded && categories && (
                <CardBody id="search-options-body" className="p-3">
                    {datePeriodBlock}
                    <SearchFieldsGrid
                        categories={categories}
                        gridClassName="grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4"
                        selectedQueries={effectiveSelectedQueries}
                        onSearch={handleQuickSearch}
                        hasData={hasData}
                    />
                </CardBody>
            )}
        </Card>
    );
};
