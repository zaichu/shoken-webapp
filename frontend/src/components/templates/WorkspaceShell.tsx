import { ReactNode } from 'react';
import { cn } from '@/lib/utils/classNames';

interface WorkspaceShellProps {
  main: ReactNode;
  rail: ReactNode;
  mainClassName?: string;
  testIdPrefix?: string;
}

export function WorkspaceShell({ main, rail, mainClassName, testIdPrefix = '' }: WorkspaceShellProps) {
  const prefix = testIdPrefix ? `${testIdPrefix}-` : '';
  return (
    <div
      className="grid gap-4 lg:grid-cols-[minmax(0,1fr)_19rem] lg:items-start xl:gap-5 xl:grid-cols-[minmax(0,1fr)_20rem]"
      data-testid={`${prefix}workspace`}
    >
      <div className={cn('min-w-0', mainClassName)} data-testid={`${prefix}main-stage`}>
        {main}
      </div>
      <aside data-testid={`${prefix}utility-rail`}>
        <div className="overflow-hidden rounded-xl border border-slate-950/10 bg-white/90 shadow-[0_18px_58px_-42px_rgba(15,23,42,0.9)] backdrop-blur-sm divide-y divide-slate-950/10">
          {rail}
        </div>
      </aside>
    </div>
  );
}
