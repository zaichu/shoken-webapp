import { Link } from 'react-router-dom';
import { Layout } from '../components/templates/Layout';
import { usePageTitle } from '../hooks/usePageTitle';

const STATUS_ITEMS = [
  {
    label: '銘柄検索',
    sub: '検索',
    to: '/search' as string | null,
    icon: (
      <svg className="h-5 w-5 text-primary" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true" focusable="false">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
      </svg>
    ),
  },
  {
    label: '資産管理',
    sub: '一覧確認',
    to: '/assetbalance' as string | null,
    icon: (
      <svg className="h-5 w-5 text-primary" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true" focusable="false">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
      </svg>
    ),
  },
  {
    label: '取引明細',
    sub: '明細確認',
    to: '/receipts' as string | null,
    icon: (
      <svg className="h-5 w-5 text-primary" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true" focusable="false">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>
    ),
  },
  {
    label: 'CSV取込',
    sub: 'CSV反映',
    to: null as string | null,
    icon: (
      <svg className="h-5 w-5 text-slate-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true" focusable="false">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
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
  { step: '1', text: '証券会社からCSVをダウンロード' },
  { step: '2', text: '各ページのCSV取込から反映' },
  { step: '3', text: '資産・配当金・取引明細を確認' },
] as const;

export function HomePage() {
  usePageTitle('ホーム');

  return (
    <Layout>
      <div className="page-surface space-y-6">
        {/* ダッシュボードヘッダー */}
        <div>
          <h1 className="text-xl font-bold text-slate-900">証券Web</h1>
          <p className="mt-0.5 text-sm text-slate-500">日々の資産・取引を確認する</p>
        </div>

        {/* ステータス／アクションストリップ */}
        <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
          {STATUS_ITEMS.map(({ label, sub, to, icon }) => {
            const inner = (
              <>
                <div className="mb-2">{icon}</div>
                <p className="text-sm font-semibold text-slate-800">{label}</p>
                <p className="text-xs text-slate-500 mt-0.5">{sub}</p>
              </>
            );

            if (to !== null) {
              return (
                <Link
                  key={label}
                  to={to}
                  className="rounded-lg border border-slate-200 bg-white px-4 py-3 hover:border-primary hover:shadow-sm transition-all group"
                >
                  {inner}
                </Link>
              );
            }

            return (
              <div
                key={label}
                className="rounded-lg border border-slate-200 bg-slate-50 px-4 py-3"
              >
                {inner}
                <p className="text-xs text-slate-600 mt-1">各ページから取込可能</p>
              </div>
            );
          })}
        </div>

        {/* 次に行う操作 */}
        <div className="pt-4 border-t border-slate-200">
          <h2 className="text-sm font-medium text-slate-500 mb-3">次に行う操作</h2>
          <div className="flex flex-wrap gap-2">
            {NEXT_ACTIONS.map(({ label, to }) => (
              <Link
                key={to}
                to={to}
                className="inline-flex items-center px-3 py-1.5 text-sm bg-slate-100 hover:bg-slate-200 text-slate-700 rounded-full transition-colors"
              >
                {label}
              </Link>
            ))}
          </div>
        </div>

        {/* データ確認フロー */}
        <div className="pt-4 border-t border-slate-200">
          <h2 className="text-sm font-medium text-slate-500 mb-3">データ確認フロー</h2>
          <div className="flex flex-wrap gap-2 items-center">
            {FLOW_STEPS.map(({ step, text }, i) => (
              <div key={step} className="flex items-center gap-2">
                <div className="flex items-center gap-1.5">
                  <span className="inline-flex h-5 w-5 items-center justify-center rounded-full bg-primary/10 text-xs font-semibold text-primary">
                    {step}
                  </span>
                  <span className="text-sm text-slate-700">{text}</span>
                </div>
                {i < FLOW_STEPS.length - 1 && (
                  <span className="text-slate-300 text-sm" aria-hidden="true">→</span>
                )}
              </div>
            ))}
          </div>
        </div>
      </div>
    </Layout>
  );
}
