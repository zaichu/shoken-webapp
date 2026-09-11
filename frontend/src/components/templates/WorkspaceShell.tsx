import { ReactNode } from 'react';
import { cn } from '@/lib/utils/classNames';

interface WorkspaceShellProps {
  main: ReactNode;
  rail: ReactNode;
  mainClassName?: string;
  railClassName?: string;
  testIdPrefix?: string;
  /**
   * Issue #839: true のとき rail を main より前にレンダリングする（DOM順自体を変える）。
   * CSS order と違い Tab 順・スクリーンリーダーの読み上げ順も rail 先になる。
   * 条件レンダリングではなく初回マウント時から固定の順序なので、
   * ブレークポイント跨ぎで rail 内コンポーネントの内部stateは失われない。
   * lg以上での配置は呼び出し元が渡す order クラスで戻す。
   */
  railFirst?: boolean;
}

export function WorkspaceShell({ main, rail, mainClassName, railClassName, testIdPrefix = '', railFirst = false }: WorkspaceShellProps) {
  const prefix = testIdPrefix ? `${testIdPrefix}-` : '';
  const mainStage = (
    <div className={cn('min-w-0', mainClassName)} data-testid={`${prefix}main-stage`}>
      {main}
    </div>
  );
  const utilityRail = (
    <aside className={railClassName} data-testid={`${prefix}utility-rail`}>
      <div className="overflow-hidden rounded-xl border border-slate-950/10 bg-white/90 shadow-[0_18px_58px_-42px_rgba(15,23,42,0.9)] backdrop-blur-sm divide-y divide-slate-950/10">
        {rail}
      </div>
    </aside>
  );
  return (
    <div
      className="grid gap-4 lg:grid-cols-[minmax(0,1fr)_19rem] lg:items-start xl:gap-5 xl:grid-cols-[minmax(0,1fr)_20rem]"
      data-testid={`${prefix}workspace`}
    >
      {railFirst ? utilityRail : mainStage}
      {railFirst ? mainStage : utilityRail}
    </div>
  );
}
