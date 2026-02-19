import React, { ReactNode, useEffect, useId, useState } from 'react';
import { cn } from '@/lib/utils/classNames';
import { Card, CardHeader, CardBody } from '@/components/atoms/Card';
import { HeaderItem } from '@/types/common';

interface ReceiptHeaderProps {
    items: HeaderItem[];
    title?: string;
    children?: ReactNode;
    collapsible?: boolean;
    defaultExpanded?: boolean;
}

export const ReceiptHeader: React.FC<ReceiptHeaderProps> = ({
    items,
    title = '集計情報',
    children,
    collapsible = false,
    defaultExpanded = true
}) => {
    const [isExpanded, setIsExpanded] = useState(() => (collapsible ? defaultExpanded : true));
    const bodyId = useId();

    useEffect(() => {
        setIsExpanded(collapsible ? defaultExpanded : true);
    }, [collapsible, defaultExpanded]);
    const effectiveExpanded = collapsible ? isExpanded : true;

    const handleToggleExpanded = () => {
        if (!collapsible) return;
        setIsExpanded(prev => !prev);
    };

    const handleKeyDown = (e: React.KeyboardEvent) => {
        if (!collapsible) return;
        if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            handleToggleExpanded();
        }
    };

    return (
        <Card>
            <CardHeader
                variant="secondary"
                className={cn(
                    'flex items-center justify-between',
                    collapsible
                        ? cn('cursor-pointer select-none transition-colors', effectiveExpanded ? 'bg-slate-600 hover:bg-slate-700' : 'bg-slate-400 hover:bg-slate-500')
                        : 'bg-slate-600'
                )}
                onClick={collapsible ? handleToggleExpanded : undefined}
                onKeyDown={collapsible ? handleKeyDown : undefined}
                role={collapsible ? 'button' : undefined}
                tabIndex={collapsible ? 0 : undefined}
                aria-expanded={collapsible ? effectiveExpanded : undefined}
                aria-controls={collapsible ? bodyId : undefined}
                data-testid="receipt-header"
            >
                <h5 className="text-sm font-semibold text-white">{title}</h5>
                {collapsible && (
                    <span className="flex items-center gap-1.5 rounded border border-white/30 bg-white/10 px-2 py-0.5" aria-hidden="true">
                        <span className="text-xs font-semibold text-white">
                            {effectiveExpanded ? '閉じる' : '開く'}
                        </span>
                        <svg
                            className={`w-4 h-4 transition-transform duration-200 ${effectiveExpanded ? 'rotate-180' : ''}`}
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                        >
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
                        </svg>
                    </span>
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
