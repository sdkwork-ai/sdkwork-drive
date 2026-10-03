# sdkwork-drive-pc-admin-storage-buckets

Domain: drive
Capability: storage-buckets
Package type: PC internal admin React package
Status: standard

This package owns the Drive storage bucket browser: the operator picks a storage
provider configuration, sees every bucket that provider account exposes, and
opens any of them in a file manager dialog that creates, reads, updates, and
deletes objects. Clicking a file opens it in the shared preview/editor components
from `sdkwork-drive-pc-file-preview` (images, video, audio, PDF, text, code,
Word/Excel/PowerPoint, archives), fed through this package's own backend-api
content channel; the dialog keeps the file list on one screen (no footer, one
compact header row) so the list itself gets the space.

It consumes the generated Drive admin storage SDK through the shared
`sdkwork-drive-pc-admin-storage-providers` service boundary and must not
construct raw HTTP requests, manual auth headers, provider SDK clients, or
generated SDK internals.

**One declared exception, scoped to direct object transfer.** The package
declares `@aws-sdk/client-s3` and `@aws-sdk/s3-request-presigner` (pinned once in
the workspace `catalog:`) so objects above the management API's inline 8 MiB
read/write limit can move between the browser and the provider's own endpoint
instead of through the backend. The bounds that keep this from becoming a second
storage plane:

