import React from 'react';
import * as Sentry from '@sentry/react';
import { createRoot } from 'react-dom/client';
import App from './App';
import './styles/tailwind.css';

const sentryDsn = import.meta.env.VITE_SENTRY_DSN as string | undefined;

if (sentryDsn) {
  Sentry.init({ dsn: sentryDsn, sendDefaultPii: false });
}

const rootElement = document.getElementById('root');

if (!rootElement) {
  throw new Error('ルート要素が見つかりません');
}

const root = createRoot(rootElement);

root.render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
