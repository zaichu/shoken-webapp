import React, { useState, useEffect } from 'react';
import { SearchCategories } from '@/types/common';
import { Button } from '@/components/atoms/Button';
import { Card, CardBody, CardHeader } from '@/components/atoms/Card';

interface SearchCardProps {
    onSearch: (query: string) => void;
    categories?: SearchCategories;
    onExpandToggle?: (isExpanded: boolean) => void; // 展開状態変更の通知
    value?: string; // 親の検索状態と同期（外部クリア対応）
}

/**
 * 検索カードコンポーネント
 * 複数の検索方法を提供する使いやすいUI
 */
export const SearchCard: React.FC<SearchCardProps> = ({
    onSearch,
    categories,
    onExpandToggle,
    value
}) => {

    const [isExpanded, setIsExpanded] = useState(true);
    const [searchQuery, setSearchQuery] = useState('');
    // アクティブな検索タイプを追跡（ドロップダウンの表示制御用）
    const [activeSearchType, setActiveSearchType] = useState<'securities' | 'years' | 'products' | 'accounts' | null>(null);

    // 親の検索状態と同期（外部からのクリア時に内部状態をリセット）
    useEffect(() => {
        if (value !== undefined && value !== searchQuery) {
            setSearchQuery(value);
            setActiveSearchType(value ? activeSearchType : null);
        }
    }, [value]); // eslint-disable-line react-hooks/exhaustive-deps

    // データが存在するかチェックするヘルパー関数
    const hasData = (data: unknown[] | undefined): boolean => {
        return Boolean(data && data.length > 0);
    };

    // 全ての検索カテゴリが空かチェック（UIで描画するカテゴリのみ判定）
    const hasAnyCategories = () => {
        if (!categories) return false;
        return hasData(categories.securities) ||
            hasData(categories.products) ||
            hasData(categories.accounts) ||
            hasData(categories.years);
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

    // 検索条件が初期状態かどうか
    const isDefaultState = searchQuery === '' && activeSearchType === null;

    // 検索条件をクリア
    const handleClearSearch = () => {
        setSearchQuery('');
        setActiveSearchType(null);
        onSearch('');
    };

    // カテゴリが何もない場合は SearchCard 自体を非表示
    if (!hasAnyCategories()) {
        return null;
    }

    const renderQuickSearchButtons = (items: string[], searchType: 'products' | 'accounts') => {
        if (!items || items.length === 0) return null;

        // いずれかが選択中かどうか（未選択チップをミュートするため）
        const hasSelection = activeSearchType === searchType;

        return items.map((item, index) => {
            const isSelected = activeSearchType === searchType && searchQuery === item;

            return (
                <React.Fragment key={index}>
                    <Button
                        type="button"
                        variant={isSelected ? 'primary' : 'outline-secondary'}
                        size="sm"
                        onClick={() => handleQuickSearch(item, searchType)}
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
        });
    };

    const renderQuickSearchDropdown = (items: { value: string, label: string }[], id: string, searchType: 'securities' | 'years') => {
        // このドロップダウンがアクティブな検索タイプの場合のみ値を表示
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
                {/* 選択時はチェックアイコンを表示 */}
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
                    {!isExpanded && activeSearchType && (
                        <span className="text-xs bg-white/20 px-2 py-0.5 rounded">
                            フィルタ適用中
                        </span>
                    )}
                </div>
                <div className="flex items-center gap-2">
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
                                : 'opacity-100 border-white/60 text-white bg-white/10 hover:bg-white/25 hover:border-white/80'
                        }`}
                        aria-label="検索条件をクリア"
                        aria-hidden={isDefaultState}
                        tabIndex={isDefaultState ? -1 : 0}
                        data-testid="search-clear-button"
                    >
                        <svg className="w-3 h-3 mr-1 inline-block" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                        </svg>
                        条件をクリア
                    </Button>
                    {/* シェブロンアイコン: 回転で開閉状態を表現 */}
                    <span className="flex items-center gap-1.5" aria-hidden="true">
                        <span className="text-xs font-medium opacity-80">
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
                                <div className="flex flex-wrap gap-1">{renderQuickSearchButtons(categories.products!, 'products')}</div>
                            </div>
                        )}

                        {/* 口座検索 */}
                        {hasData(categories.accounts) && (
                            <div className="space-y-1">
                                <div className="text-sm font-medium text-dark">口座</div>
                                <div className="flex flex-wrap gap-1">{renderQuickSearchButtons(categories.accounts!, 'accounts')}</div>
                            </div>
                        )}
                    </div>
                </CardBody>
            )}
        </Card>
    );
};
