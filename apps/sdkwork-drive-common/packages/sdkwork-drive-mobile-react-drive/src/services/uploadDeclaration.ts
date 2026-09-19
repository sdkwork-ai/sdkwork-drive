/**
 * Application upload declaration constants.
 *
 * Authority: `DRIVE_SPEC.md` section 18 (Application Upload Declaration Contract).
 * Declared values live in `apps/sdkwork-drive-common/specs/upload.declaration.json`; this
 * module carries them into code so upload call sites reference a constant instead of
 * repeating literals. Call sites MUST NOT inline these values, and the declaration MUST NOT
 * be duplicated as a second local authority.
 *
 * The previous local values (`mobile-file-browser`, `drive_h5_file_upload`, `h5_local_file`)
 * were not rule-conforming: `appResourceType` must be a dotted `<domain>.<resource>` business
 * type, `scene` and `source` must be lowercase kebab-case. They are converged here to the
 * declared values.
 */

export interface DriveCommonUploadDeclarationEntry {
  readonly appResourceIdKind: "application" | "entity" | "draft";
  readonly appResourceType: string;
  readonly purpose: string;
  readonly retention: "long_term" | "temporary";
  readonly scene: string;
  readonly source: string;
  readonly uploadProfileCode: string;
}

/** The single call-origin label for every upload from this application root. */
export const DRIVE_COMMON_UPLOAD_SOURCE = "sdkwork-drive-common" as const;

const DRIVE_COMMON_APP_RESOURCE_ID_KIND = "application" as const;
const DRIVE_COMMON_RETENTION = "long_term" as const;

/** File uploaded into a Drive space from the shared mobile Drive browser surface. */
export const DRIVE_COMMON_MOBILE_FILE_BROWSER_ENTRY_UPLOAD = {
  appResourceIdKind: DRIVE_COMMON_APP_RESOURCE_ID_KIND,
  appResourceType: "drive.mobile_file_browser_entry",
  purpose: "File uploaded into a Drive space from the shared mobile Drive browser surface.",
  retention: DRIVE_COMMON_RETENTION,
  scene: "file-upload",
  source: DRIVE_COMMON_UPLOAD_SOURCE,
  uploadProfileCode: "generic",
} as const satisfies DriveCommonUploadDeclarationEntry;

/** Every declared upload purpose for this application root. */
export const DRIVE_COMMON_UPLOAD_DECLARATIONS: readonly DriveCommonUploadDeclarationEntry[] = [
  DRIVE_COMMON_MOBILE_FILE_BROWSER_ENTRY_UPLOAD,
];
