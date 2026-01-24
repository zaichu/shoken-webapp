/**
 * 銘柄コードのリンクHTML生成
 * 銘柄検索ページへ遷移するリンクを生成
 */
export const createSecurityCodeLink = (securityCode: string): string => {
    return `<a href="/search?code=${securityCode}" class="security-code-link" style="color: #0d6efd; font-weight: 600;">${securityCode}</a>`;
};
