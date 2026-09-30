# sdkwork-drive-pc-electron

Domain: drive
Capability: pc-electron
Package type: electron-host
Status: standard

This README is the SDKWork module entrypoint for `sdkwork-drive-pc-electron`. The machine-readable component contract is `specs/component.spec.json`; canonical standards are under `../../../../../sdkwork-specs/`.

## Public API

- `.`
- `./host`

## Host Profile

`clientArchitecture = "electron"`, `runtimeTarget = "desktop"`. This package owns exactly one host architecture; it `MUST NOT` carry `src-tauri/` or Capacitor scaffold, and it `MUST NOT` be merged with the Tauri host package.

Layout follows `DESKTOP_APP_ARCHITECTURE_SPEC.md` section 5.3:

```text
src/
  host/                     # renderer-side DesktopHost adapter
electron-builder.config.mjs
src-electron/
  main/                     # index.ts, window.ts, ipc.ts, secure-store.ts, tray.ts, shortcuts.ts, clipboard.ts
  preload/                  # index.ts (single contextBridge allowlist)
  shared/                   # ipc-channels.ts (protocol method table)
resources/
  icons/
```

## Bridge Protocol

Channels equal the protocol method names from `DESKTOP_APP_ARCHITECTURE_SPEC.md` section 5.6:

- `sdkwork:tray:setVisible` / `setMenu` / `restoreWindow`
- `sdkwork:shortcut:registerAll` / `unregister` / `unregisterAll`
- `sdkwork:clipboard:cutPaths` / `readPaths` / `writeText`
- `sdkwork:window:minimize` / `maximize` / `unmaximize` / `close` / `show`
- `sdkwork:shellOpen:openExternal`

Host-initiated events: `sdkwork:tray:menu`, `sdkwork:shortcut:triggered`, `sdkwork:deepLinks:open`.

The preload allowlist and the `ipcMain.handle` registry are both generated from `src-electron/shared/ipc-channels.ts`, so a method cannot be reachable without an explicit handler and unknown channels are rejected rather than forwarded.

## Security

Mandatory baseline, not parameterized:

- `contextIsolation = true`, `nodeIntegration = false`, `sandbox = true`, `webSecurity = true`.
- The preload exposes a method allowlist through `contextBridge.exposeInMainWorld`; it never exposes `ipcRenderer` or a generic pass-through.
- Production loads the packaged renderer only; `loadURL` to unapproved remote origins is refused.
- Tokens use `safeStorage` (OS-backed) inside the app user-data directory.
- Signing keys, entitlements references, and updater publish config stay out of source control.

## Renderer Binding

`electron-builder.config.mjs` `extraResources` points at the root PC app build output (`../../../dist`); the renderer tree is never copied into this package. The dev server URL is read from `ELECTRON_START_URL` and `MUST` equal the root PC renderer dev server port.

## Verification

- `pnpm --filter sdkwork-drive-pc-electron typecheck`

## Owner And Status

Owner and lifecycle status are tracked in `specs/component.spec.json`. Update that contract before changing public integration behavior.
