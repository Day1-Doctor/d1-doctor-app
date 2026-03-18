import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  // Prevent vite from obscuring Rust errors
  clearScreen: false,
  server: {
    port: 1420,
    // Tauri requires a consistent port
    strictPort: true,
    host: true,
  },
  // Env variables starting with TAURI_ will be exposed
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    // Tauri uses Chromium on Windows and WebKit on macOS/Linux
    target: process.env.TAURI_PLATFORM === "windows" ? "chrome105" : "safari14",
    // Produce sourcemaps for debug builds
    sourcemap: !!process.env.TAURI_DEBUG,
    // Don't minify for debug builds
    minify: !process.env.TAURI_DEBUG ? "esbuild" : false,
  },
});
