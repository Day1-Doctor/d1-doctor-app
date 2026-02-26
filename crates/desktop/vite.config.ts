import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

export default defineConfig({
  plugins: [vue()],
  root: resolve(__dirname, 'src'),
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
  build: {
    target: ['es2021', 'chrome105', 'safari15'],
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG,
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'src/index.html'),
        ninja: resolve(__dirname, 'src/ninja.html'),
      },
    },
    outDir: '../dist',
    emptyOutDir: true,
  },
  test: {
    environment: 'jsdom',
    globals: true,
    root: resolve(__dirname, 'src'),
  },
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },
})
