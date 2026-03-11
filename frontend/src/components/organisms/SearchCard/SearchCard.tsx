import React, { useState } from 'react';
import { SearchCategories } from '@/types/common';
import { Button } from '@/components/atoms/Button';
import { Card, CardBody, CardHeader } from '@/components/atoms/Card';

interface QuickSearchDropdownProps {
    items: { value: string; label: string }[];
    id: string;
    searchType: 'securities' | 'years';
    activeSearchType: 'securities' | 'years' | 'products' | 'accounts' | null;
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
                className={`w-full rounded-md border px-3 py-2 text-sm transition-colors focus:outline-none focus:ring-2 ${
                    isSelected
                        ? 'border-primary bg-primary/20 text-blue-800 font-bold ring-2 ring-primary/50'
                        : 'border-gray-300 bg-white text-dark hover:border-slate-400 focus:border-primary focus:ring-primary/25'
                }`}
                value={displayValue}
                onChange={(e) => onSearch(e.target.value, searchType)}
                aria-label="検索フィルター"
            >
                <option value="">全て表示</option>
                {items.map((option) => (
                    <option key={option.value} value={option.value}>
                        {option.label}
                    </option>
                ))}
            </select>
            {isSelected && (
                <span className="absolute right-8 top-1/2 -translate-y-1/2 text-blue-700 pointer-events-none">
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
    activeSearchType: 'securities' | 'years' | 'products' | 'accounts' | null;
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
                                ? 'ring-2 ring-primary ring-offset-1 font-bold shadow-md'
                                : `${hasSelection ? 'opacity-50' : 'opacity-80'} hover:opacity-100 hover:bg-slate-100 hover:ring-1 hover:ring-slate-300 focus-visible:ring-2 focus-visible:ring-primary/50 focus-visible:outline-none`
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
    activeSearchType: 'securities' | 'years' | 'products' | 'accounts' | null;
    searchQuery: string;
    onSearch: (value: string, searchType: 'securities' | 'years' | 'products' | 'accounts') => void;
    hasData: (data: unknown[] | undefined) => boolean;
}

const SearchFieldsGrid: React.FC<SearchFieldsGridProps> = ({
    categories, gridClassName, activeSearchType, searchQuery, onSearch, hasData,
}) => (
    <div className={`grid ${gridClassName}`}>
        {DROPDOWN_CONFIGS.map(({ key, label }) =>
            hasData(categories[key]) && (
                <div key={key} className="space-y-1">
                    <label htmlFor={`${key}-search`} className="text-sm font-medium text-dark">{label}</label>
                    <QuickSearchDropdown
                        items={categories[key]!}
                        id={`${key}-search`}
                        searchType={key}
                        activeSearchType={activeSearchType}
                        searchQuery={searchQuery}
                        onSearch={onSearch}
                    />
                </div>
            )
        )}
        {BUTTONS_CONFIGS.map(({ key, label }) =>
            hasData(categories[key]) && (
                <div key={key} className="space-y-1">
                    <div className="text-sm font-medium text-dark">{label}</div>
                    <div className="flex flex-wrap gap-1">
                        <QuickSearchButtons
                            items={categories[key]!}
                            searchType={key}
                            activeSearchType={activeSearchType}
                            searchQuery={searchQuery}
                            onSearch={onSearch}
                        />
                    </div>
                </div>
            )
        )}
    </div>
);

interface SearchCardProps {
    onSearch: (query: string) => void;
    categories?: SearchCategories;
    onExpandToggle?: (isExpanded: boolean) => void; // 展開状態変更の通知
    value?: string; // 親の検索状態と同期（外部クリア対応）
    defaultExpanded?: boolean; // 初期展開状態（デフォルト: true）
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
    defaultExpanded = true,
    compact = false,
}) => {

    const [isExpanded, setIsExpanded] = useState(defaultExpanded);
    const [searchQuery, setSearchQuery] = useState('');
    // アクティブな検索タイプを追跡（ドロップダウンの表示制御用）
    const [activeSearchType, setActiveSearchType] = useState<'securities' | 'years' | 'products' | 'accounts' | null>(null);
    const effectiveSearchQuery = value ?? searchQuery;
    const effectiveActiveSearchType = value === '' ? null : activeSearchType;

    // データが存在するかチェック
    const hasData = (data: unknown[] | undefined): boolean =>
        Boolean(data && data.length > 0);

    // 描画対象カテゴリが1つ以上あるか（毎レンダーで再評価されないよう定数化）
    const hasAnyCategories = categories != null && (
        hasData(categories.securities) ||
        hasData(categories.products) ||
        hasData(categories.accounts) ||
        hasData(categories.years)
    );

    // 展開状態の切り替え処理
    const handleToggleExpanded = () => {
        const newExpandedState = !isExpanded;
        setIsExpanded(newExpandedState);
        onExpandToggle?.(newExpandedState);
    };

    // キーボードイベントハンドラ
    const handleKeyDown = (e: React.KeyboardEvent) => {
        if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            handleToggleExpanded();
        }
    };

    const handleQuickSearch = (value: string, searchType: 'securities' | 'years' | 'products' | 'accounts') => {
        setSearchQuery(value);
        setActiveSearchType(value ? searchType : null);
        onSearch(value);
    };

    // 検索条件が初期状態かどうか
    const isDefaultState = effectiveSearchQuery === '' && effectiveActiveSearchType === null;

    // 検索条件をクリア
    const handleClearSearch = () => {
        setSearchQuery('');
        setActiveSearchType(null);
        onSearch('');
    };

