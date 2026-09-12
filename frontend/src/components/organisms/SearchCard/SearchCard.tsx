import React, { useState } from 'react';
import { Button } from '@/components/atoms/Button';
import { Card, CardBody, CardHeader } from '@/components/atoms/Card';
import { cn } from '@/lib/utils/classNames';
import { SearchCardProps } from './types';
import { useSearchCardQuery } from './useSearchCardQuery';
import { hasData } from './searchQueryUtils';
import { SearchFieldsGrid } from './SearchFieldsGrid';
import { DatePeriodBlock } from './DatePeriodBlock';

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
    // 展開/折りたたみの UI 表示 state（API クエリには影響しない）。
    // initialExpanded はあくまで初期値。ユーザー操作後は上書きしない
    // （Issue #854: スマホでは initialExpanded=false が渡され続けるため、
    // 同期 effect があるとユーザー操作が無効化される。layout はマウント中不変で
    // タブ切替は再マウントになるため、同期の必要はない）。
    const [isExpanded, setIsExpanded] = useState(initialExpanded);

    // 年ピッカーの開閉も検索クエリに影響しない UI 表示 state
    const [isYearPickerOpen, setIsYearPickerOpen] = useState(false);
    const closeYearPicker = () => setIsYearPickerOpen(false);

    // 検索条件（API クエリ params）の state はフックに分離
    const {
        effectiveSelectedQueries,
        hasActiveSearch,
        isDefaultState,
        dateSegment,
        dateInputs,
        handleQuickSearch,
        handleSegmentChange,
        handleYearOptionSelect,
        handleMonthChange,
        handleDateValueChange,
        handleRangeStartChange,
        handleRangeEndChange,
        handleClearSearch,
    } = useSearchCardQuery({ categories, value, onSearch, onCloseYearPicker: closeYearPicker });

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

    // カテゴリが何もない場合は SearchCard 自体を非表示
    if (!hasAnyCategories) {
        return null;
    }

    const datePeriodBlock = categories?.dates ? (
        <div className="mb-3.5">
            <DatePeriodBlock
                dateSegment={dateSegment}
                years={categories?.years ?? []}
                dateInputs={dateInputs}
                isYearPickerOpen={isYearPickerOpen}
                onSegmentChange={handleSegmentChange}
                onToggleYearPicker={() => setIsYearPickerOpen(open => !open)}
                onYearOptionSelect={handleYearOptionSelect}
                onMonthChange={handleMonthChange}
                onDateValueChange={handleDateValueChange}
                onRangeStartChange={handleRangeStartChange}
                onRangeEndChange={handleRangeEndChange}
                onClose={closeYearPicker}
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
                        className="flex min-w-0 items-center gap-2.5 text-left select-none cursor-pointer max-sm:min-h-[44px]"
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
                            className={cn('whitespace-nowrap rounded-md border-slate-300 bg-white px-2 py-1 text-xs text-slate-700 hover:bg-slate-50 transition-opacity max-sm:min-h-[44px]', isDefaultState && 'opacity-0 pointer-events-none')}
                            aria-label="検索条件をクリア"
                            aria-hidden={isDefaultState}
                            tabIndex={isDefaultState ? -1 : 0}
                            data-testid="search-clear-button"
                        >
                            解除
                        </Button>
                        <button
                            type="button"
                            className="flex h-8 w-8 cursor-pointer items-center justify-center rounded-md border border-slate-300 bg-white text-slate-700"
                            onClick={handleToggleExpanded}
                            aria-expanded={isExpanded}
                            aria-controls="search-options-body"
                            aria-label={`検索オプション ${isExpanded ? '閉じる' : '開く'}`}
                            data-testid="search-card-chevron-toggle"
                        >
                            <svg
                                className={cn('h-4 w-4 text-slate-500 transition-transform duration-200', isExpanded && 'rotate-180')}
                                fill="none"
                                stroke="currentColor"
                                viewBox="0 0 24 24"
                                aria-hidden="true"
                            >
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
                            </svg>
                        </button>
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
                    className="flex items-center gap-2 text-left select-none cursor-pointer max-sm:min-h-[44px]"
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
                            'text-xs px-2 py-0.5 transition-opacity max-sm:min-h-[44px]',
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
                    {/* シェブロンアイコン: 回転で開閉状態を表現。ヘッダーとは別のタップ位置としても操作できる */}
                    <button
                        type="button"
                        className="flex cursor-pointer items-center gap-1.5 rounded-md border border-white/25 bg-white/10 px-2 py-0.5"
                        onClick={handleToggleExpanded}
                        aria-expanded={isExpanded}
                        aria-controls="search-options-body"
                        aria-label={`検索オプション ${isExpanded ? '閉じる' : '開く'}`}
                        data-testid="search-card-chevron-toggle"
                    >
                        <span className="text-xs font-semibold whitespace-nowrap text-white">
                            {isExpanded ? '閉じる' : '開く'}
                        </span>
                        <svg
                            className={cn('w-4 h-4 transition-transform duration-200', isExpanded && 'rotate-180')}
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                            aria-hidden="true"
                        >
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
                            </svg>
                        </button>
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
                    />
                </CardBody>
            )}
        </Card>
    );
};
