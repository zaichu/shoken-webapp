import { ReactNode } from 'react';
import { Header } from '../organisms/Header';

interface LayoutProps {
  children: ReactNode;
}

export function Layout({ children }: LayoutProps) {
  return (
    <div className="min-h-screen flex flex-col bg-slate-50">
      <Header />

      <main className="mx-auto w-full max-w-7xl px-4 sm:px-6 lg:px-8 py-6 flex-1">
        {children}
      </main>

      {/* <Footer /> */}
    </div>
  );
}
