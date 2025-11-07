import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react-swc'
/// <reference types="vite/client" />

// https://vite.dev/config/
export default defineConfig({
  server: {
    host: true,
    port: 5173,
    proxy: {
      "/api": {
        target: "https://apiwhatanime.sibbeeegold.dev",
        changeOrigin: true,
        secure: true,
        rewrite: (path) => path.replace(/^\/api/, ""),
      }
    },
    allowedHosts: ['whatanime.ddns.net', 'whatanime.sibbeeegold.dev'],
  },
  plugins: [react()],
})
