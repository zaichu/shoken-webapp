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
      <Header />

      <main className={`${APP_SHELL_CONTAINER} flex flex-1 flex-col py-4`}>
        {children}
      </main>

      <Footer />
    </div>
  );
}
