import React, { ReactNode, useEffect, useId, useState } from 'react';
import { StatItem } from '@/components/atoms/StatItem';
import { Card, CardBody, CardHeader } from '@/components/atoms/Card';

interface HeaderItem {
    title: string;
    value: number;
    format: (value: number) => string;
}

interface ReceiptHeaderProps {
    items: HeaderItem[];
    title?: string;
    children?: ReactNode;
    collapsible?: boolean;
}

export const ReceiptHeader: React.FC<ReceiptHeaderProps> = ({
    items,
    title = '集計情報',
    children,
    collapsible = false
}) => {
    const [isExpanded, setIsExpanded] = useState(true);
    const bodyId = useId();

    // 折りたたみを無効化したら展開状態に戻す
    useEffect(() => {
        if (!collapsible) {
            setIsExpanded(true);
        }
    }, [collapsible]);

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
                variant="primary"
                className={collapsible ? 'flex cursor-pointer items-center justify-between' : undefined}
                onClick={collapsible ? handleToggleExpanded : undefined}
                onKeyDown={collapsible ? handleKeyDown : undefined}
                role={collapsible ? 'button' : undefined}
                tabIndex={collapsible ? 0 : undefined}
                aria-expanded={collapsible ? isExpanded : undefined}
                aria-controls={collapsible ? bodyId : undefined}
                data-testid="receipt-header"
            >
                <h5>{title}</h5>
                {collapsible && (
                    <span className="flex items-center gap-1 text-xs font-medium bg-white/20 px-2 py-1 rounded">
                        {isExpanded ? '▲ 閉じる' : '▼ 開く'}
                    </span>
                )}
            </CardHeader>
            {(!collapsible || isExpanded) && (
                <CardBody id={bodyId} className="p-4">
                    {/* stat-gridはitemsがある場合のみ表示 */}
                    {items.length > 0 && (
                        <div className="stat-grid">
                            {items.map((item, index) => (
                                <StatItem
                                    key={index}
                                    title={item.title}
                                    value={
                                        <span data-negative={item.value < 0 ? 'true' : undefined}>
                                            {item.format(item.value)}
                                        </span>
                                    }
                                />
                            ))}
                        </div>
                    )}
                    {/* childrenはitemsがある場合のみ上余白を設ける */}
                    {children && (
                        <div className={items.length > 0 ? 'mt-4 pt-4 border-t border-slate-200' : ''}>
                            {children}
                        </div>
                    )}
                </CardBody>
            )}
        </Card >
    );
};
