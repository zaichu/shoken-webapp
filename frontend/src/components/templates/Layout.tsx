import { ReactNode } from 'react';
import { Header } from '../organisms/Header';

interface LayoutProps {
  children: ReactNode;
}

export function Layout({ children }: LayoutProps) {
  return (
    <div className="app-container">
      <Header />

      <main className="container mt-2 app-container-wide">
        {children}
      </main>

      {/* <Footer /> */}
    </div>
  );
}
