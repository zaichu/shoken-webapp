import { lazy, Suspense } from 'react';
import { createBrowserRouter, RouterProvider, Navigate } from 'react-router-dom';
import { ErrorBoundary } from '../components/molecules/ErrorBoundary';
import { ErrorPage } from '../components/templates/ErrorPage';

const HomePage = lazy(() => import('../pages/Home').then(m => ({ default: m.HomePage })));
const SearchPage = lazy(() => import('../pages/Search').then(m => ({ default: m.SearchPage })));
const ReceiptsPage = lazy(() => import('../pages/Receipts').then(m => ({ default: m.ReceiptsPage })));
const AssetBalancePage = lazy(() => import('../pages/AssetBalance').then(m => ({ default: m.AssetBalancePage })));
const NotFoundPage = lazy(() => import('../pages/NotFound').then(m => ({ default: m.NotFoundPage })));
const LoginPage = lazy(() => import('../pages/Login').then(m => ({ default: m.LoginPage })));

const router = createBrowserRouter([
  {
    path: "/",
    element: <Suspense fallback={null}><HomePage /></Suspense>,
    errorElement: <ErrorPage />,
  },
  {
    path: "/search",
    element: <Suspense fallback={null}><SearchPage /></Suspense>,
  },
  {
    path: "/receipts",
    element: <Suspense fallback={null}><ReceiptsPage /></Suspense>,
  },
  {
    path: "/assetbalance",
    element: <Suspense fallback={null}><AssetBalancePage /></Suspense>,
  },
  {
    path: "/login",
    element: <Suspense fallback={null}><LoginPage /></Suspense>,
  },
  {
    path: "/404",
    element: <Suspense fallback={null}><NotFoundPage /></Suspense>,
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
