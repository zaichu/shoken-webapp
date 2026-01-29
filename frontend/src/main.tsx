import React from 'react';
import { createRoot } from 'react-dom/client';
import App from './App';
import './styles/tailwind.css';

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
