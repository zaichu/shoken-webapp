import { ReactNode } from 'react';
import { Header } from '../organisms/Header';

interface LayoutProps {
  children: ReactNode;
}

export function Layout({ children }: LayoutProps) {
  return (
    <div className="min-h-screen flex flex-col bg-slate-50">
      <Header />

      <main className="mx-auto w-full max-w-[1440px] px-4 sm:px-5 lg:px-6 py-4 flex-1 flex flex-col">
        {children}
      </main>

      {/* <Footer /> */}
    </div>
  );
}
