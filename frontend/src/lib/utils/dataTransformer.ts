export function createSearchOptions<T>(
    data: T[], 
    valueField: keyof T, 
    labelField: keyof T, 
    prefix?: boolean
): { value: string, label: string }[] {
    return data
        .map(item => ({
            value: String(item[valueField] || item[labelField]),
            label: prefix 
                ? `${item[valueField] ? `${String(item[valueField])}:` : ''}${String(item[labelField])}` 
                : String(item[labelField]),
        }))
        .filter((item, index, self) => 
            index === self.findIndex(t => t.value === item.value)
        )
        .sort((a, b) => a.value.localeCompare(b.value));
}

export function filterDataBySearchQuery<T>(
    data: T[],
    searchQuery: string,
    searchFields: (keyof T)[]
): T[] {
    const query = searchQuery.toLowerCase();
    
    if (!query) {
        return data;
    }
    
    return data.filter(item => 
        searchFields.some(field => 
            String(item[field]).toLowerCase().includes(query)
        )
    );
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
