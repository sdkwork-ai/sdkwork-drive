import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { defineConfig } from 'vitest/config';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, '../..');
const workspaceDependencyRoot = (dependencyId: string) =>
  path.resolve(repoRoot, '..', dependencyId);
const appbaseRoot = process.env.SDKWORK_APPBASE_ROOT ?? workspaceDependencyRoot('sdkwork-appbase');
const iamRoot = process.env.SDKWORK_IAM_ROOT ?? workspaceDependencyRoot('sdkwork-iam');
const sdkCommonsRoot = process.env.SDKWORK_SDK_COMMONS_ROOT ?? workspaceDependencyRoot('sdkwork-sdk-commons');
const utilsRoot = process.env.SDKWORK_UTILS_ROOT ?? workspaceDependencyRoot('sdkwork-utils');
const uiRoot = process.env.SDKWORK_UI_PC_REACT_ROOT
  ?? path.resolve(workspaceDependencyRoot('sdkwork-ui'), 'sdkwork-ui-pc-react');

export default defineConfig({
  resolve: {
    dedupe: ['react', 'react-dom'],
    alias: {
      react: path.resolve(__dirname, 'node_modules/react'),
      '@': path.resolve(__dirname, '.'),
    },
  },
  test: {
    server: {
      deps: {
        inline: [
          /@radix-ui\/.*/,
          /@sdkwork\/ui-pc-react/,
          /react-remove-scroll.*/,
          /react-style-singleton/,
          /use-callback-ref/,
          /use-sidecar/,
        ],
      },
    },
  },
});
