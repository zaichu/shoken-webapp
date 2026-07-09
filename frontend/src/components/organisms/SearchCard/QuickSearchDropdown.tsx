import React from 'react';
import { cn } from '@/lib/utils/classNames';
import { ActiveSearchType } from './types';

interface QuickSearchDropdownProps {
    items: { value: string; label: string }[];
    id: string;
    searchType: 'securities' | 'years';
    activeSearchType: ActiveSearchType;
    searchQuery: string;
    onSearch: (value: string, searchType: 'securities' | 'years') => void;
}

export const QuickSearchDropdown: React.FC<QuickSearchDropdownProps> = ({
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
