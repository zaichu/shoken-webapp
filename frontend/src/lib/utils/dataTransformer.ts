export function createSearchOptions<T>(
    data: T[],
    valueField: keyof T,
    labelField: keyof T,
    prefix?: boolean
): { value: string, label: string }[] {
    const seen = new Map<string, { value: string; label: string }>();
    for (const item of data) {
        const value = String(item[valueField] || item[labelField]);
        if (!seen.has(value)) {
            seen.set(value, {
                value,
                label: prefix
                    ? `${item[valueField] ? `${String(item[valueField])}: ` : ''}${String(item[labelField])}`
                    : String(item[labelField]),
            });
        }
    }
    return [...seen.values()].sort((a, b) => a.value.localeCompare(b.value));
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
