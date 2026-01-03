import { createBrowserRouter, RouterProvider, Navigate } from 'react-router-dom';
import { ErrorBoundary } from '../components/molecules/ErrorBoundary';
import { ErrorPage } from '../components/templates/ErrorPage';
import { HomePage } from '../pages/Home';
import { SearchPage } from '../pages/Search';
import { ReceiptsPage } from '../pages/Receipts';
import { AssetBalancePage } from '../pages/AssetBalance';
import { NotFoundPage } from '../pages/NotFound';

const router = createBrowserRouter([
  {
    path: "/",
    element: <HomePage />,
    errorElement: <ErrorPage />,
  },
  {
    path: "/search",
    element: <SearchPage />,
  },
  {
    path: "/receipts",
    element: <ReceiptsPage />,
  },
  {
    path: "/assetbalance",
    element: <AssetBalancePage />,
  },
  {
    path: "/404",
    element: <NotFoundPage />,
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
