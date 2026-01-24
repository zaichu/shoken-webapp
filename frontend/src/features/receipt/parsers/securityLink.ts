/**
 * HTMLエスケープ処理
 * XSS攻撃を防ぐため、特殊文字をエスケープ
 */
const escapeHtml = (str: string): string => {
    const htmlEscapes: Record<string, string> = {
        '&': '&amp;',
        '<': '&lt;',
        '>': '&gt;',
        '"': '&quot;',
        "'": '&#39;',
    };
    return str.replace(/[&<>"']/g, char => htmlEscapes[char]);
};

/**
 * 銘柄コードのリンクHTML生成
 * 銘柄検索ページへ遷移するリンクを生成
 * CSVからの入力値はエスケープしてXSSを防止
 */
export const createSecurityCodeLink = (securityCode: string): string => {
    const escapedCode = escapeHtml(securityCode);
    return `<a href="/search?code=${encodeURIComponent(securityCode)}" class="security-code-link" style="color: #0d6efd; font-weight: 600;">${escapedCode}</a>`;
};
