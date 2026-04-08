import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
      '@siderust/siderust-web': path.resolve(__dirname, './node_modules/siderust-js/siderust-web'),
      '@siderust/tempoch-web': path.resolve(__dirname, './node_modules/siderust-js/tempoch-js/tempoch-web'),
      '@siderust/qtty-web': path.resolve(__dirname, './node_modules/siderust-js/qtty-js/qtty-web'),
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
})
