# sdkwork-drive-pc-file-preview

Domain: drive
Capability: file-preview
Package type: PC internal React package
Status: standard

This package owns the Drive file preview and editor components. Every preview is
an independent component (image, video, audio, PDF, text, code, Word, Excel,
PowerPoint, archive, unsupported) and `FilePreviewSurface` is the only piece that
knows how to pick one.

It is deliberately host-agnostic: content arrives through the
`FilePreviewResource` port (read bytes, read text, resolve a direct URL, save
text, download) and every user-facing string arrives through
`FilePreviewLabels`, so the admin storage console and the Drive console can mount
the same components over their own services and dictionaries. The package never
imports a generated SDK, never builds a request, and never reaches for a host
dictionary.

## Behaviour contracts

- **The draft belongs to `FilePreviewSurface`, not to the editor.** Switching between
  Preview and Edit, or refetching after a save, remounts the editor; a draft held
  inside it would be silently discarded and the "closing with unsaved changes"
  guard would disarm itself. The surface owns the text, derives `dirty` from it,
  reports it through `onDirtyChange`, and the editor is fully controlled.
- **A save never eats keystrokes.** The editor goes read-only while a save is in
  flight, and the draft is only cleared when it still equals what was sent.
- **Download failures are visible.** `download` may return a promise; a rejection
  is shown next to the toolbar actions instead of disappearing.
- **Honest fallback over a wrong guess.** Office and archive previews check the
  container before parsing: a non-ZIP (legacy `.doc`/`.xls`/`.ppt`, rar/7z/tar) and
  a ZIP without the expected marker part (`word/document.xml`, `xl/workbook.xml`,
  `ppt/presentation.xml` — which covers `.odt`/`.ods`/`.odp`, Pages/Numbers/Keynote
  and damaged OOXML) both land on the download-oriented fallback panel. A damaged
  container whose central directory cannot be read still reaches the parser, so the
  user is told "damaged" rather than "unsupported".
- **Errors are localized by code, not by message.** Parser failures carry a stable
  code that maps to `corruptFileHint` / `parserUnavailableHint`; only host-channel
  errors (network, auth) surface their own message, because that text is what an
  operator needs to diagnose.
- **Delimited text is delimiter-aware.** `.tsv` splits on tabs and `.csv` sniffs
  `,` / `;` / tab from the first record.
- **Truncation is never silent.** Row and column limits each report their own flag
  and their own notice.
- **Every format has loading, empty, error, too-large and unavailable states**, each
  announced to assistive technology (`role="status"` / `role="alert"`).
- **A failed lazy chunk does not blank the host.** `PreviewErrorBoundary` wraps the
  lazily loaded editor so a failed Monaco fetch becomes a retryable panel.
- **Monaco is loaded locally and lazily.** The editor engine sits behind
  `React.lazy`, is configured (`loader.config({ monaco })`) at module evaluation
  rather than in an effect, and therefore never falls back to the CDN loader.

## Public API

- `.`

## Mounting it from another console

A host needs two things and nothing else: a content channel (`FilePreviewResource`) and
whatever copy it wants to customize (`FilePreviewLabelsInput`, merged over the English
defaults). `tests/publicApiReuse.test.tsx` is exactly that host, written against the public
barrel only, and it mounts every preview component — treat it as the integration recipe.

```tsx
import {
  FilePreviewSurface,
  mergeFilePreviewLabels,
  type FilePreviewResource,
} from 'sdkwork-drive-pc-file-preview';

const labels = mergeFilePreviewLabels({ download: 'Save a copy' });

const resource: FilePreviewResource = {
  name: object.name,
  contentType: object.contentType,
  sizeBytes: object.sizeBytes,
  readBytes: () => mySdk.readBytes(object.key),
  readText: () => mySdk.readText(object.key),
  saveText: (text) => mySdk.writeText(object.key, text),
  download: () => mySdk.download(object.key),
};

<FilePreviewSurface labels={labels} resource={resource} onClose={close} />;
```

`FILE_PREVIEW_KINDS` is exported so a host can enumerate the kinds (for menus, filters or
its own icon mapping) without duplicating the union type.

## Runtime requirements

The package is framework-light but not capability-free. A host should know what each
kind needs before it mounts one:

| Capability | Needed by | If missing |
| --- | --- | --- |
| `URL.createObjectURL` / `revokeObjectURL` | image, video, audio, PDF when the host passes bytes instead of a URL | the media kinds cannot build a source; pass `resolveUrl` instead |
| `DOMParser` | Word / Excel / PowerPoint previews (OOXML is XML) | the parser reports `dom-parser-unavailable` and the panel says the runtime cannot parse it |
| `DecompressionStream('deflate-raw')` | deflated ZIP entries (Office, archives) | falls back to `Blob.stream()`/`Response`; without either, only stored entries decode |
| `document.fullscreenElement` + fullscreen events | video fullscreen control | the button is hidden, playback is unaffected |
| `navigator.clipboard` | the copy actions | the copy buttons degrade to a no-op rather than throwing |
| `Intl.DateTimeFormat` | formatted sizes/timestamps | falls back to the runtime default locale |

Everything else (the kind registry, the OOXML model builders, the label merging) is pure
and runs in Node as well; the test suite proves it under both `node` and `jsdom`.

## Editor engine (Monaco) and the host

- The engine is loaded lazily and configured at module evaluation
  (`loader.config({ monaco })`), so it never falls back to the jsdelivr loader. A host
  that mounts its own `@monaco-editor/react` must do the same *before* rendering an
  `<Editor>`, otherwise the shared loader is already initialised and cannot be redirected.
- `MonacoEnvironment` is merged, not replaced: the package keeps whatever
  `getWorker` the host installed and only fills in the worker it knows.
- `monaco-editor` / `@monaco-editor/react` are declared as dependencies with a caret
  range; a host that pins another Monaco must add it to its bundler's `dedupe` list so
  one engine is bundled. The webserver host does exactly that (`vite.config.ts`
  `resolve.dedupe`).

## Reusing it from the Drive console (recommended path)

The Drive console (`sdkwork-drive-pc-file`) still ships its own `preview-modules/*`.
Migrating it is not all-or-nothing:

1. **Start with text/code.** It needs `readFileText`/`saveFileText` (already on
   `DriveFileService`), gives the console the VS Code stack instead of a CDN Monaco, and
   removes the largest duplication.
2. **Then images/PDF/video/audio** via `resolveUrl` (presigned URLs already exist there).
3. **Leave Office and archives for last**: the console's Office preview is a link card and
   its archive preview lists/extracts *server-side* into Drive — a capability this package
   deliberately does not have. Migrating those needs a new bytes API on `DriveFileService`,
   and dropping the server-side extraction would be a feature regression.

## Icon palette and contrast

`FILE_KIND_VISUALS` is the single definition of "folder is amber, spreadsheet is green".
Every icon colour is calibrated against its own tile in both themes, and
`tests/fileKindContrast.test.ts` fails the build if any pair drops below WCAG 1.4.11's
3:1 for graphical objects. The audit that produced this gate found the folder icon at
**2.07:1** (`amber-500` on `amber-50`) — the most frequent icon in the list was the
hardest to see; the current worst pair is 4.1:1.

## Verification

The package has no `test` script of its own (the workspace resolves one shared Vitest
config at the application root), so run the suite from `apps/sdkwork-drive-pc`:

- `pnpm exec vitest run packages/sdkwork-drive-pc-file-preview/tests`
- `pnpm typecheck`
