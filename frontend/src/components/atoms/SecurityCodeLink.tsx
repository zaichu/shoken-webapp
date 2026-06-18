import { cn } from '@/lib/utils/classNames';
import { normalizeSecurityCode, SECURITY_CODE_REGEX } from '@/lib/utils/formatters';

interface SecurityCodeLinkProps {
  value: unknown;
  className?: string;
}

// hover:font-bold や sm:font-semibold 等の responsive/state prefix を含む font-weight クラスを検出する
const FONT_WEIGHT_CLASS_REGEX = /(?:^|\s)(?:[a-z-]+:)*font-(?:thin|extralight|light|normal|medium|semibold|bold|extrabold|black)\b/;

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

  const hasFontWeight = !!className && FONT_WEIGHT_CLASS_REGEX.test(className);

  return (
    <a
      href={`/search?code=${encodeURIComponent(code)}`}
      className={cn(
        'security-code-link text-blue-700 underline-offset-2 hover:text-blue-900 hover:underline focus-visible:rounded-sm focus-visible:outline-2 focus-visible:outline-blue-500',
        !hasFontWeight && 'font-bold',
        className
      )}
      data-search={code}
    >
      {code}
    </a>
  );
};

interface CopyableInstrumentNameProps {
  name: unknown;
  code?: unknown;
  className?: string;
}

const toDisplayText = (value: unknown): string => {
  if (value === null || value === undefined) return "-";
  const text = String(value).trim();
  return text || "-";
};

const createInstrumentCopyText = (name: unknown, code?: unknown): string => {
  const displayName = toDisplayText(name);
  const normalizedCode = normalizeSecurityCode(code);
  return normalizedCode ? displayName + "(" + normalizedCode + ")" : displayName;
};

export const CopyableInstrumentName: React.FC<CopyableInstrumentNameProps> = ({ name, code, className }) => {
  const displayName = toDisplayText(name);
  const copyText = createInstrumentCopyText(name, code);

  const handleContextMenu = (event: React.MouseEvent<HTMLSpanElement>) => {
    event.preventDefault();
    if (copyText && navigator.clipboard?.writeText) {
      void navigator.clipboard.writeText(copyText);
    }
  };

  const combinedClassName = className ? className + " cursor-copy" : "cursor-copy";

  return (
    <span className={combinedClassName} title={copyText} onContextMenu={handleContextMenu}>
      {displayName}
    </span>
  );
};

/**
 * カラムのformat用レンダラー（ReactNodeを返す）
 */
// eslint-disable-next-line react-refresh/only-export-components
export function renderSecurityCode(value: unknown): React.ReactNode {
  return <SecurityCodeLink value={value} />;
}
