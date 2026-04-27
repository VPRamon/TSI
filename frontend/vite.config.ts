import { defineConfig, loadEnv } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'path';

// https://vitejs.dev/config/
export default defineConfig(({ mode }) => {
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

  return {
    plugins: [react()],
    resolve: {
      alias: {
        '@': path.resolve(__dirname, './src'),
        '@root': path.resolve(__dirname, '../..'),
        'tsi-extensions-pack': extensionsPath,
      },
    },
    server: {
      port: 3000,
      proxy: {
        '/api': {
          target: 'http://localhost:8080',
          changeOrigin: true,
          rewrite: (path) => path.replace(/^\/api/, ''),
        },
      },
    },
  };
});
