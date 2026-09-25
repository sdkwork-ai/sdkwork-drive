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
    // Vite's server target defaults to `mainFields: ['main']`, which picks the
    // CJS bundle of packages that publish `main` + `module` without an
    // `exports` map. Those CJS bundles `require('react')` through native Node,
    // which resolves from the dependency's own store — a second React instance
    // that no Vite-level alias or dedupe can reach, and every hook call in a
    // rendered component then reads `null`. preferring `module` turns those
    // imports into ES imports that the resolver does own.
    mainFields: ['module', 'main'],
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
          // lucide-react has no `exports` map, so it must both resolve to ESM
          // (mainFields above) and be inlined; externalized modules are handed
          // to native Node and bypass the resolver entirely.
          /lucide-react/,
          /react-remove-scroll.*/,
          /react-style-singleton/,
          /use-callback-ref/,
          /use-sidecar/,
        ],
      },
    },
  },
});
