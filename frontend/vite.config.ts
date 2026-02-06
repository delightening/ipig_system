import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  server: {
    port: 5173,
    host: '0.0.0.0', // Ensure it binds to all interfaces for Docker
    proxy: {
      '/api': {
        // In Docker, 'api' is the service name. Locally, it should be 'localhost'.
        // We use an environment variable VITE_API_URL or fallback.
        target: process.env.VITE_API_URL || 'http://localhost:8000',
        changeOrigin: true,
      },
    },
  },
})
