import { parseSearchTokens } from './searchUtils';

/**
 * グループキー決定ルール
 */
export interface GroupKeyRule<T> {
    /** ANDトークンとデータ行の一致判定 */
    test: (item: T, token: string) => boolean;
    /** マッチした場合のグループキー */
    keyFn: (item: T) => string;
}

/**
 * 検索クエリに応じたグループキー関数を生成する
 *
 * - searchQuery が空なら dateKeyFn を返す
 * - ANDトークンのうち最初にマッチしたルールのキーを返す
 * - いずれもマッチしなければ dateKeyFn を返す
 */
export function createGroupKeyFn<T>(
    searchQuery: string,
    dateKeyFn: (item: T) => string,
    rules: GroupKeyRule<T>[]
): (item: T) => string {
    if (!searchQuery) return dateKeyFn;

    const tokens = parseSearchTokens(searchQuery);

    return (item: T): string => {
        for (const rule of rules) {
            if (tokens.some(t => rule.test(item, t))) {
                return rule.keyFn(item);
            }
        }
        return dateKeyFn(item);
    };
}

/**
 * 検索クエリから銘柄コードを派生する
 *
 * - コード付きラベル形式（"XXXX: 銘柄名" / "XXXX：銘柄名"）はコード部分を返す
 * - ANDトークンのうち銘柄コード/銘柄名に完全一致するものがあればその銘柄コードを返す
 * - マッチなしは空文字
 */
export function deriveSecurityCodeFromQuery<T extends { security_code: string; security_name: string }>(
    query: string,
    data: T[]
): string {
    if (!query) return '';

    const labelMatch = query.match(/^\s*([0-9A-Za-z]+)\s*[:：]/);
    if (labelMatch) {
        const lowerCode = labelMatch[1].toLowerCase();
        const matchedItem = data.find(item => item.security_code.toLowerCase() === lowerCode);
        if (matchedItem) return matchedItem.security_code;
    }

    const tokens = parseSearchTokens(query);
    const matchedItem = data.find(item =>
        tokens.some(t =>
            item.security_code.toLowerCase() === t ||
            item.security_name.toLowerCase() === t
        )
    );
    return matchedItem?.security_code ?? '';
}
