import React from 'react';
import { StatItem } from '@/components/atoms/StatItem';
import { Card, CardBody, CardHeader } from '@/components/atoms/Card';

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
        <Card>
            <CardHeader variant="primary">
                <h5>集計情報</h5>
            </CardHeader>
            <CardBody className="p-4">
                <div className="grid grid-cols-1 md:grid-cols-3">
                    {items.map((item, index) => (
                        <StatItem key={index} title={item.title} value={item.format(item.value)} />
                    ))}
                </div>
            </CardBody>
        </Card >
    );
};
