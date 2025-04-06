import { ReactNode } from 'react';
import { Link } from 'react-router-dom';
// import { useUserInfo } from '../context/UserContext';

interface LayoutProps {
  children: ReactNode;
}

export const Layout = ({ children }: LayoutProps) => {
  // const { userInfo } = useUserInfo();

  return (
    <>
      <nav className="navbar bg-dark navbar-expand-lg bg-body-tertiary" data-bs-theme="dark">
        <div className="container-fluid" style={{ maxWidth: '1600px' }}>
          <Link className="navbar-brand" to="/shoken-webapp/">証券Web</Link>
          <button
            className="navbar-toggler"
            type="button"
            data-bs-toggle="collapse"
            data-bs-target="#navbarNav"
            aria-controls="navbarNav"
            aria-expanded="false"
            aria-label="Toggle navigation"
          >
            <span className="navbar-toggler-icon"></span>
          </button>
          <div className="collapse navbar-collapse" id="navbarNav">
            <ul className="nav navbar-nav nav-underline justify-content-center">
              <li className="nav-item">
                <Link className="nav-link" to="/shoken-webapp/search">銘柄検索</Link>
              </li>
              <li className="nav-item">
                <Link className="nav-link" to="/shoken-webapp/receipts">受取金</Link>
              </li>
            </ul>
          </div>
        </div>
      </nav>

      <main className="container mt-4" style={{ maxWidth: '1600px' }}>
        {children}
      </main>

      <footer className="bg-light text-center text-lg-start mt-4">
        <div className="container p-1">
          <p className="text-center mt-3">© 2024 証券Web</p>
        </div>
      </footer>
    </>
  );
}
