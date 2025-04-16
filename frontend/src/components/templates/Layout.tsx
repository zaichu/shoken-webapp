import { ReactNode, useState } from 'react';
import { Header } from '../organisms/Header';
import { Footer } from '../organisms/Footer';

interface LayoutProps {
  children: ReactNode;
}

export function Layout({ children }: LayoutProps) {
  const [theme, setTheme] = useState<'light' | 'dark'>('light');
  const toggleTheme = () => {
    setTheme(prevTheme => prevTheme === 'light' ? 'dark' : 'light');
    if (theme === 'light') {
      document.documentElement.setAttribute('data-bs-theme', 'dark');
    } else {
      document.documentElement.setAttribute('data-bs-theme', 'light');
    }
  };

  return (
    <div className={`app-container ${theme}`}>
      <Header onToggleTheme={toggleTheme} />

      <main className="container mt-4" style={{ maxWidth: '1600px' }}>
        {children}
      </main>

      {/* <Footer /> */}
    </div>
  );
}
