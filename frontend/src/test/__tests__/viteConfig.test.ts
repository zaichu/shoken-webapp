import { readFileSync } from 'fs';
import { dirname, resolve } from 'path';
import { fileURLToPath, pathToFileURL } from 'url';
import { describe, expect, it } from 'vitest';

const currentDir = dirname(fileURLToPath(import.meta.url));
const configPath = resolve(currentDir, '../../../vite.config.ts');

type MinimalViteConfig = {
  resolve?: {
    alias?: Record<string, string>;
  };
  build?: {
    rollupOptions?: {
      output?:
        | {
            manualChunks?: unknown;
          }
        | Array<{
            manualChunks?: unknown;
          }>;
    };
  };
};

async function loadViteConfig() {
  const source = readFileSync(configPath, 'utf8');
  const originalUint8Array = globalThis.Uint8Array;
  const encoderUint8Array = new TextEncoder().encode('').constructor as typeof Uint8Array;

  try {
    Object.defineProperty(globalThis, 'Uint8Array', {
      configurable: true,
      value: encoderUint8Array,
      writable: true,
    });
    const executableSource = source
      .replace(/import\s+\{\s*defineConfig\s*\}\s+from\s+['"]vitest\/config['"];?\r?\n?/g, '')
      .replace(/import\s+react\s+from\s+['"]@vitejs\/plugin-react['"];?\r?\n?/g, '')
      .replace(/import\s+tailwindcss\s+from\s+['"]@tailwindcss\/vite['"];?\r?\n?/g, '')
      .replace(/import\s+\{\s*[^}]*\bresolve\b[^}]*\}\s+from\s+['"]path['"];?\r?\n?/g, '')
      .replace(/import\s+\{\s*fileURLToPath\s*\}\s+from\s+['"]url['"];?\r?\n?/g, '')
      .replace(/\s+as const/g, '')
      .replace(/id: string/g, 'id')
      .replace(/import\.meta/g, 'import_meta')
      .replace('export default defineConfig(', 'return defineConfig(');

    const evaluateConfig = new Function(
      'defineConfig',
      'react',
      'tailwindcss',
      'resolve',
      'dirname',
      'fileURLToPath',
      'process',
      'importMetaUrl',
      `
        "use strict";
        const import_meta = { url: importMetaUrl };
        ${executableSource}
      `,
    ) as (
      defineConfig: <T>(config: T) => T,
      react: (...args: unknown[]) => unknown,
      tailwindcss: (...args: unknown[]) => unknown,
      resolve: typeof import('path').resolve,
      dirname: typeof import('path').dirname,
      fileURLToPath: typeof import('url').fileURLToPath,
      process: NodeJS.Process,
      importMetaUrl: string,
    ) => MinimalViteConfig;

    return evaluateConfig(
      <T>(config: T) => config,
      () => ({ name: 'react' }),
      () => ({ name: 'tailwindcss' }),
      resolve,
      dirname,
      fileURLToPath,
      process,
      pathToFileURL(configPath).href,
    );
  } finally {
    Object.defineProperty(globalThis, 'Uint8Array', {
      configurable: true,
      value: originalUint8Array,
      writable: true,
    });
  }
}

describe('vite.config.ts', () => {
  it('ESM として読み込めて @ エイリアスを src に解決する', async () => {
    const config = await loadViteConfig();

    expect(config.resolve?.alias).toMatchObject({
      '@': resolve(currentDir, '../../../src'),
    });
  });

  it('manualChunks で vendor チャンクを関数で振り分ける', async () => {
    const config = await loadViteConfig();
    const output = Array.isArray(config.build?.rollupOptions?.output)
      ? config.build.rollupOptions.output[0]
      : config.build?.rollupOptions?.output;
    const { manualChunks } = output ?? {};

    expect(typeof manualChunks).toBe('function');

    if (typeof manualChunks !== 'function') {
      throw new Error('manualChunks が関数ではありません');
    }

    expect(manualChunks('/node_modules/react/index.js')).toBe('vendor');
    expect(manualChunks('/node_modules/react-dom/index.js')).toBe('vendor');
    expect(manualChunks('/node_modules/react-router-dom/index.js')).toBe('vendor');
    expect(manualChunks('/src/main.tsx')).toBeUndefined();
  });
});
