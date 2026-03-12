import { ReactNode } from 'react';
import { cn } from '@/lib/utils/classNames';

interface WorkspaceShellProps {
  main: ReactNode;
  rail: ReactNode;
  mainClassName?: string;
  testIdPrefix?: string;
}

/**
 * 2カラム workspace レイアウト（main stage + utility rail）の共通テンプレート。
 * ReceiptTemplate および AssetBalance の grid / rail スタイルを一元管理する。
 */
export function WorkspaceShell({ main, rail, mainClassName, testIdPrefix = '' }: WorkspaceShellProps) {
  const prefix = testIdPrefix ? `${testIdPrefix}-` : '';
  return (
    <div
      className="grid gap-5 lg:grid-cols-[minmax(0,1fr)_22rem] lg:items-start xl:gap-6 xl:grid-cols-[minmax(0,1fr)_24rem]"
      data-testid={`${prefix}workspace`}
    >
      <div className={cn('min-w-0', mainClassName)} data-testid={`${prefix}main-stage`}>
        {main}
      </div>
      <aside data-testid={`${prefix}utility-rail`}>
        <div className="overflow-hidden rounded-[2rem] border border-slate-200/90 bg-white/80 shadow-[0_20px_48px_-34px_rgba(15,23,42,0.45)] backdrop-blur-sm divide-y divide-slate-200/80">
          {rail}
        </div>
      </aside>
    </div>
  );
}
