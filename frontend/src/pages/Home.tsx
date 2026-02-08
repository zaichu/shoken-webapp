import { Link } from 'react-router-dom';
import { Layout } from '../components/templates/Layout';
import { Card, CardBody } from '../components/atoms/Card';
import { PageHeader } from '../components/atoms/PageHeader';

// クイックアクション（ショートカット）
const QUICK_ACTIONS = [
  { label: '配当金を確認', to: '/receipts', icon: '💰' },
  { label: '保有銘柄を確認', to: '/assetbalance', icon: '📊' },
  { label: '銘柄を検索', to: '/search', icon: '🔍' },
] as const;

const FEATURES = [
  {
    title: '銘柄検索',
    description: '日本の株式銘柄情報を検索できます。',
    to: '/search',
    icon: (
      <svg className="h-6 w-6 text-primary" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true" focusable="false">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
      </svg>
    ),
  },
  {
    title: '保有銘柄',
    description: '保有している銘柄の一覧と評価額を確認できます。',
    to: '/assetbalance',
    icon: (
      <svg className="h-6 w-6 text-primary" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true" focusable="false">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
      </svg>
    ),
  },
  {
    title: '受取金',
    description: '配当金や分配金の記録を管理できます。',
    to: '/receipts',
    icon: (
      <svg className="h-6 w-6 text-primary" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true" focusable="false">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>
    ),
  },
] as const;

export function HomePage() {
  return (
    <Layout>
      <div className="rounded-lg border border-slate-200 bg-white p-4 shadow-sm">
        <PageHeader
          title="証券Webへようこそ"
          description="銘柄検索や受取金の管理ができます。"
        />

        <div className="grid gap-4 md:grid-cols-3">
          {FEATURES.map(({ title, description, to, icon }) => (
            <Link key={to} to={to} className="block group">
              <Card className="h-full transition-shadow hover:shadow-md">
                <CardBody className="space-y-2 p-3">
                  <div className="flex items-center gap-2">
                    {icon}
                    <h3 className="text-base font-semibold text-slate-900 group-hover:text-primary transition-colors">
                      {title}
                    </h3>
                  </div>
                  <p className="text-sm text-slate-600">{description}</p>
                </CardBody>
              </Card>
            </Link>
          ))}
        </div>

        {/* クイックアクション */}
        <div className="mt-6 pt-4 border-t border-slate-200">
          <h2 className="text-sm font-medium text-slate-500 mb-3">クイックアクション</h2>
          <div className="flex flex-wrap gap-2">
            {QUICK_ACTIONS.map(({ label, to, icon }) => (
              <Link
                key={to}
                to={to}
                className="inline-flex items-center gap-1.5 px-3 py-1.5 text-sm bg-slate-100 hover:bg-slate-200 text-slate-700 rounded-full transition-colors"
              >
                <span aria-hidden="true">{icon}</span>
                {label}
              </Link>
            ))}
          </div>
        </div>

        {/* はじめかた */}
        <div className="mt-6 pt-4 border-t border-slate-200">
          <h2 className="text-sm font-medium text-slate-500 mb-3">はじめかた</h2>
          <div className="grid gap-3 sm:grid-cols-3">
            <div className="rounded-lg border border-slate-200 bg-slate-50 p-3">
              <p className="text-xs font-semibold text-slate-500 mb-1">STEP 1</p>
              <p className="text-sm text-slate-700">証券会社からCSVをダウンロード</p>
            </div>
            <div className="rounded-lg border border-slate-200 bg-slate-50 p-3">
              <p className="text-xs font-semibold text-slate-500 mb-1">STEP 2</p>
              <p className="text-sm text-slate-700">各ページでCSVファイルを取り込み</p>
            </div>
            <div className="rounded-lg border border-slate-200 bg-slate-50 p-3">
              <p className="text-xs font-semibold text-slate-500 mb-1">STEP 3</p>
              <p className="text-sm text-slate-700">配当金や保有銘柄の情報を確認</p>
            </div>
          </div>
        </div>
      </div>
    </Layout>
  );
}
