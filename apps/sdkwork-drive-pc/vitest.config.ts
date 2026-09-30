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
      // `react` alone is not enough. Inlined workspace deps (@sdkwork/ui-pc-react)
      // are symlinked cross-repo, and their own `node_modules/react` can point at
      // a *different* pnpm store with another React minor (observed: 19.2.4 from
      // the cloudrouter store vs the app's 19.2.8). react-dom then renders with
      // 19.2.8 while the component tree imported 19.2.4, so every useContext
      // reads null. Pin the whole react surface — including the subpath entries
      // the JSX runtime and the react-dom client go through — to this app's copy.
      react: path.resolve(__dirname, 'node_modules/react'),
      'react/jsx-runtime': path.resolve(__dirname, 'node_modules/react/jsx-runtime.js'),
      'react/jsx-dev-runtime': path.resolve(__dirname, 'node_modules/react/jsx-dev-runtime.js'),
      'react-dom': path.resolve(__dirname, 'node_modules/react-dom'),
      'react-dom/client': path.resolve(__dirname, 'node_modules/react-dom/client.js'),
      'react-dom/test-utils': path.resolve(__dirname, 'node_modules/react-dom/test-utils.js'),
      '@': path.resolve(__dirname, '.'),
      // Sibling workspace packages publish *raw TypeScript* through their
      // `exports` map (no built `dist`). The app's `node_modules/@sdkwork/*`
      // entries symlink straight into those sibling repos, which live outside
      // this Vite root — and Vite refuses to transform a module whose resolved
      // real path is outside `server.fs.allow`, reporting the confusing
      // "Failed to load url .../src/index.ts ... Does the file exist?" even
      // though the file is there. `vite.config.ts` already aliases the
      // credential/runtime entrypoints for the same reason; the test config
      // must stay in sync or those imports fail only under vitest.
      //
      // `@sdkwork/utils` ships a full `exports` map (`.` plus `./id`, `./money`,
      // `./path`, …). Aliasing only the bare specifier would break every
      // subpath import, so it is NOT aliased here; instead it is inlined and
      // the workspace root is added to `server.fs.allow` below, which lets
      // Vite resolve both the package and its subpaths through `exports`.
      '@sdkwork/auth-runtime-pc-react': path.resolve(
        iamRoot,
        'apps/sdkwork-iam-pc/packages/sdkwork-auth-runtime-pc-react/src/index.ts',
      ),
      '@sdkwork/iam-runtime': path.resolve(
        iamRoot,
        'apps/sdkwork-iam-common/packages/sdkwork-iam-runtime/src/index.ts',
      ),
      '@sdkwork/iam-contracts': path.resolve(
        iamRoot,
        'apps/sdkwork-iam-common/packages/sdkwork-iam-contracts/src/index.ts',
      ),
      '@sdkwork/iam-credential-entry': path.resolve(
        iamRoot,
        'apps/sdkwork-iam-common/packages/sdkwork-iam-credential-entry/src/index.ts',
      ),
      '@sdkwork/core-pc-react': path.resolve(__dirname, 'src/bootstrap/sdkworkCorePcReactShim.ts'),
    },
  },
  test: {
    server: {
      // Vite resolves the `@sdkwork/*` symlinks to their real sibling-repo
      // paths; allow the whole workspace so those raw-TS modules can be loaded.
      fs: {
        allow: [repoRoot, path.resolve(repoRoot, '..')],
      },
      deps: {
        inline: [
          /@radix-ui\/.*/,
          /@sdkwork\/ui-pc-react/,
          // Raw-TypeScript workspace packages must be inlined for the same
          // reason as the aliases above: externalized modules are handed to
          // native Node, which cannot execute `.ts`.
          /@sdkwork\/utils/,
          /@sdkwork\/iam-/,
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
