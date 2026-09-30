/**
 * electron-builder configuration for the SDKWork Drive PC Electron host.
 *
 * The renderer is NOT copied into this package: `files`/`extraResources` point
 * at the root PC app build output (`apps/sdkwork-drive-pc/dist`), so Tauri and
 * Electron ship the same renderer artifact.
 *
 * Bundle metadata (productName, appId, artifactName, icon) mirrors
 * `src-tauri/tauri.conf.json` and `sdkwork.app.config.json`.
 */

import { existsSync, readdirSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const PACKAGE_DIR = dirname(fileURLToPath(import.meta.url));
// `packages/sdkwork-drive-pc-electron` -> `apps/sdkwork-drive-pc/dist`.
const APP_DIST = resolve(PACKAGE_DIR, '../../dist');

/**
 * The Vite build is environment-scoped (`dist/<deploymentProfile>/<environment>/`),
 * so the renderer is never at `dist/index.html`. Resolve the directory that
 * actually contains an `index.html` and fail loudly when there is none: shipping
 * a renderer-less bundle produces an app that starts to a blank window, which is
 * far harder to diagnose from a release artifact than a packaging error.
 */
function resolveRendererDir(root) {
  if (!existsSync(root)) {
    throw new Error(
      `Renderer build output missing at ${root}; run the PC app build (pnpm build) before packaging the Electron host.`,
    );
  }
  const queue = [root];
  while (queue.length > 0) {
    const current = queue.shift();
    if (existsSync(join(current, 'index.html'))) {
      return current;
    }
    for (const entry of readdirSync(current, { withFileTypes: true })) {
      if (entry.isDirectory()) {
        queue.push(join(current, entry.name));
      }
    }
  }
  throw new Error(`No index.html found under ${root}; the renderer bundle is incomplete.`);
}

const RENDERER_DIR = resolveRendererDir(APP_DIST);

/** @type {import('electron-builder').Configuration} */
const config = {
  appId: 'com.sdkwork.drive.pc',
  productName: 'SDKWork Drive',
  copyright: 'Copyright © SDKWork',
  directories: {
    // Two levels up is `apps/sdkwork-drive-pc/`; three would land in the shared
    // `apps/` directory and scatter packaging output across the app family.
    output: '../../dist-electron',
    buildResources: 'resources',
  },
  // The main/preload tree is emitted to `dist-electron/` by
  // `tsconfig.build.json`; shipping raw `src-electron/**.ts` would leave
  // Electron unable to resolve `main`.
  files: [
    'dist-electron/src-electron/**/*.js',
    'package.json',
  ],
  extraResources: [
    {
      from: RENDERER_DIR,
      to: 'renderer',
      filter: ['**/*'],
    },
    // The main process resolves the tray icon from `<resources>/icons`; `files`
    // only ships compiled JS, so the asset needs an explicit copy.
    {
      from: 'resources/icons',
      to: 'icons',
      filter: ['*'],
    },
  ],
  win: {
    target: ['nsis', 'msi'],
    icon: 'resources/icons/icon.ico',
    publisherName: 'SDKWork',
    artifactName: 'SDKWork-Drive-${version}-${arch}-setup.${ext}',
  },
  mac: {
    target: ['dmg', 'zip'],
    icon: 'resources/icons/icon.icns',
    category: 'public.app-category.productivity',
    artifactName: 'SDKWork-Drive-${version}-${arch}.${ext}',
  },
  linux: {
    target: ['AppImage', 'deb'],
    icon: 'resources/icons',
    category: 'Utility',
    artifactName: 'SDKWork-Drive-${version}-${arch}.${ext}',
  },
  asar: true,
  /**
   * `npmRebuild` runs `@electron/rebuild` against the packaged dependency tree
   * to recompile native modules against Electron's ABI. This host has no native
   * dependencies (only `electron`, `electron-builder` and TypeScript), so the
   * step can only fail — and it does: on Windows pnpm leaves a *dangling*
   * `.pnpm/node_modules/fsevents` symlink (fsevents is macOS-only and is never
   * installed here), which the rebuild walker `stat`s and crashes on with
   * `ENOENT`. electron-builder offers no per-platform opt-out, and the failure
   * happens before any target is produced, so rebuild is disabled.
   *
   * Re-enable this only alongside an actual native dependency; at that point the
   * macOS runner will need the rebuild anyway.
   */
  npmRebuild: false,
  // Signing keys and entitlements references stay out of source control; the
  // release pipeline injects them through environment variables.
  publish: null,
};

export default config;
