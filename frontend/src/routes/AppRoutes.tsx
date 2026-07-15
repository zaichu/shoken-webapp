import { lazy, Suspense, type ReactNode } from 'react';
import { createBrowserRouter, RouterProvider, Navigate } from 'react-router-dom';
import { ErrorBoundary } from '../components/molecules/ErrorBoundary';
import { ErrorPage } from '../components/templates/ErrorPage';
import { Spinner } from '../components/atoms/Spinner';
import { useAuth } from '@/features/auth/hooks/useAuth';
import { ROUTE_PATHS } from './routePaths';

const HomePage = lazy(() => import('../pages/Home').then(m => ({ default: m.HomePage })));
const SearchPage = lazy(() => import('../pages/Search').then(m => ({ default: m.SearchPage })));
const ReceiptsPage = lazy(() => import('../pages/Receipts').then(m => ({ default: m.ReceiptsPage })));
const AssetBalancePage = lazy(() => import('../pages/AssetBalance').then(m => ({ default: m.AssetBalancePage })));
const NotFoundPage = lazy(() => import('../pages/NotFound').then(m => ({ default: m.NotFoundPage })));
const LoginPage = lazy(() => import('../pages/Login').then(m => ({ default: m.LoginPage })));

function PageLoader() {
  return (
    <div className="flex items-center justify-center min-h-screen">
      <Spinner size="lg" />
    </div>
  );
}

function ProtectedRoute({ children }: { children: ReactNode }) {
  const { isAuthenticated, isLoading } = useAuth();

  if (isLoading) {
    return <PageLoader />;
  }

  if (!isAuthenticated) {
    return <Navigate to={ROUTE_PATHS.login} replace />;
  }

  return children;
}

const router = createBrowserRouter([
  {
    path: ROUTE_PATHS.home,
    element: <Suspense fallback={<PageLoader />}><HomePage /></Suspense>,
    errorElement: <ErrorPage />,
  },
  {
    // 銘柄検索は未ログインでも利用できる公開ルート。
    path: ROUTE_PATHS.search,
    element: <Suspense fallback={<PageLoader />}><SearchPage /></Suspense>,
  },
  {
    path: ROUTE_PATHS.receipts,
    element: (
      <ProtectedRoute>
        <Suspense fallback={<PageLoader />}><ReceiptsPage /></Suspense>
      </ProtectedRoute>
    ),
  },
  {
    path: ROUTE_PATHS.assetBalance,
    element: (
      <ProtectedRoute>
        <Suspense fallback={<PageLoader />}><AssetBalancePage /></Suspense>
      </ProtectedRoute>
    ),
  },
  {
    path: ROUTE_PATHS.login,
    element: <Suspense fallback={<PageLoader />}><LoginPage /></Suspense>,
  },
  {
    path: ROUTE_PATHS.notFound,
    element: <Suspense fallback={<PageLoader />}><NotFoundPage /></Suspense>,
  },
  {
    path: "*",
    element: <Navigate to={ROUTE_PATHS.notFound} replace />,
  },
]);

export function AppRoutes() {
  return (
    <ErrorBoundary fallback={<ErrorPage />}>
      <RouterProvider router={router} />
    </ErrorBoundary>
  );
}
