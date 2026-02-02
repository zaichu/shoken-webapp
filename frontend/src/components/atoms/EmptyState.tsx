import { ReactNode } from 'react';
import { cn } from '../../lib/utils/classNames';

type HeadingLevel = 'h1' | 'h2' | 'h3';

interface EmptyStateProps {
  icon?: ReactNode;
  title: string;
  description?: string;
  action?: ReactNode;
  className?: string;
  /** 見出しレベル（デフォルト: h3） */
  headingLevel?: HeadingLevel;
}

const headingStyles: Record<HeadingLevel, string> = {
  h1: 'text-2xl font-bold text-slate-900',
  h2: 'text-xl font-bold text-slate-900',
  h3: 'text-base font-semibold text-slate-900',
};

/**
 * 空状態を表示するコンポーネント
 * データがない場合やエラー時に使用
 */
export function EmptyState({
  icon,
  title,
  description,
  action,
  className,
  headingLevel = 'h3',
}: EmptyStateProps) {
  const Heading = headingLevel;

  return (
    <div
      className={cn(
        'flex flex-col items-center justify-center py-8 px-4 text-center',
        className
      )}
    >
      {icon && (
        <div className="mb-3 text-slate-400" aria-hidden="true">
          {icon}
        </div>
      )}
      <Heading className={headingStyles[headingLevel]}>{title}</Heading>
      {description && (
        <p className="mt-1.5 max-w-md text-sm text-slate-600">{description}</p>
      )}
      {action && <div className="mt-4">{action}</div>}
    </div>
  );
}
