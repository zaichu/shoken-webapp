import { Link } from 'react-router-dom';
import { Layout } from '../components/templates/Layout';
import { usePageTitle } from '../hooks/usePageTitle';

const STATUS_ITEMS = [
  {
    label: '銘柄検索',
    sub: '検索',
    to: '/search' as string | null,
    icon: (
      <svg className="h-5 w-5 text-slate-950" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true" focusable="false">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.8} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
      </svg>
    ),
  },
  {
    label: '資産管理',
    sub: '一覧確認',
    to: '/assetbalance' as string | null,
    icon: (
      <svg className="h-5 w-5 text-slate-950" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true" focusable="false">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.8} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
      </svg>
    ),
  },
  {
    label: '取引明細',
    sub: '明細確認',
    to: '/receipts' as string | null,
    icon: (
      <svg className="h-5 w-5 text-slate-950" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true" focusable="false">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.8} d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>
    ),
  },
  {
    label: 'CSV取込',
    sub: 'CSV反映',
    to: null as string | null,
    icon: (
      <svg className="h-5 w-5 text-slate-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true" focusable="false">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.8} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
      </svg>
    ),
  },
] as const;

const NEXT_ACTIONS = [
  { label: '銘柄検索', to: '/search' },
  { label: '資産管理', to: '/assetbalance' },
  { label: '取引明細', to: '/receipts' },
] as const;

const FLOW_STEPS = [
  { step: '01', text: 'CSV取得' },
  { step: '02', text: '各ページで取込' },
  { step: '03', text: '資産と明細を確認' },
] as const;

export function HomePage() {
  usePageTitle('ホーム');

  return (
    <Layout>
      <div className="page-surface space-y-7">
        <div className="grid gap-6 lg:grid-cols-[minmax(0,1.25fr)_minmax(20rem,0.75fr)] lg:items-end">
          <div>
            <p className="mb-2 text-[11px] font-black uppercase tracking-[0.28em] text-amber-700">Portfolio Desk</p>
            <h1 className="text-3xl font-black leading-tight tracking-normal text-slate-950 sm:text-4xl">証券Web</h1>
            <p className="mt-2 max-w-2xl text-sm font-medium text-slate-600">資産、配当、取引明細をひとつの作業面で確認します。</p>
          </div>
          <div className="rounded-xl border border-slate-950/10 bg-slate-950 p-4 text-white shadow-[0_18px_44px_-34px_rgba(15,23,42,0.95)]">
            <p className="text-[11px] font-bold uppercase tracking-[0.22em] text-amber-300">Current Focus</p>
            <div className="mt-3 flex flex-wrap gap-2">
              {NEXT_ACTIONS.map(({ label, to }) => (
                <Link
                  key={to}
                  to={to}
                  className="rounded-md border border-white/15 bg-white/10 px-3 py-2 text-sm font-bold text-white transition-colors hover:bg-white hover:text-slate-950"
                >
                  {label}
                </Link>
              ))}
            </div>
          </div>
        </div>

        <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-4">
          {STATUS_ITEMS.map(({ label, sub, to, icon }) => {
            const inner = (
              <>
                <div className="flex items-start justify-between gap-3">
                  <span className="inline-flex h-10 w-10 items-center justify-center rounded-md border border-slate-950/10 bg-white shadow-sm">
                    {icon}
                  </span>
                  <span className="text-[11px] font-black uppercase tracking-[0.18em] text-slate-400">{sub}</span>
                </div>
                <p className="mt-4 text-base font-black text-slate-950">{label}</p>
              </>
            );

            if (to !== null) {
              return (
                <Link
                  key={label}
                  to={to}
                  className="group rounded-xl border border-slate-950/10 bg-white/80 px-4 py-4 shadow-sm transition-[border-color,box-shadow,transform] hover:-translate-y-0.5 hover:border-amber-500 hover:shadow-[0_18px_40px_-32px_rgba(15,23,42,0.85)]"
                >
                  {inner}
                </Link>
              );
            }

            return (
              <div
                key={label}
                className="rounded-xl border border-dashed border-slate-300 bg-slate-100/70 px-4 py-4"
              >
                {inner}
                <p className="mt-2 text-xs font-medium text-slate-500">各ページから取込可能</p>
              </div>
            );
          })}
        </div>

        <div className="grid gap-4 border-t border-slate-950/10 pt-5 lg:grid-cols-[minmax(0,1fr)_minmax(18rem,0.55fr)]">
          <section>
            <h2 className="text-sm font-black text-slate-950">データ確認フロー</h2>
            <div className="mt-3 grid gap-2 sm:grid-cols-3">
              {FLOW_STEPS.map(({ step, text }) => (
                <div key={step} className="rounded-lg border border-slate-950/10 bg-white/75 px-3 py-3">
                  <span className="text-[11px] font-black uppercase tracking-[0.18em] text-amber-700">{step}</span>
                  <p className="mt-1 text-sm font-bold text-slate-800">{text}</p>
                </div>
              ))}
            </div>
          </section>
          <section className="rounded-lg border border-slate-950/10 bg-white/70 px-4 py-3">
            <h2 className="text-sm font-black text-slate-950">操作ショートカット</h2>
            <div className="mt-3 flex flex-wrap gap-2">
              {NEXT_ACTIONS.map(({ label, to }) => (
                <Link
                  key={to}
                  to={to}
                  className="inline-flex items-center rounded-md border border-slate-300 bg-white px-3 py-1.5 text-sm font-bold text-slate-700 transition-colors hover:border-slate-950 hover:text-slate-950"
                >
                  {label}
                </Link>
              ))}
            </div>
          </section>
        </div>
      </div>
    </Layout>
  );
}
