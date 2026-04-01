import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

const getMonacoChunkName = (id: string) => {
  if (id.includes('node_modules/monaco-editor/') || id.includes('@monaco-editor/react')) {
    return 'monaco-vendor';
  }

  return null;
};

const manualChunks = (id: string) => {
  if (!id.includes('node_modules')) {
    if (id.includes('/src/components/DetailView/')) return 'detail-view';
    return undefined;
  }

  const monacoChunk = getMonacoChunkName(id);
  if (monacoChunk) return monacoChunk;

  if (id.includes('react-json-view-lite')) return 'json-preview-vendor';
  if (id.includes('@tauri-apps')) return 'tauri-vendor';
  // Keep Floating UI with Radix so React-related popper modules do not
  // create a cross-chunk cycle with the generic vendor chunk in production.
  if (id.includes('@floating-ui/')) return 'radix-vendor';
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
