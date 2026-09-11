import { ReactNode } from 'react';
import { cn } from '../../lib/utils/classNames';

interface PageHeaderProps {
  title: string;
  eyebrow?: string;
  description?: string;
  actions?: ReactNode;
  className?: string;
}

export function PageHeader({ title, eyebrow, description, actions, className }: PageHeaderProps) {
  return (
    <div className={cn('mb-5 max-sm:mb-2', className)}>
      <div className="flex flex-col gap-3 border-l-4 border-amber-500 pl-4 sm:flex-row sm:items-end sm:justify-between">
        <div>
          {eyebrow ? (
            <p className="mb-1 text-[11px] font-bold uppercase tracking-[0.22em] text-slate-500 max-sm:hidden">{eyebrow}</p>
          ) : null}
          <h1 className="text-2xl font-black leading-tight tracking-normal text-slate-950 max-sm:text-lg">{title}</h1>
          {description ? (
            <p className="mt-1 text-sm font-medium text-slate-600 max-sm:hidden">{description}</p>
          ) : null}
        </div>
        {actions ? <div className="flex items-center gap-2">{actions}</div> : null}
      </div>
    </div>
  );
}
