import React from 'react';
import { render, screen } from '@testing-library/react';
import { ReceiptHeader } from '../ReceiptHeader';

describe('ReceiptHeader', () => {
    const mockFormatter = (value: number) => `¥${value.toLocaleString()}`;

    it('各項目が正しく表示される', () => {
        const items = [
            { title: 'テスト項目1', value: 1000, format: mockFormatter },
            { title: 'テスト項目2', value: 2000, format: mockFormatter },
            { title: 'テスト項目3', value: 3000, format: mockFormatter }
        ];

        render(<ReceiptHeader items={items} />);

        expect(screen.getByText('集計情報')).toBeInTheDocument();
        
        items.forEach(item => {
            expect(screen.getByText(item.title)).toBeInTheDocument();
            expect(screen.getByText(mockFormatter(item.value))).toBeInTheDocument();
        });
    });

    it('0項目でも表示される', () => {
        render(<ReceiptHeader items={[]} />);
        expect(screen.getByText('集計情報')).toBeInTheDocument();
    });

    it('1項目でも表示される', () => {
        const items = [
            { title: '単一項目', value: 1000, format: mockFormatter }
        ];

        render(<ReceiptHeader items={items} />);
        
        expect(screen.getByText('集計情報')).toBeInTheDocument();
        expect(screen.getByText('単一項目')).toBeInTheDocument();
        expect(screen.getByText(mockFormatter(1000))).toBeInTheDocument();
    });
});
