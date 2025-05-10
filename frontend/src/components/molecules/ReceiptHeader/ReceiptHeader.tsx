import React from 'react';
import { StatItem } from '@/components';

interface HeaderItem {
    title: string;
    value: number;
    format: (value: number) => string;
}

interface ReceiptHeaderProps {
    items: HeaderItem[];
}

export const ReceiptHeader: React.FC<ReceiptHeaderProps> = ({ items }) => {
    return (
        <div className="card shadow-sm mt-1">
            <div className="card-header bg-primary text-white">
                <h5 className="mb-0">集計情報</h5>
            </div>
            <div className="card-body">
                <div className="row">
                    {items.map((item, index) => (
                        <StatItem key={index} title={item.title} value={item.format(item.value)} />
                    ))}
                </div>
            </div>
        </div>
    );
};
