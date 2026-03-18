import { ReactNode } from 'react';
import { Footer } from '../organisms/Footer';
import { Header } from '../organisms/Header';
import { APP_SHELL_CONTAINER } from '@/lib/layout';

interface LayoutProps {
  children: ReactNode;
}

export function Layout({ children }: LayoutProps) {
  return (
    <div className="min-h-screen flex flex-col bg-slate-50">
      {/* スキップリンク: キーボード・スクリーンリーダー利用者向け */}
      <a
        href="#main-content"
        className="sr-only focus:not-sr-only focus:fixed focus:top-2 focus:left-2 focus:z-50 focus:rounded focus:bg-white focus:px-4 focus:py-2 focus:text-sm focus:font-semibold focus:shadow-lg focus:outline-2 focus:outline-primary"
      >
        メインコンテンツへスキップ
      </a>

      <Header />

      <main id="main-content" className={`${APP_SHELL_CONTAINER} flex flex-1 flex-col py-4`}>
        {children}
      </main>

      <Footer />
    </div>
  );
}
