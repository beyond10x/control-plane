import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
export default defineConfig({ plugins: [vue()], build: { sourcemap: false, rollupOptions: { output: { entryFileNames: 'app.js', chunkFileNames: '[name].js', assetFileNames: '[name][extname]' } } } })
