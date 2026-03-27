import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

const getMonacoChunkName = (id: string) => {
  const marker = 'node_modules/monaco-editor/esm/';
  if (!id.includes(marker)) {
    return null;
  }

  const relative = id.split(marker)[1] ?? '';
  if (relative.includes('vs/basic-languages/')) return 'monaco-basic-languages';
  if (relative.includes('vs/language/json/')) return 'monaco-language-json';
  if (relative.includes('vs/language/css/')) return 'monaco-language-css';
  if (relative.includes('vs/language/html/')) return 'monaco-language-html';
  if (relative.includes('vs/language/typescript/')) return 'monaco-language-typescript';
  if (relative.includes('vs/editor/contrib/')) {
    const contribMatch = relative.match(/vs\/editor\/contrib\/([^/]+)/);
    if (contribMatch?.[1]) {
      return `monaco-contrib-${contribMatch[1]}`;
    }
    return 'monaco-editor-contrib';
  }
  if (relative.includes('vs/editor/common/')) {
    const commonMatch = relative.match(/vs\/editor\/common\/([^/]+)/);
    if (commonMatch?.[1]) {
      return `monaco-editor-common-${commonMatch[1]}`;
    }
    return 'monaco-editor-common';
  }
  if (relative.includes('vs/editor/browser/')) {
    const browserMatch = relative.match(/vs\/editor\/browser\/([^/]+)/);
    if (browserMatch?.[1]) {
      return `monaco-editor-browser-${browserMatch[1]}`;
    }
    return 'monaco-editor-browser';
  }
  if (relative.includes('vs/editor/standalone/')) return 'monaco-editor-standalone';
  if (relative.includes('vs/editor/')) return 'monaco-editor-misc';
  if (relative.includes('vs/base/common/')) return 'monaco-base-common';
  if (relative.includes('vs/base/browser/')) return 'monaco-base-browser';
  if (relative.includes('vs/base/worker/')) return 'monaco-base-worker';
  if (relative.includes('vs/base/parts/')) return 'monaco-base-parts';
  if (relative.includes('vs/base/node/')) return 'monaco-base-node';
  if (relative.includes('vs/base/')) return 'monaco-base-misc';
  if (relative.includes('vs/platform/')) return 'monaco-platform';
  if (relative.includes('vs/')) return 'monaco-misc';
  return 'monaco';
};

const manualChunks = (id: string) => {
  if (!id.includes('node_modules')) {
    if (id.includes('/src/components/DetailView/')) return 'detail-view';
    return undefined;
  }

  const monacoChunk = getMonacoChunkName(id);
  if (monacoChunk) return monacoChunk;

  if (id.includes('@monaco-editor/react')) return 'monaco-react';
  if (id.includes('react-json-view-lite')) return 'json-preview-vendor';
  if (id.includes('@tauri-apps')) return 'tauri-vendor';
  if (id.includes('@radix-ui')) return 'radix-vendor';
  if (id.includes('@tanstack')) return 'tanstack-vendor';
  if (id.includes('react-i18next') || id.includes('i18next')) return 'i18n-vendor';
  if (id.includes('lucide-react')) return 'icon-vendor';
  if (
    id.includes('/react/') ||
    id.includes('/react-dom/') ||
    id.includes('/scheduler/') ||
    id.includes('use-sync-external-store')
  ) {
    return 'react-vendor';
  }

  return 'vendor';
};

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [react()],
  build: {
    rollupOptions: {
      output: {
        manualChunks,
      },
    },
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: 'ws',
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ['**/src-tauri/**'],
    },
  },
}));
