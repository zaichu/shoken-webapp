function toDate(d: Date | string | null | undefined): Date | null {
    if (!d) return null;
    const parsed = d instanceof Date ? d : new Date(d);
    return Number.isNaN(parsed.getTime()) ? null : parsed;
}

export function createSearchOptions<T>(
    data: T[],
    valueField: keyof T,
    labelField: keyof T,
    prefix?: boolean,
    dateField?: keyof T | ((item: T) => Date | string | null | undefined)
): { value: string, label: string }[] {
    const getDate = dateField
        ? (typeof dateField === 'function'
            ? dateField
            : (item: T) => item[dateField] as Date | string | null | undefined)
        : undefined;

    const labels = new Map<string, string>();
    const dates = new Map<string, Date | null>();

    for (const item of data) {
        const value = String(item[valueField] || item[labelField]);
        const label = prefix
            ? `${item[valueField] ? `${String(item[valueField])}: ` : ''}${String(item[labelField])}`
            : String(item[labelField]);
        const itemDate = getDate ? toDate(getDate(item)) : null;

        if (!labels.has(value)) {
            labels.set(value, label);
            dates.set(value, itemDate);
        } else if (itemDate) {
            const existingDate = dates.get(value) ?? null;
            if (!existingDate || itemDate > existingDate) {
                labels.set(value, label);
                dates.set(value, itemDate);
            }
        }
    }

    return [...labels.entries()]
        .map(([value, label]) => ({ value, label }))
        .sort((a, b) => a.value.localeCompare(b.value));
}

export type SummaryResult<K extends string | number | symbol> = {
    filter: string;
    [key: string]: unknown;
} & Record<K, number>;

export function groupAndSummarizeData<T, K extends keyof T>(
    data: T[],
    groupByFn: (item: T) => string,
    sumFields: K[]
): SummaryResult<K>[] {
    const groupMap = new Map<string, Record<K, number>>();

    data.forEach(item => {
        const key = groupByFn(item);

        if (!groupMap.has(key)) {
            const initial = {} as Record<K, number>;
            sumFields.forEach(field => {
                initial[field] = 0;
            });
            groupMap.set(key, initial);
        }

        const group = groupMap.get(key)!;
        sumFields.forEach(field => {
            group[field] += Number(item[field]) || 0;
        });
    });

    return Array.from(groupMap.entries())
        .map(([filter, values]) => ({
            filter,
            ...values,
            [Symbol.for('key')]: undefined // インデックスシグネチャを満たすための仮のプロパティ
        } as SummaryResult<K>))
        .sort((a, b) => a.filter.localeCompare(b.filter));
}
