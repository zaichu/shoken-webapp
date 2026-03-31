import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import tailwindcss from '@tailwindcss/vite';
import { dirname, resolve } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));

export default defineConfig({
  plugins: [
    react({
      babel: {
        plugins: [
          // Vitest / Playwright 実行時は React Compiler を無効化（dev/prod のみ有効）
          ...(process.env.VITEST || process.env.PLAYWRIGHT_TEST ? [] : [['babel-plugin-react-compiler', {}] as const]),
        ],
      },
    }),
    tailwindcss(),
  ],
  base: '/',
  server: {
    port: 8080,
    open: !process.env.CI && !process.env.PLAYWRIGHT_TEST,
  },
  build: {
    outDir: 'dist',
    sourcemap: true,
    minify: 'terser',
    terserOptions: {
      compress: {
        passes: 3,
        drop_console: true,
      },
      mangle: {
        toplevel: true,
      },
    },
    rollupOptions: {
      output: {
        manualChunks: (id: string) => {
          if (
            id.includes('/node_modules/react/') ||
            id.includes('/node_modules/react-dom/') ||
            id.includes('/node_modules/react-router/')  ||
            id.includes('/node_modules/react-router-dom/')
          ) {
            return 'vendor';
          }
          if (id.includes('/node_modules/@tanstack/')) {
            return 'tanstack';
          }
        },
      },
    },
  },
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: './src/test/setup.ts',
    testTimeout: 10000,
    hookTimeout: 10000,
    clearMocks: true,
    restoreMocks: true,
    retry: 1,
    exclude: ['node_modules', 'dist', 'e2e/**'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      exclude: ['node_modules/', 'src/test/'],
      thresholds: {
        lines: 70,
      },
    },
    pool: 'forks',
    maxWorkers: 1,
  },
});
