import { ReactNode } from 'react';
import { Header } from '../organisms/Header';

interface LayoutProps {
  children: ReactNode;
}

export function Layout({ children }: LayoutProps) {
  return (
    <div className="min-h-screen flex flex-col">
      <Header />

      <main className="mx-auto w-full max-w-[1600px] px-4 py-4 flex-1">
        {children}
      </main>

      {/* <Footer /> */}
    </div>
  );
}
