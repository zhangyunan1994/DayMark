import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: { '@': import.meta.dirname + '/src' },
  },
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
  },
})
