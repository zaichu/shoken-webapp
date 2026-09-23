import { describe, it, expect } from 'vitest';
import {
    filterByConfig,
    FilterConfig,
    createYearOptions,
    getUniqueValues,
    matchesYear,
    matchesYearMonth,
} from '../searchUtils';
import fixtureData from '../../../../../frontend-leptos/tests/fixtures/receipts/search.json' with { type: 'json' };

interface FixtureItem {
    id: string;
    code: string;
    name: string;
    category: string;
    date: string;
    amount: number;
}

interface FixtureConfig {
    stringFields?: string[];
    partialStringFields?: string[];
    dateField?: string;
    yearSearch?: boolean;
    yearMonthSearch?: boolean;
    dateSearch?: boolean;
    dateRangeSearch?: boolean;
    amountFields?: string[];
}

interface FixtureCase {
    name: string;
    data: FixtureItem[];
    query: string;
    config: FixtureConfig;
    expected_ids: string[];
}

interface Fixture {
    cases: FixtureCase[];
}

const fixture: Fixture = fixtureData;

function createConfig(config: FixtureConfig): FilterConfig<FixtureItem> {
    const stringGetters: FilterConfig<FixtureItem>['stringFields'] = (config.stringFields || []).map(field => {
        switch (field) {
            case 'code': return (item: FixtureItem) => item.code;
            case 'name': return (item: FixtureItem) => item.name;
            case 'category': return (item: FixtureItem) => item.category;
            default: throw new Error(`unknown string field: ${field}`);
        }
    });

    const partialStringGetters: FilterConfig<FixtureItem>['partialStringFields'] = (config.partialStringFields || []).map(field => {
        switch (field) {
            case 'code': return (item: FixtureItem) => item.code;
            case 'name': return (item: FixtureItem) => item.name;
            case 'category': return (item: FixtureItem) => item.category;
            default: throw new Error(`unknown partial field: ${field}`);
        }
    });

    const amountGetters: FilterConfig<FixtureItem>['amountFields'] = (config.amountFields || []).map(() =>
        (item: FixtureItem) => item.amount
    );

    const dateGetter = config.dateField
        ? (item: FixtureItem) => new Date(item.date + 'T00:00:00.000Z')
        : undefined;

    return {
        stringFields: stringGetters.length > 0 ? stringGetters : undefined,
        partialStringFields: partialStringGetters.length > 0 ? partialStringGetters : undefined,
        dateField: dateGetter,
        yearSearch: config.yearSearch,
        yearMonthSearch: config.yearMonthSearch,
        dateSearch: config.dateSearch,
        dateRangeSearch: config.dateRangeSearch,
        amountFields: amountGetters.length > 0 ? amountGetters : undefined,
    };
}

function checkCase(case_: FixtureCase): void {
    const config = createConfig(case_.config);
    const dataWithDates: FixtureItem[] = case_.data.map(item => ({
        ...item,
        date: item.date,
    }));

    const result = filterByConfig(dataWithDates, case_.query, config);
    const actualIds = result.map(item => item.id);
    expect(actualIds).toEqual(case_.expected_ids);
}

describe('shared search fixtures (React)', () => {
    for (const case_ of fixture.cases) {
        it(case_.name, () => {
            checkCase(case_);
        });
    }
});

describe('shared createYearOptions fixtures (React)', () => {
    const yearCase = fixture.cases.find(c => c.name === 'year');
    if (yearCase) {
        it('createYearOptions returns correct options', () => {
            const data = yearCase.data.map(item => ({
                date: new Date(item.date + 'T00:00:00.000Z'),
            }));
            const options = createYearOptions(data, item => item.date);
            expect(options).toEqual([
                { value: '2023', label: '2023年' },
                { value: '2024', label: '2024年' },
            ]);
        });
    }

    it('createYearOptions returns empty for empty array', () => {
        expect(createYearOptions([], () => new Date())).toEqual([]);
    });
});

describe('shared getUniqueValues fixtures (React)', () => {
    it('returns unique values excluding empty', () => {
        const data = [
            { category: '特定' },
            { category: 'NISA' },
            { category: '特定' },
            { category: '' },
        ];
        const values = getUniqueValues(data, item => item.category);
        expect(values).toEqual(['特定', 'NISA']);
    });

    it('returns empty for empty array', () => {
        expect(getUniqueValues([], (item: { v: string }) => item.v)).toEqual([]);
    });
});

describe('shared matchesYear fixtures (React)', () => {
    it('returns true for matching year', () => {
        expect(matchesYear(new Date('2024-06-15'), '2024')).toBe(true);
    });

    it('returns false for non-matching year', () => {
        expect(matchesYear(new Date('2024-06-15'), '2023')).toBe(false);
    });
});

describe('shared matchesYearMonth fixtures (React)', () => {
    it('returns true for matching year-month', () => {
        expect(matchesYearMonth(new Date('2024-03-15'), '2024-03')).toBe(true);
    });

    it('returns false for non-zero-padded month', () => {
        expect(matchesYearMonth(new Date('2024-01-05'), '2024-01')).toBe(true);
        expect(matchesYearMonth(new Date('2024-01-05'), '2024-1')).toBe(false);
    });

    it('returns false for non-matching year-month', () => {
        expect(matchesYearMonth(new Date('2024-03-15'), '2024-04')).toBe(false);
    });
});
