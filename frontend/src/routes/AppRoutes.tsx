import { createBrowserRouter, RouterProvider, Navigate } from 'react-router-dom';
import { ErrorBoundary } from '../components/molecules/ErrorBoundary';
import { ErrorPage } from '../components/templates/ErrorPage';
import { HomePage } from '../pages/Home';
import { SearchPage } from '../pages/Search';
import { ReceiptsPage } from '../pages/Receipts';
import { HoldingsPage } from '../pages/HoldingsPage';
import { NotFoundPage } from '../pages/NotFound';

const router = createBrowserRouter([
  {
    path: "/shoken-webapp/",
    element: <HomePage />,
    errorElement: <ErrorPage />,
  },
  {
    path: "/shoken-webapp/search",
    element: <SearchPage />,
  },
  {
    path: "/shoken-webapp/receipts",
    element: <ReceiptsPage />,
  },
  {
    path: "/shoken-webapp/holdings",
    element: <HoldingsPage />,
  },
  {
    path: "/shoken-webapp/404",
    element: <NotFoundPage />,
  },
  {
    path: "*",
    element: <Navigate to="/shoken-webapp/404" replace />,
  },
]);

export function AppRoutes() {
  return (
    <ErrorBoundary fallback={<ErrorPage />}>
      <RouterProvider router={router} />
    </ErrorBoundary>
  );
}
