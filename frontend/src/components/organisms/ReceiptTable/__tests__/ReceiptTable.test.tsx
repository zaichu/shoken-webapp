import React from 'react';
import { render, screen } from '@testing-library/react';
import { ReceiptTable } from '../ReceiptTable';
import { TableColumnConfig, SummaryColumnConfig } from '@/lib/interfaces/receipt';

interface TestItem {
    id: number;
    name: string;
    category: string;
    amount: number;
}

interface TestSummary {
    filter: string;
    total: number;
}

describe('ReceiptTable', () => {
    const testData: TestItem[] = [
        { id: 1, name: 'テスト1', category: 'A', amount: 1000 },
        { id: 2, name: 'テスト2', category: 'A', amount: 2000 },
        { id: 3, name: 'テスト3', category: 'B', amount: 3000 }
    ];

    const testSummary: TestSummary[] = [
        { filter: 'A', total: 3000 },
        { filter: 'B', total: 3000 }
    ];

    const getGroupKey = (item: TestItem) => item.category;

    const columns: TableColumnConfig[] = [
        { key: 'id', header: 'ID' },
        { key: 'name', header: '名前' },
        { key: 'category', header: 'カテゴリ' },
        { key: 'amount', header: '金額', format: (value) => `¥${value.toLocaleString()}` }
    ];

    const summaryColumns: SummaryColumnConfig[] = [
        { key: 'total', colSpan: 4, format: (value) => `¥${value.toLocaleString()}` }
    ];

    it('データが正しく表示される', () => {
        render(
            <ReceiptTable
                data={testData}
                summary={testSummary}
                columns={columns}
                summaryColumns={summaryColumns}
                getGroupKey={getGroupKey}
            />
        );

        expect(screen.getByText('ID')).toBeInTheDocument();
        expect(screen.getByText('名前')).toBeInTheDocument();
        expect(screen.getByText('カテゴリ')).toBeInTheDocument();
        expect(screen.getByText('金額')).toBeInTheDocument();

        testData.forEach(item => {
            expect(screen.getByText(String(item.id))).toBeInTheDocument();
            expect(screen.getByText(item.name)).toBeInTheDocument();
            expect(screen.getByText(item.category)).toBeInTheDocument();
            expect(screen.getByText(`¥${item.amount.toLocaleString()}`)).toBeInTheDocument();
        });

        testSummary.forEach(item => {
            expect(screen.getByText(`¥${item.total.toLocaleString()}`)).toBeInTheDocument();
        });
    });

    it('空のデータでも表示される', () => {
        render(
            <ReceiptTable
                data={[]}
                summary={[]}
                columns={columns}
                summaryColumns={summaryColumns}
                getGroupKey={getGroupKey}
            />
        );

        expect(screen.getByText('ID')).toBeInTheDocument();
        expect(screen.getByText('名前')).toBeInTheDocument();
        expect(screen.getByText('カテゴリ')).toBeInTheDocument();
        expect(screen.getByText('金額')).toBeInTheDocument();
    });

    it('スタイルとformatが正しく適用される', () => {
        const styledColumns: TableColumnConfig[] = [
            { key: 'id', header: 'ID', width: '50px', textAlign: 'center' },
            { key: 'name', header: '名前', width: '150px' },
            { key: 'category', header: 'カテゴリ' },
            { key: 'amount', header: '金額', width: '100px', textAlign: 'right', format: (value) => `¥${value.toLocaleString()}` }
        ];

        const { container } = render(
            <ReceiptTable
                data={testData}
                summary={testSummary}
                columns={styledColumns}
                summaryColumns={summaryColumns}
                getGroupKey={getGroupKey}
            />
        );

        const cells = container.querySelectorAll('td');
        const headers = container.querySelectorAll('th');
        
        expect(headers[0]).toHaveStyle('textAlign: center');
        expect(headers[3]).toHaveStyle('textAlign: center');
        
        expect(cells[0]).toHaveStyle('width: 50px');
        expect(cells[1]).toHaveStyle('width: 150px');
        expect(cells[3]).toHaveStyle('width: 100px');
        expect(cells[3]).toHaveStyle('textAlign: right');
    });
});
