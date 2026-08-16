# Drive Sandbox Explorer

Reusable PC React explorer for a Drive-owned, server-authorized sandbox or a host-local adapter.

The package accepts an injected `SandboxExplorerPort`; it never constructs SDK clients,
reads credentials, or receives a server physical path. Hosts must configure it before rendering.

```ts
configureDriveSandboxExplorerRuntime({ port });
```

Use `SandboxDirectoryPickerView` for project import flows and `SandboxExplorerView` for
interactive file-space management.

`SandboxExplorerView` follows desktop file-browser interaction: click selects, double-click or
`Enter` opens, `Alt+Left`/`Alt+Right` navigates history, `Backspace` moves up, `F5` refreshes,
`F2` renames, `Delete` removes, and `Ctrl`/`Cmd+F` focuses the loaded-item filter. Manage mode
supports create, UTF-8 text editing, Base64 read-only binary preview, rename, move, recursive
directory deletion, details/grid views, and a context menu. Select-directory mode is intentionally
non-mutating and exposes only browsing plus directory confirmation.

Directory items use one roving tab stop, arrow-key navigation, `Home`/`End`, `PageUp`/`PageDown`,
and filename typeahead. Grid arrows preserve row/column movement where possible. Refresh keeps the
current directory visible, selection and filter state stable while the request is pending. Large
directories request up to 1000 entries per cursor page, automatically continue beyond that limit,
and defer off-screen layout and paint work in browsers that support `content-visibility`.

Entry context menus expose only implemented operations: open, copy logical path, rename, move,
properties/info, and explicitly permanent delete. Current-folder context menus expose new folder,
new file, refresh, sort direction, details/grid view, details pane visibility, copy current path,
and properties/info. Menu labels and shortcut hints adapt to Windows, macOS, and Linux. Use right
click, the keyboard Menu key, or `Shift+F10`; arrow keys, Home/End, type-ahead, Escape, and focus
restoration follow desktop menu conventions. Directory-selection mode removes every mutation item.

Click or keyboard-focus the address bar to reveal and select the complete sandbox logical address
(`sandbox://<sandbox-id>/<logical-path>`). The copy button writes that address to the clipboard.
It intentionally does not reveal a server physical filesystem path.

The directory picker Modal uses platform-aware Windows, macOS, or Linux title-bar controls. Its
header can maximize or restore from the window button or a double-click, while `Escape`, the close
button, and backdrop clicks cancel the selection.

The filter applies only to the directory pages already loaded in the browser. Hosts should avoid
presenting it as a server-wide search until the sandbox list API exposes a standard search query.

Application shells can wrap their renderer with `SandboxDirectoryPickerProvider` and call
`useSandboxDirectoryPicker().pickDirectory()` from project workflows. The controller serializes
modal requests and resolves cancellation as `null`, so consumers do not duplicate dialog state.
