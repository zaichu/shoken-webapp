import { describe, it, expect } from 'vitest';
import { createGroupKeyFn, deriveSecurityCodeFromQuery } from '../searchGroupKey';

// テスト用データ型
interface TestItem {
    code: string;
    name: string;
    product: string;
    date: Date;
}

const dateKeyFn = (item: TestItem): string => {
    const y = item.date.getFullYear();
    const m = String(item.date.getMonth() + 1).padStart(2, '0');
    return `${y}-${m}`;
};

const rules = [
    {
        test: (item: TestItem, t: string) =>
            item.code.toLowerCase() === t || item.name.toLowerCase() === t,
        keyFn: (item: TestItem) => item.name,
    },
    {
        test: (item: TestItem, t: string) => item.product.toLowerCase() === t,
        keyFn: (item: TestItem) => item.product,
    },
];

const item: TestItem = {
    code: '7203',
    name: 'トヨタ自動車',
    product: '国内株式',
    date: new Date('2024-03-15'),
};

describe('createGroupKeyFn', () => {
    it('searchQueryが空の場合はdateKeyFnをそのまま返す', () => {
        const fn = createGroupKeyFn('', dateKeyFn, rules);
        expect(fn(item)).toBe('2024-03');
    });

    it('銘柄コードに完全一致するトークンで銘柄名グループを返す', () => {
        const fn = createGroupKeyFn('7203', dateKeyFn, rules);
        expect(fn(item)).toBe('トヨタ自動車');
    });

    it('銘柄名に完全一致するトークンで銘柄名グループを返す', () => {
        const fn = createGroupKeyFn('トヨタ自動車', dateKeyFn, rules);
        expect(fn(item)).toBe('トヨタ自動車');
    });

    it('商品に完全一致するトークンで商品グループを返す', () => {
        const fn = createGroupKeyFn('国内株式', dateKeyFn, rules);
        expect(fn(item)).toBe('国内株式');
    });

    it('いずれのルールにもマッチしない場合はdateKeyFnを使用する', () => {
        const fn = createGroupKeyFn('2024', dateKeyFn, rules);
        expect(fn(item)).toBe('2024-03');
    });

    it('ANDクエリ（スペース区切り）の最初にマッチしたルールを使用する', () => {
        const fn = createGroupKeyFn('7203 国内株式', dateKeyFn, rules);
        // 最初のルール（銘柄コード）が優先される
        expect(fn(item)).toBe('トヨタ自動車');
    });

    it('部分一致ルールで正しくグループ化できる', () => {
        const partialRules = [
            {
                test: (it: TestItem, t: string) => it.name.toLowerCase().includes(t),
                keyFn: (it: TestItem) => it.name,
            },
        ];
        const fn = createGroupKeyFn('トヨタ', dateKeyFn, partialRules);
        expect(fn(item)).toBe('トヨタ自動車');
    });
});

// テスト用セキュリティデータ
const securityData = [
    { security_code: '7203', security_name: 'トヨタ自動車' },
    { security_code: '9984', security_name: 'ソフトバンクグループ' },
];

describe('deriveSecurityCodeFromQuery', () => {
    it('queryが空の場合は空文字を返す', () => {
        expect(deriveSecurityCodeFromQuery('', securityData)).toBe('');
    });

    it('コード付きラベル形式（半角コロン）からコードを抽出する', () => {
        expect(deriveSecurityCodeFromQuery('7203: トヨタ自動車', securityData)).toBe('7203');
    });

    it('コード付きラベル形式（全角コロン）からコードを抽出する', () => {
        expect(deriveSecurityCodeFromQuery('7203：トヨタ自動車', securityData)).toBe('7203');
    });

    it('銘柄コードに完全一致するトークンから銘柄コードを返す', () => {
        expect(deriveSecurityCodeFromQuery('7203', securityData)).toBe('7203');
    });

    it('銘柄名に完全一致するトークンから銘柄コードを返す', () => {
        expect(deriveSecurityCodeFromQuery('トヨタ自動車', securityData)).toBe('7203');
    });

    it('どのデータにもマッチしない場合は空文字を返す', () => {
        expect(deriveSecurityCodeFromQuery('任天堂', securityData)).toBe('');
    });

    it('コードの前後にスペースがあるラベル形式でも抽出できる', () => {
        expect(deriveSecurityCodeFromQuery('  9984 ：ソフトバンクグループ', securityData)).toBe('9984');
    });
});
