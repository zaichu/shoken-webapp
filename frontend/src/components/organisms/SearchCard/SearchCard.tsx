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

    // 展開状態の切り替え処理
    const handleToggleExpanded = useCallback(() => {
        const newExpandedState = !isExpanded;
        setIsExpanded(newExpandedState);
        onExpandToggle?.(newExpandedState);
    }, [isExpanded, onExpandToggle]);

    // const handleClear = useCallback(() => {
    //     setLocalQuery('');
    //     onSearch('');
    // }, [onSearch]);

    const handleQuickSearch = useCallback((value: string) => {
        setSearchQuery(value);
        onSearch(value);
    }, [onSearch]);

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

    const renderQuickSearchDropdown = (items: { value: string, label: string }[]) => {
        if (!items || items.length === 0) return null;
        return (
            <select
                id="security-search"
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
            <div className="card-header bg-primary text-white d-flex justify-content-between align-items-center" onClick={handleToggleExpanded}>
                <h5 className="mb-0 text-white">検索オプション</h5>
                <button type="button" className="btn btn-sm mb-0 text-white">
                    {isExpanded ? '折りたたむ' : '展開'}
                </button>
            </div>
            {isExpanded && categories && (
                <div className="card-body">
                    {/* 銘柄検索 */}
                    {categories.securities && categories.securities.length > 0 && (
                        <div style={{ width: '500px' }}>
                            <div className="d-flex align-items-center">銘柄</div>
                            <div>{renderQuickSearchDropdown(categories.securities)}</div>
                        </div>
                    )}

                    <div className="d-flex mt-3">
                        {/* 年度検索 */}
                        {categories.years && categories.years.length > 0 && (
                            <div>
                                <div className="d-flex align-items-center" style={{ width: '200px' }}>年度</div>
                                <div>{renderQuickSearchDropdown(categories.years)}</div>
                            </div>
                        )}

                        {/* 年月検索 */}
                        {/* {categories.yearMonths && categories.yearMonths.length > 0 && (
                            <div>
                                <div className="d-flex align-items-center mb-2" style={{ width: '200px' }}>年月</div>
                                <div>{renderQuickSearchDropdown(categories.yearMonths)}</div>
                            </div>
                        )} */}
                    </div>

                    <div className="d-flex mt-3">
                        {/* 商品検索 */}
                        {categories.products && categories.products.length > 0 && (
                            <div>
                                <div className="d-flex align-items-center">商品</div>
                                <div>{renderQuickSearchButtons(categories.products, "outline-success")}</div>
                            </div>
                        )}

                        {/* 口座検索 */}
                        {categories.accounts && categories.accounts.length > 0 && (
                            <div>
                                <div className="d-flex align-items-center">口座</div>
                                <div>{renderQuickSearchButtons(categories.accounts, "outline-warning")}</div>
                            </div>
                        )}
                    </div>
                </div>
            )}
        </div>
    );
};
