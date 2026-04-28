import { defineConfig, loadEnv, type PluginOption } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'path';

// https://vitejs.dev/config/
export default defineConfig(async ({ mode }) => {
  const env = loadEnv(mode, process.cwd(), '');
  // Path to the TSI extension pack module that exports `extensions: TsiExtensions`.
  // Defaults to the in-repo PhD/EST pack; integrators can point this at
  // their own pack via the `VITE_TSI_EXTENSIONS_PATH` env var
  // (absolute path, or relative to the frontend cwd).
  const extensionsPathRaw =
    env.VITE_TSI_EXTENSIONS_PATH ?? path.resolve(__dirname, '../../phd-extensions');
  const extensionsPath = path.isAbsolute(extensionsPathRaw)
    ? extensionsPathRaw
    : path.resolve(process.cwd(), extensionsPathRaw);

  const plugins: PluginOption[] = [react()];

  // Bundle visualizer is opt-in via `ANALYZE=1 npm run build` so it
  // never runs in CI/production builds by default.
  if (process.env.ANALYZE === '1') {
    const { visualizer } = await import('rollup-plugin-visualizer');
    plugins.push(
      visualizer({
        filename: 'dist/stats.html',
        gzipSize: true,
        brotliSize: true,
        template: 'treemap',
      }) as PluginOption
    );
  }

  return {
    plugins,
    resolve: {
      alias: {
        '@': path.resolve(__dirname, './src'),
        '@root': path.resolve(__dirname, '../..'),
        'tsi-extensions-pack': extensionsPath,
      },
    },
    build: {
      rollupOptions: {
        output: {
          // Stable vendor chunks so application code changes don't
          // invalidate the (large) third-party caches.
          manualChunks: {
            'plotly-basic': ['plotly.js-basic-dist-min'],
            'plotly-full': ['plotly.js-dist-min'],
            'react-vendor': ['react', 'react-dom', 'react-router-dom'],
            tanstack: ['@tanstack/react-query', 'react-window'],
          },
        },
      },
    },
    server: {
      port: 3000,
      proxy: {
        '/api': {
          target: 'http://localhost:8080',
          changeOrigin: true,
          rewrite: (p: string) => p.replace(/^\/api/, ''),
        },
      },
    },
  };
});
