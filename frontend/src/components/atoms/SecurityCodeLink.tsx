import { normalizeSecurityCode, SECURITY_CODE_REGEX } from '@/lib/utils/formatters';

interface SecurityCodeLinkProps {
  value: unknown;
  className?: string;
}

/**
 * 銘柄コードをリンクとして表示するコンポーネント
 * - 銘柄コードを正規化してパラメータ検証
 * - 有効なコードのみリンク化
 */
export const SecurityCodeLink: React.FC<SecurityCodeLinkProps> = ({ value, className }) => {
  const code = normalizeSecurityCode(value);

  if (!code) {
    return <span>-</span>;
  }

  // 想定外の文字列はリンク化せずテキスト表示のみ
  if (!SECURITY_CODE_REGEX.test(code)) {
    return <span>{code}</span>;
  }

  const baseClassName = 'security-code-link text-primary font-semibold hover:underline';
  const combinedClassName = className ? `${baseClassName} ${className}` : baseClassName;

  return (
    <a
      href={`/search?code=${encodeURIComponent(code)}`}
      className={combinedClassName}
      data-search={code}
    >
      {code}
    </a>
  );
};

/**
 * カラムのformat用レンダラー（ReactNodeを返す）
 */
// eslint-disable-next-line react-refresh/only-export-components
export function renderSecurityCode(value: unknown): React.ReactNode {
  return <SecurityCodeLink value={value} />;
}
