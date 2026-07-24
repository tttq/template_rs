import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },
  server: {
    port: 3000,
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:8080',
        changeOrigin: true,
      },
    },
  },
  build: {
    chunkSizeWarningLimit: 2500,
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (id.includes('node_modules')) {
            // antd 优先匹配（避免被 vue 的宽泛规则吞掉）
            if (id.includes('ant-design-vue') || id.includes('@ant-design')) {
              return 'antd-vendor'
            }
            // 精确匹配 vue 核心（用路径分隔符避免误匹配 *-vue-* 等）
            if (id.includes('/vue/') || id.includes('/vue-router/') || id.includes('/pinia/') || id.includes('/@vue/')) {
              return 'vue-vendor'
            }
            if (id.includes('/axios/') || id.includes('/dayjs/') || id.includes('/nprogress/')) {
              return 'utils-vendor'
            }
          }
        },
      },
    },
  },
})