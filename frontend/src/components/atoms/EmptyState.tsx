import { ReactNode } from 'react';
import { cn } from '../../lib/utils/classNames';

type HeadingLevel = 'h1' | 'h2' | 'h3';

interface EmptyStateProps {
  icon?: ReactNode;
  title: string;
  description?: string;
  action?: ReactNode;
  className?: string;
  headingLevel?: HeadingLevel;
}

const headingStyles: Record<HeadingLevel, string> = {
  h1: 'text-2xl font-black text-slate-950',
  h2: 'text-xl font-black text-slate-950',
  h3: 'text-base font-black text-slate-950',
};

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
        'flex flex-col items-center justify-center rounded-xl border border-dashed border-slate-300 bg-slate-50/70 px-4 py-8 text-center',
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
        <p className="mt-1.5 max-w-md text-sm font-medium text-slate-600">{description}</p>
      )}
      {action ? <div className="mt-4">{action}</div> : null}
    </div>
  );
}
