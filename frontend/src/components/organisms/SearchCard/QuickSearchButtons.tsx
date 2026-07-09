import React from 'react';
import { Button } from '@/components/atoms/Button';
import { ActiveSearchType } from './types';

interface QuickSearchButtonsProps {
    items: string[];
    searchType: 'products' | 'accounts';
    activeSearchType: ActiveSearchType;
    searchQuery: string;
    onSearch: (value: string, searchType: 'products' | 'accounts') => void;
}

export const QuickSearchButtons: React.FC<QuickSearchButtonsProps> = ({
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
