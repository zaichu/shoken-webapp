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

    const [isExpanded, setIsExpanded] = useState(true);
    const [searchQuery, setSearchQuery] = useState('');
    // アクティブな検索タイプを追跡（ドロップダウンの表示制御用）
    const [activeSearchType, setActiveSearchType] = useState<'securities' | 'years' | 'products' | 'accounts' | null>(null);

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

    const handleQuickSearch = (value: string, searchType: 'securities' | 'years' | 'products' | 'accounts') => {
        setSearchQuery(value);
        setActiveSearchType(value ? searchType : null);
        onSearch(value);
    };

    // カテゴリが何もない場合は SearchCard 自体を非表示
    if (!hasAnyCategories()) {
        return null;
    }

    const renderQuickSearchButtons = (items: string[], variant: string, searchType: 'products' | 'accounts') => {
        if (!items || items.length === 0) return null;

        return items.map((item, index) => {
            // 選択中のボタンを判定
            const isSelected = activeSearchType === searchType && searchQuery === item;
            // 選択中はprimaryバリアント、そうでなければ元のvariant
            const buttonVariant = isSelected ? 'primary' : variant;

            return (
                <React.Fragment key={index}>
                    <Button
                        type="button"
                        variant={buttonVariant as ButtonVariant}
                        size="sm"
                        onClick={() => handleQuickSearch(item, searchType)}
                        className={isSelected ? 'ring-2 ring-primary ring-offset-1' : ''}
                    >
                        {item}
                    </Button>
                    {index % 10 === 9 && <div className="mt-1" />}
                </React.Fragment>
            );
        });
    };

    const renderQuickSearchDropdown = (items: { value: string, label: string }[], id: string, searchType: 'securities' | 'years') => {
        // このドロップダウンがアクティブな検索タイプの場合のみ値を表示
        const displayValue = activeSearchType === searchType ? searchQuery : '';
        const isSelected = activeSearchType === searchType && displayValue !== '';

        return (
            <select
                id={id}
                className={`w-full rounded-md border px-3 py-2 text-sm focus:outline-none focus:ring-2 ${
                    isSelected
                        ? 'border-primary bg-primary/5 text-primary font-medium ring-2 ring-primary/25'
                        : 'border-gray-300 bg-white text-dark focus:border-primary focus:ring-primary/25'
                }`}
                value={displayValue}
                onChange={(e) => handleQuickSearch(e.target.value, searchType)}
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
                className="flex cursor-pointer items-center justify-between select-none hover:bg-primary/90 transition-colors"
                onClick={handleToggleExpanded}
                onKeyDown={handleKeyDown}
                role="button"
                tabIndex={0}
                aria-expanded={isExpanded}
                aria-controls="search-options-body"
                data-testid="search-card-header"
            >
                <div className="flex items-center gap-2">
                    <h5 className="text-sm font-medium">検索オプション</h5>
                    {!isExpanded && activeSearchType && (
                        <span className="text-xs bg-white/20 px-2 py-0.5 rounded">
                            フィルタ適用中
                        </span>
                    )}
                </div>
                <div className="flex items-center gap-1">
                    <span className="text-xs opacity-75 hidden sm:inline">
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
                </div>
            </CardHeader>
            {isExpanded && categories && (
                <CardBody id="search-options-body" className="p-3">
                    {/* グリッドレイアウト: モバイル1列、sm2列、lg4列 */}
                    <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
                        {/* 銘柄検索 */}
                        {hasData(categories.securities) && (
                            <div className="space-y-1">
                                <label htmlFor="securities-search" className="text-sm font-medium text-dark">銘柄</label>
                                {renderQuickSearchDropdown(categories.securities!, 'securities-search', 'securities')}
                            </div>
                        )}

                        {/* 年度検索 */}
                        {hasData(categories.years) && (
                            <div className="space-y-1">
                                <label htmlFor="years-search" className="text-sm font-medium text-dark">西暦</label>
                                {renderQuickSearchDropdown(categories.years!, 'years-search', 'years')}
                            </div>
                        )}

                        {/* 商品検索 */}
                        {hasData(categories.products) && (
                            <div className="space-y-1">
                                <div className="text-sm font-medium text-dark">商品</div>
                                <div className="flex flex-wrap gap-1">{renderQuickSearchButtons(categories.products!, "outline-success", 'products')}</div>
                            </div>
                        )}

                        {/* 口座検索 */}
                        {hasData(categories.accounts) && (
                            <div className="space-y-1">
                                <div className="text-sm font-medium text-dark">口座</div>
                                <div className="flex flex-wrap gap-1">{renderQuickSearchButtons(categories.accounts!, "outline-warning", 'accounts')}</div>
                            </div>
                        )}
                    </div>
                </CardBody>
            )}
        </Card>
    );
};
