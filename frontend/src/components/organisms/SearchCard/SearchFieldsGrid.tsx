import React from 'react';
import { cn } from '@/lib/utils/classNames';
import { SearchCategories } from '@/types/common';
import { SearchKey } from './types';
import { hasData } from './searchQueryUtils';
import { QuickSearchDropdown } from './QuickSearchDropdown';
import { QuickSearchButtons } from './QuickSearchButtons';

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
}

export const SearchFieldsGrid: React.FC<SearchFieldsGridProps> = ({
    categories, gridClassName, selectedQueries, onSearch,
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
