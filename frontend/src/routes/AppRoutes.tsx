import { lazy, Suspense, type ReactNode } from 'react';
import { createBrowserRouter, RouterProvider, Navigate } from 'react-router-dom';
import { ErrorBoundary } from '../components/molecules/ErrorBoundary';
import { ErrorPage } from '../components/templates/ErrorPage';
import { Spinner } from '../components/atoms/Spinner';
import { useAuth } from '@/features/auth/hooks/useAuth';

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
    return <Navigate to="/login" replace />;
  }

  return children;
}

const router = createBrowserRouter([
  {
    path: "/",
    element: <Suspense fallback={<PageLoader />}><HomePage /></Suspense>,
    errorElement: <ErrorPage />,
  },
  {
    path: "/search",
    element: <Suspense fallback={<PageLoader />}><SearchPage /></Suspense>,
  },
  {
    path: "/receipts",
    element: (
      <ProtectedRoute>
        <Suspense fallback={<PageLoader />}><ReceiptsPage /></Suspense>
      </ProtectedRoute>
    ),
  },
  {
    path: "/assetbalance",
    element: (
      <ProtectedRoute>
        <Suspense fallback={<PageLoader />}><AssetBalancePage /></Suspense>
      </ProtectedRoute>
    ),
  },
  {
    path: "/login",
    element: <Suspense fallback={<PageLoader />}><LoginPage /></Suspense>,
  },
  {
    path: "/404",
    element: <Suspense fallback={<PageLoader />}><NotFoundPage /></Suspense>,
  },
  {
    path: "*",
    element: <Navigate to="/404" replace />,
  },
]);

export function AppRoutes() {
  return (
    <ErrorBoundary fallback={<ErrorPage />}>
      <RouterProvider router={router} />
    </ErrorBoundary>
  );
}
