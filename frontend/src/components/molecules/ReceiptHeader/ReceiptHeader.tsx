import React, { ReactNode, useId, useState } from 'react';
import { cn } from '@/lib/utils/classNames';
import { Card, CardHeader, CardBody } from '@/components/atoms/Card';
import { HeaderItem, KpiTone } from '@/types/common';

interface ReceiptHeaderProps {
    items: HeaderItem[];
    title?: string;
    children?: ReactNode;
    collapsible?: boolean;
    defaultExpanded?: boolean;
    compact?: boolean;
}

export const ReceiptHeader: React.FC<ReceiptHeaderProps> = ({
    items,
    title = '集計情報',
    children,
    collapsible = false,
    defaultExpanded = true,
    compact = false,
}) => {
    const [isExpanded, setIsExpanded] = useState(() => (collapsible ? defaultExpanded : true));
    const bodyId = useId();
    const effectiveExpanded = collapsible ? isExpanded : true;

    const handleToggleExpanded = () => {
        if (!collapsible) return;
        setIsExpanded(prev => !prev);
    };

    const TONE_CLASSES: Record<KpiTone, string> = {
        emerald: 'border-emerald-100 bg-emerald-50/90 shadow-[0_12px_24px_-28px_rgba(5,150,105,0.35)]',
        red:     'border-rose-100 bg-rose-50/90 shadow-[0_12px_24px_-28px_rgba(225,29,72,0.28)]',
        blue:    'border-blue-100 bg-blue-50/90 shadow-[0_12px_24px_-28px_rgba(37,99,235,0.28)]',
        slate:   'border-slate-200/90 bg-white shadow-[0_12px_24px_-28px_rgba(15,23,42,0.4)]',
    };

    const getCompactItemTone = (item: HeaderItem) => {
        // tone prop が優先。未指定の場合は後方互換で className から推定
        if (item.tone) return TONE_CLASSES[item.tone];
        const token = `${item.className ?? ''} ${item.valueClassName ?? ''}`;
        if (token.includes('emerald')) return TONE_CLASSES.emerald;
        if (token.includes('red'))     return TONE_CLASSES.red;
        if (token.includes('blue'))    return TONE_CLASSES.blue;
        return TONE_CLASSES.slate;
    };

    if (compact) {
        const compactChevron = collapsible && (
            <span className="flex items-center gap-1.5 rounded-full border border-slate-200 bg-white px-2.5 py-1 text-slate-700" aria-hidden="true">
                <span className="text-xs font-semibold">{effectiveExpanded ? '閉じる' : '開く'}</span>
                <svg
                    className={cn('w-4 h-4 text-slate-500 transition-transform duration-200', effectiveExpanded && 'rotate-180')}
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                >
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
                </svg>
            </span>
        );
        return (
            <section
                className="rounded-[2rem] border border-slate-200/90 bg-white/85 px-5 py-5 shadow-[0_22px_48px_-36px_rgba(15,23,42,0.45)]"
                data-testid="receipt-summary-strip"
            >
                {collapsible ? (
                    <button
                        type="button"
                        className="flex w-full items-start justify-between gap-3 border-b border-slate-200/80 pb-4 text-left"
                        onClick={handleToggleExpanded}
                        aria-expanded={effectiveExpanded}
                        aria-controls={bodyId}
                        data-testid="receipt-header"
                    >
                        <div><h2 className="text-sm font-semibold text-slate-800">{title}</h2></div>
                        {compactChevron}
                    </button>
                ) : (
                    <div
                        className="flex items-start justify-between gap-3 border-b border-slate-200/80 pb-4"
                        data-testid="receipt-header"
                    >
                        <div><h2 className="text-sm font-semibold text-slate-800">{title}</h2></div>
                    </div>
                )}
                <div id={bodyId} hidden={collapsible && !effectiveExpanded} className="pt-4">
                    {items.length > 0 && (
                        <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-3" data-testid="kpi-grid">
                            {items.map((item) => (
                                <div
                                    key={item.title}
                                    className={cn(
                                        'rounded-[1.35rem] border px-4 py-4',
                                        getCompactItemTone(item),
                                        item.className,
                                    )}
                                >
                                    <p className="mb-1 text-xs font-medium text-slate-600">{item.title}</p>
                                    <p
                                        className={cn('text-3xl font-bold tabular-nums', item.valueClassName ?? 'text-slate-800')}
                                        data-negative={item.value < 0 ? 'true' : undefined}
                                    >
                                        {item.format(item.value)}
                                    </p>
                                </div>
                            ))}
                        </div>
                    )}
                    {children && (
                        <div className={items.length > 0 ? 'mt-4 border-t border-slate-200/90 pt-4' : ''}>
                            {children}
                        </div>
                    )}
                </div>
            </section>
        );
    }

    const chevron = (
        <span
            className="flex items-center gap-1.5 rounded border border-white/30 bg-white/10 px-2 py-0.5"
            aria-hidden="true"
        >
            <span className="text-xs font-semibold text-white">
                {effectiveExpanded ? '閉じる' : '開く'}
            </span>
            <svg
                className={cn('w-4 h-4 transition-transform duration-200', effectiveExpanded && 'rotate-180')}
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
            >
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
            </svg>
        </span>
    );

    return (
        <Card>
            <CardHeader
                variant="secondary"
                className={cn(
                    collapsible
                        ? cn('p-0', effectiveExpanded ? 'bg-slate-600' : 'bg-slate-400')
                        : 'flex items-center justify-between bg-slate-600'
                )}
                data-testid={collapsible ? undefined : 'receipt-header'}
            >
                {collapsible ? (
                    <button
                        type="button"
                        className={cn(
                            'flex w-full items-center justify-between px-4 py-2 transition-colors',
                            effectiveExpanded ? 'hover:bg-slate-700' : 'hover:bg-slate-500',
                        )}
                        onClick={handleToggleExpanded}
                        aria-expanded={effectiveExpanded}
                        aria-controls={bodyId}
                        data-testid="receipt-header"
                    >
                        <h5 className="text-sm font-semibold text-white">{title}</h5>
                        {chevron}
                    </button>
                ) : (
                    <h5 className="text-sm font-semibold text-white">{title}</h5>
                )}
            </CardHeader>
            {/* 折りたたみ時はhiddenで非表示（children内のstateを保持するため） */}
            <CardBody id={bodyId} hidden={collapsible && !effectiveExpanded} className="p-4">
                {items.length > 0 && (
                    <div className="grid grid-cols-1 gap-4 md:grid-cols-3" data-testid="kpi-grid">
                        {items.map((item) => (
                            <div
                                key={item.title}
                                className={cn('rounded-lg px-4 py-3', item.className ?? 'bg-slate-50')}
                            >
                                <p className="text-xs font-medium text-slate-600 mb-1">{item.title}</p>
                                <p
                                    className={cn('text-2xl font-bold tabular-nums', item.valueClassName ?? 'text-slate-800')}
                                    data-negative={item.value < 0 ? 'true' : undefined}
                                >
                                    {item.format(item.value)}
                                </p>
                            </div>
                        ))}
                    </div>
                )}
                {children && (
                    <div className={items.length > 0 ? 'mt-4 pt-4 border-t border-slate-200' : ''}>
                        {children}
                    </div>
                )}
            </CardBody>
        </Card>
    );
};