    // カテゴリが何もない場合は SearchCard 自体を非表示
    if (!hasAnyCategories) {
        return null;
    }

    if (compact) {
        return (
            <section className="px-5 py-5" data-testid="search-card-compact">
                <div
                    className="flex cursor-pointer items-start justify-between gap-3 select-none"
                    onClick={handleToggleExpanded}
                    onKeyDown={handleKeyDown}
                    role="button"
                    tabIndex={0}
                    aria-expanded={isExpanded}
                    aria-controls="search-options-body"
                    aria-label={`検索オプション ${isExpanded ? '閉じる' : '開く'}`}
                    data-testid="search-card-header"
                >
                    <div className="flex min-w-0 items-start gap-3">
                        <span className="mt-0.5 inline-flex h-9 w-9 shrink-0 items-center justify-center rounded-2xl bg-slate-100 text-slate-600">
                            <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z" />
                            </svg>
                        </span>
                        <div className="min-w-0">
                            <p className="text-[11px] font-semibold uppercase tracking-[0.18em] text-slate-400">Filter</p>
                            <h5 className="mt-1 text-sm font-semibold text-slate-800">検索オプション</h5>
                            {!isExpanded && effectiveActiveSearchType && (
                                <span className="mt-2 inline-flex rounded-full border border-blue-100 bg-blue-50 px-2.5 py-1 text-xs font-semibold text-blue-700">
                                    フィルタ適用中
                                </span>
                            )}
                        </div>
                    </div>
                    <div className="flex shrink-0 items-center gap-2">
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
                            className={`rounded-full px-3 py-1 text-xs transition-opacity ${
                                isDefaultState
                                    ? 'pointer-events-none opacity-0'
                                    : 'opacity-100 border-slate-300 bg-white text-slate-700 hover:bg-slate-50'
                            }`}
                            aria-label="検索条件をクリア"
                            aria-hidden={isDefaultState}
                            tabIndex={isDefaultState ? -1 : 0}
                            data-testid="search-clear-button"
                        >
                            リセット
                        </Button>
                        <span className="flex items-center gap-1.5 rounded-full border border-slate-200 bg-white px-2.5 py-1 text-slate-700" aria-hidden="true">
                            <span className="text-xs font-semibold">{isExpanded ? '閉じる' : '開く'}</span>
                            <svg
                                className={`h-4 w-4 text-slate-500 transition-transform duration-200 ${isExpanded ? 'rotate-180' : ''}`}
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
                        <SearchFieldsGrid
                            categories={categories}
                            gridClassName="grid-cols-1 gap-3.5"
                            activeSearchType={effectiveActiveSearchType}
                            searchQuery={effectiveSearchQuery}
                            onSearch={handleQuickSearch}
                            hasData={hasData}
                        />
                    </div>
                )}
            </section>
        );
    }

    return (
        <Card className="mt-1">
            <CardHeader
                variant="secondary"
                className={`flex cursor-pointer items-center justify-between select-none transition-colors focus-within:ring-2 focus-within:ring-white/50 focus-within:ring-inset ${
                    isExpanded
                        ? 'bg-slate-600 hover:bg-slate-700 border-b-2 border-slate-700'
                        : 'bg-slate-400 hover:bg-slate-500'
                }`}
                onClick={handleToggleExpanded}
                onKeyDown={handleKeyDown}
                role="button"
                tabIndex={0}
                aria-expanded={isExpanded}
                aria-controls="search-options-body"
                aria-label={`検索オプション ${isExpanded ? '閉じる' : '開く'}`}
                data-testid="search-card-header"
            >
                <div className="flex items-center gap-2">
                    <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z" />
                    </svg>
                    <h5 className="text-sm font-semibold">検索オプション</h5>
                    {!isExpanded && effectiveActiveSearchType && (
                        <span className="text-xs px-2 py-0.5 rounded bg-white/20">
                            フィルタ適用中
                        </span>
                    )}
                </div>
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
                        className={`text-xs px-2 py-0.5 transition-opacity ${
                            isDefaultState
                                ? 'opacity-0 pointer-events-none border-transparent text-transparent'
                                : 'opacity-100 border-white text-white bg-white/20 font-semibold hover:bg-white/30 hover:border-white'
                        }`}
                        aria-label="検索条件をクリア"
                        aria-hidden={isDefaultState}
                        tabIndex={isDefaultState ? -1 : 0}
                        data-testid="search-clear-button"
                    >
                        <svg className="w-3.5 h-3.5 inline-block" fill="none" stroke="currentColor" strokeWidth={2.5} viewBox="0 0 24 24" aria-hidden="true">
                            <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                        {!compact && <span className="ml-1">絞り込み解除</span>}
                    </Button>
                    {/* シェブロンアイコン: 回転で開閉状態を表現 */}
                    <span
                        className="flex items-center gap-1.5 rounded border border-white/30 bg-white/10 px-2 py-0.5"
                        aria-hidden="true"
                    >
                        <span className="text-xs font-semibold whitespace-nowrap text-white">
                            {isExpanded ? '閉じる' : '開く'}
                        </span>
                        <svg
                            className={`w-4 h-4 transition-transform duration-200 ${isExpanded ? 'rotate-180' : ''}`}
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
                    <SearchFieldsGrid
                        categories={categories}
                        gridClassName="grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4"
                        activeSearchType={effectiveActiveSearchType}
                        searchQuery={effectiveSearchQuery}
                        onSearch={handleQuickSearch}
                        hasData={hasData}
                    />
                </CardBody>
            )}
        </Card>
    );
};