- the client is built only from a **server-issued** claim — the S3-compatible
  endpoint, region, path-style flag, scoped bucket and short-lived scoped
  credentials (the same field set CloudRouter's `cloudStorageSdkConfig` returns);
  no provider secret is ever bundled, configured, or entered in the browser;
- every other storage call — provider configuration, bucket inventory, object
  listing under the limit, credentials — keeps going through the service
  boundary above;
- the dependency is declared ahead of its caller: the transfer path lands
  together with the presign/credential endpoint on the Drive admin storage API.
  Until then nothing in this package imports it.

## Behaviour contracts

- **The bucket travels with every request.** Reads, writes, copies, deletes and
  downloads all carry the bucket the operator opened, instead of relying on the
  bucket configured on the provider account.
- **Directory placeholders are not rows.** The backend materialises a "folder" as an
  empty object whose key equals the prefix, and returns it when that prefix is
  listed; the panel filters it out, so an empty folder shows its empty state instead
  of a nameless `0 B` row that could be renamed away — which would delete the folder.
- **Mutations refresh the list on both outcomes.** A rename is copy+delete and an
  upload is a sequence of writes, so a failure can leave partial results; the list
  is refetched either way, and the failure copy says the operation may have partly
  completed. The refresh never moves the operator back to the folder they left.
- **Destructive writes ask first.** Deleting, deleting a selection, overwriting an
  existing object on upload, and renaming onto an existing key each require an
  explicit confirmation, because object-storage writes replace whole objects
  irreversibly.
- **Names are validated before submission.** A create/rename value cannot contain
  `/` or `\`, and cannot be `.` or `..`; otherwise it would silently move the object
  into another prefix.
- **Objects above the 8 MiB inline limit are honest, not broken.** Their download
  action stays visible but disabled and explains the limit, and the preview shows
  the too-large panel with the provider-console route instead of a doomed button.
- **Search and categories cover the current page only**, and the rail says so.
- **Every bucket row answers three questions: which bucket, where it lives, and what it is
  here.** The role column carries a value on every row (write target / browse only) — the
  status column it replaced was a dash on every row but the configured one. The region
  column shows exactly what the API returned for that bucket, localized (name + code), and
  stays empty when it returned none: the account inventory spans regions, so the region
  configured on the provider says where its default endpoint points, not where an
  arbitrary bucket lives. Rows are 48px — the same density as the file manager's — and the
  whole row is the control (Tab + Enter/Space), with the per-row action button stopping
  propagation so one click opens the bucket exactly once.
- **Timestamps are formatted where the language is known.** The service boundary carries
  ISO 8601 (`creationDateIso` / `lastModifiedIso`) and every display site formats through
  `formatDriveDate` / `formatDriveDateTime` with the console language; formatting in the
  service would read the *browser* locale and put the wrong language's dates on an
  otherwise localized page.
- **The provider rail is the shared providers-package component.** Its credential
  tabs (configured / missing / all) and its localized built-in configuration names
  are owned by `sdkwork-drive-pc-admin-storage-providers`; this page supplies the
  option set, the selection, and the bucket list behind it.
- **Uploads above 8 MiB go through presigned multipart, directly to the vendor.**
  `uploadObjectInParts` (in `src/utils/multipartUpload.ts`) opens a vendor multipart
  upload (`storageProviders.objects.multipartUpload.create`), presigns batches of parts
  (`…parts.presign`), PUTs each 8 MiB part straight to the vendor URL with the signed
  headers echoed verbatim, and completes with the collected ETags (`…complete`). The
  bytes never traverse our gateway, there is no server-side buffering, progress is
  reported per part, transient part failures are retried with backoff, and cancelling
  (or closing the dialog) calls `…abort` so orphaned parts stop billing. Objects at or
  below 8 MiB keep using the single-request JSON+base64 channel: simpler, and free of
  any cross-origin dependency.
  **Operator prerequisite:** the bucket's CORS policy must allow this console's origin,
  the `PUT` method, the signed request headers, and — easy to miss — it must
  **expose the `ETag` response header**. Without that exposure every part uploads
  successfully and the completion still fails, so the uploader classifies both failure
  shapes explicitly (`cors` when the browser blocks the request, `etag-missing` when the
  response hides the ETag) and the dialog prints the exact rule to add instead of the
  browser's bare `Failed to fetch`.
- **Uploads are bounded by the *request body* limit, not only by the 8 MiB object cap.**
  Object content travels as base64 inside JSON, so a legal 8 MiB object is a ≈11.2 MB
  request; the gateway's `limits.maxRequestBodyBytes` (16 MiB by default) and the
  route's explicit `DefaultBodyLimit` must both admit it. When a server still rejects
  the body, the dialog explains the 1.37× encoding overhead and names the knob to raise
  instead of echoing the framework's bare `Payload too large` (code 41301). A failed
  upload also keeps its message across the follow-up refresh (`preserveError`): that
  refresh exists to show the real state, not to erase why the operation failed.
  This path only ever carries ≤8 MiB objects now; larger ones use multipart above.
- **The object list is paginated, not "load more".** The list API is cursor-only
  (`page_size` + `cursor`, no offset and no total), so the page footer carries the
  page number, the item count on that page, a per-page selector (50/100/200) and
  previous/next buttons. Pages already visited stay in memory, which makes
  *previous* instant and exact; forward paging follows the cursor the previous page
  returned. Changing the page size discards the cursor chain and restarts at page
  one. Selection survives paging (batch delete/download resolve keys across every
  loaded page), and the single scroll container is the row area — the toolbar, the
  table header and the page footer never scroll with the rows.

## Interaction model (industry-drive parity)

- Click a row to open: a folder navigates into it, a file opens the preview. The row
  is focusable and opens with Enter/Space.
- Hover-revealed checkboxes on each row plus a select-all checkbox in the header give
  multi-selection; the selection replaces the toolbar's action group in place (no
  extra row, so the list keeps its height) with download-selected, delete-selected and
  clear. `Ctrl/Cmd+A` selects everything visible, `Esc` clears, `Delete`/`Backspace`
  asks to delete the selection.
- Dropping files anywhere on the list uploads them into the current folder, through
  the same path as the file picker (size pre-check, overwrite confirmation,
  sequential writes).
- Searching and filtering never issue a second list request; they narrow what is
  already loaded.

## Public API

- `.`

## Required SDK Surface

- `@sdkwork/drive-admin-storage-sdk` (consumed through `sdkwork-drive-pc-admin-core`)

## Verification

- `pnpm test -- packages/sdkwork-drive-pc-admin-storage-buckets/tests`
- `pnpm typecheck`
