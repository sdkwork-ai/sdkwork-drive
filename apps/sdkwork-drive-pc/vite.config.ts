import tailwindcss from '@tailwindcss/vite';
import react from '@vitejs/plugin-react';
import path from 'path';
import { defineConfig, loadEnv } from 'vite';

const DEFAULT_APP_API_PROXY_TARGET = 'http://127.0.0.1:3900';
const DEFAULT_ADMIN_API_PROXY_TARGET = 'http://127.0.0.1:18083';

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, __dirname, '');
  const repoRoot = path.resolve(__dirname, '../..');
  const workspaceDependencyRoot = (dependencyId: string) =>
    path.resolve(repoRoot, '..', dependencyId);
  const appbaseRoot = process.env.SDKWORK_APPBASE_ROOT ?? workspaceDependencyRoot('sdkwork-appbase');
  const iamRoot = process.env.SDKWORK_IAM_ROOT ?? workspaceDependencyRoot('sdkwork-iam');
  const sdkCommonsRoot = process.env.SDKWORK_SDK_COMMONS_ROOT ?? workspaceDependencyRoot('sdkwork-sdk-commons');
  const utilsRoot = process.env.SDKWORK_UTILS_ROOT ?? workspaceDependencyRoot('sdkwork-utils');
  const uiRoot = process.env.SDKWORK_UI_PC_REACT_ROOT
    ?? path.resolve(workspaceDependencyRoot('sdkwork-ui'), 'sdkwork-ui-pc-react');
  const appApiProxyTarget =
    process.env.SDKWORK_DRIVE_DEV_APP_API_PROXY_TARGET
    || env.VITE_DRIVE_PC_PLATFORM_API_GATEWAY_HTTP_URL
    || env.VITE_DRIVE_PC_APP_API_BASE_URL
    || DEFAULT_APP_API_PROXY_TARGET;
  const adminApiProxyTarget =
    process.env.SDKWORK_DRIVE_DEV_ADMIN_API_PROXY_TARGET
    || env.VITE_DRIVE_PC_DRIVE_ADMIN_STORAGE_API_BASE_URL
    || env.VITE_DRIVE_PC_BACKEND_API_BASE_URL
    || DEFAULT_ADMIN_API_PROXY_TARGET;

  return {
    define: {
      'process.env.SDKWORK_ACCESS_TOKEN': JSON.stringify(env.SDKWORK_ACCESS_TOKEN ?? ''),
    },
            plugins: [react(), tailwindcss()],
    resolve: {
      alias: {
        '@': path.resolve(__dirname, '.'),
        react: path.resolve(__dirname, 'node_modules/react'),
      },
      dedupe: ['react', 'react-dom'],
    },
    server: {
      host: 'localhost',
      port: 5183,
      strictPort: true,
      hmr: process.env.DISABLE_HMR !== 'true',
      watch: process.env.DISABLE_HMR === 'true' ? null : {},
      proxy: {
        '/app/v3/api': {
          target: appApiProxyTarget,
          changeOrigin: true,
        },
        '/admin/v3/api': {
          target: adminApiProxyTarget,
          changeOrigin: true,
        },
        '/backend/v3/api': {
          target: adminApiProxyTarget,
          changeOrigin: true,
        },
      },
    },
  };
});
