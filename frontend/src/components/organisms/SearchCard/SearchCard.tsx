import React, { useState } from 'react';
import { SearchCategories } from '@/types/common';
import { Button, ButtonVariant } from '@/components/atoms/Button';
import { Card, CardBody, CardHeader } from '@/components/atoms/Card';

interface SearchCardProps {
    onSearch: (query: string) => void;
    categories?: SearchCategories;
    onExpandToggle?: (isExpanded: boolean) => void; // 展開状態変更の通知
}

/**
 * 検索カードコンポーネント
 * 複数の検索方法を提供する使いやすいUI
 */
export const SearchCard: React.FC<SearchCardProps> = ({
    onSearch,
    categories,
    onExpandToggle
}) => {

    const [isExpanded, setIsExpanded] = useState(false);
    const [searchQuery, setSearchQuery] = useState('');

    // データが存在するかチェックするヘルパー関数
    const hasData = (data: unknown[] | undefined): boolean => {
        return Boolean(data && data.length > 0);
    };

    // 全ての検索カテゴリが空かチェック
    const hasAnyCategories = () => {
        if (!categories) return false;
        return hasData(categories.securities) ||
            hasData(categories.products) ||
            hasData(categories.accounts) ||
            hasData(categories.years) ||
            hasData(categories.yearMonths);
    };

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

    const handleQuickSearch = (value: string) => {
        setSearchQuery(value);
        onSearch(value);
    };

    // カテゴリが何もない場合は SearchCard 自体を非表示
    if (!hasAnyCategories()) {
        return null;
    }

    const renderQuickSearchButtons = (items: string[], variant: string) => {
        if (!items || items.length === 0) return null;

        return items.map((item, index) => (
            <React.Fragment key={index}>
                <Button type="button" variant={variant as ButtonVariant} size="sm" onClick={() => handleQuickSearch(item)}>
                    {item}
                </Button>
                {index % 10 === 9 && <div className="mt-1" />}
            </React.Fragment>
        ));
    };

    const renderQuickSearchDropdown = (items: { value: string, label: string }[], id: string = 'search-dropdown') => {
        return (
            <select
                id={id}
                className="w-full rounded-md border border-gray-300 bg-white px-3 py-2 text-sm text-dark focus:border-primary focus:outline-none focus:ring-2 focus:ring-primary/25"
                value={searchQuery}
                onChange={(e) => handleQuickSearch(e.target.value)}
                aria-label="検索フィルター"
            >
                <option value="">全て表示</option>
                {items.map((option, index) => (
                    <option key={`search-option-${option.value}-${index}`} value={option.value}>
                        {option.label}
                    </option>
                ))}
            </select>
        );
    };

    return (
        <Card className="mt-1">
            <CardHeader
                variant="primary"
                className="flex cursor-pointer items-center justify-between"
                onClick={handleToggleExpanded}
                onKeyDown={handleKeyDown}
                role="button"
                tabIndex={0}
                aria-expanded={isExpanded}
                aria-controls="search-options-body"
                data-testid="search-card-header"
            >
                <h5>検索オプション</h5>
                <div
                    className={`h-0 w-0 border-l-[6px] border-r-[6px] border-t-[8px] border-l-transparent border-r-transparent border-t-white transition-transform ${isExpanded ? 'rotate-180' : ''}`}
                />
            </CardHeader>
            {isExpanded && categories && (
                <CardBody id="search-options-body" className="space-y-4 p-3">
                    {/* 銘柄検索 */}
                    {hasData(categories.securities) && (
                        <div className="w-full max-w-[500px] space-y-2">
                            <div className="text-sm font-medium text-dark">銘柄</div>
                            <div>{renderQuickSearchDropdown(categories.securities!, 'securities-search')}</div>
                        </div>
                    )}

                    <div className="flex flex-wrap gap-6">
                        {/* 年度検索 */}
                        {hasData(categories.years) && (
                            <div>
                                <div className="mb-2 w-[200px] text-sm font-medium text-dark">西暦</div>
                                <div>{renderQuickSearchDropdown(categories.years!, 'years-search')}</div>
                            </div>
                        )}
                        {/* 年月検索 */}
                        {/* 年月検索は現状非表示 */}
                    </div>

                    <div className="flex flex-wrap gap-6">
                        {/* 商品検索 */}
                        {hasData(categories.products) && (
                            <div className="space-y-2">
                                <div className="text-sm font-medium text-dark">商品</div>
                                <div className="flex flex-wrap gap-2">{renderQuickSearchButtons(categories.products!, "outline-success")}</div>
                            </div>
                        )}
                        {/* 口座検索 */}
                        {hasData(categories.accounts) && (
                            <div className="space-y-2">
                                <div className="text-sm font-medium text-dark">口座</div>
                                <div className="flex flex-wrap gap-2">{renderQuickSearchButtons(categories.accounts!, "outline-warning")}</div>
                            </div>
                        )}
                    </div>
                </CardBody>
            )}
        </Card>
    );
};
