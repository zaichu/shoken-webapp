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

  return (
    <span className={cn('group inline-flex items-center gap-0.5', className)}>
      <span>{displayName}</span>
      <button
        type="button"
        aria-label={`${copyText} をコピー`}
        onClick={() => { void navigator.clipboard?.writeText(copyText); }}
        className="opacity-0 group-hover:opacity-100 focus:opacity-100 rounded p-0.5 text-gray-400 hover:text-gray-600 focus-visible:outline-2 focus-visible:outline-blue-500"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
          <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
          <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
        </svg>
      </button>
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
