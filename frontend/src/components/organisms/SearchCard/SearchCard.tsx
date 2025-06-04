import React, { useState, useCallback } from 'react';
import { SearchCategories } from '@/types/common';

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
    const hasAnyCategories = useCallback(() => {
        if (!categories) return false;
        return hasData(categories.securities) ||
            hasData(categories.products) ||
            hasData(categories.accounts) ||
            hasData(categories.years) ||
            hasData(categories.yearMonths);
    }, [categories]);

    // 展開状態の切り替え処理
    const handleToggleExpanded = useCallback(() => {
        const newExpandedState = !isExpanded;
        setIsExpanded(newExpandedState);
        onExpandToggle?.(newExpandedState);
    }, [isExpanded, onExpandToggle]);

    const handleQuickSearch = useCallback((value: string) => {
        setSearchQuery(value);
        onSearch(value);
    }, [onSearch]);

    // カテゴリが何もない場合は SearchCard 自体を非表示
    if (!hasAnyCategories()) {
        return null;
    }

    const renderQuickSearchButtons = (items: string[], variant: string) => {
        if (!items || items.length === 0) return null;

        return items.map((item, index) => (
            <React.Fragment key={index}>
                <button type="button" className={`btn btn-${variant} me-2`} onClick={() => handleQuickSearch(item)}>
                    {item}
                </button>
                {index % 10 === 9 && <div className="mt-1"></div>}
            </React.Fragment>
        ));
    };

    const renderQuickSearchDropdown = (items: { value: string, label: string }[], id: string = 'search-dropdown') => {
        return (
            <select
                id={id}
                className="form-select form-select-sm"
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
        <div className="card shadow-sm mt-1">
            <div className="card-header bg-primary text-white d-flex justify-content-between align-items-center" style={{ cursor: 'pointer' }} onClick={handleToggleExpanded}>
                <h5 className="mb-0 text-white">検索オプション</h5>
                <div style={{
                    width: '0',
                    height: '0',
                    borderLeft: '6px solid transparent',
                    borderRight: '6px solid transparent',
                    borderTop: isExpanded ? 'none' : '8px solid white',
                    borderBottom: isExpanded ? '8px solid white' : 'none',
                    transition: 'all 0.3s ease'
                }}></div>
            </div>
            {isExpanded && categories && (
                <div className="card-body">
                    {/* 銘柄検索 */}
                    {hasData(categories.securities) && (
                        <div style={{ width: '500px' }}>
                            <div className="d-flex align-items-center">銘柄</div>
                            <div>{renderQuickSearchDropdown(categories.securities!, 'securities-search')}</div>
                        </div>
                    )}

                    <div className="d-flex mt-3">
                        {/* 年度検索 */}
                        {hasData(categories.years) && (
                            <div>
                                <div className="d-flex align-items-center" style={{ width: '200px' }}>年度</div>
                                <div>{renderQuickSearchDropdown(categories.years!, 'years-search')}</div>
                            </div>
                        )}
                        <div className="me-2" />
                        {/* 年月検索 */}
                        {/* {hasData(categories.yearMonths) && (
                            <div>
                                <div className="d-flex align-items-center" style={{ width: '200px' }}>年月</div>
                                <div>{renderQuickSearchDropdown(categories.yearMonths!, 'year-months-search')}</div>
                            </div>
                        )} */}
                    </div>

                    <div className="d-flex mt-3">
                        {/* 商品検索 */}
                        {hasData(categories.products) && (
                            <div>
                                <div className="d-flex align-items-center">商品</div>
                                <div>{renderQuickSearchButtons(categories.products!, "outline-success")}</div>
                            </div>
                        )}
                        <div className="me-2" />
                        {/* 口座検索 */}
                        {hasData(categories.accounts) && (
                            <div>
                                <div className="d-flex align-items-center">口座</div>
                                <div>{renderQuickSearchButtons(categories.accounts!, "outline-warning")}</div>
                            </div>
                        )}
                    </div>
                </div>
            )}
        </div>
    );
};
