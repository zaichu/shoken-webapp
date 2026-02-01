import { ReactNode } from 'react';
import { cn } from '../../lib/utils/classNames';

interface SectionHeaderProps {
  title: string;
  description?: string;
  actions?: ReactNode;
  className?: string;
  level?: 2 | 3;
}

/**
 * セクション見出しコンポーネント
 * H2/H3レベルの見出しとして使用
 */
export function SectionHeader({
  title,
  description,
  actions,
  className,
  level = 2,
}: SectionHeaderProps) {
  const Tag = `h${level}` as const;
  const titleClasses = level === 2
    ? 'text-xl font-semibold text-slate-900'
    : 'text-lg font-medium text-slate-900';

  return (
    <div className={cn('mb-4', className)}>
      <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <Tag className={titleClasses}>{title}</Tag>
          {description && (
            <p className="mt-1 text-sm text-slate-600">{description}</p>
          )}
        </div>
        {actions && <div className="flex items-center gap-2">{actions}</div>}
      </div>
    </div>
  );
}
