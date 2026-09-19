/**
 * Application upload declaration constants.
 *
 * Authority: `DRIVE_SPEC.md` section 18 (Application Upload Declaration Contract).
 * Declared values live in `apps/sdkwork-drive-pc/specs/upload.declaration.json`; this module
 * carries them into code so upload call sites reference a constant instead of repeating
 * literals. Call sites MUST NOT inline these values, and the declaration MUST NOT be
 * duplicated as a second local authority.
 *
 * The previous local values (`desktop-file-browser`, `desktop-file-editor`,
 * `drive_pc_file_upload`, `drive_pc_text_save`, `pc_text_editor`, `pc_local_file`) were not
 * rule-conforming: `appResourceType` must be a dotted `<domain>.<resource>` business type,
 * `scene` and `source` must be lowercase kebab-case. They are converged here to the declared
 * values.
 */

export interface DrivePcUploadDeclarationEntry {
  readonly appResourceIdKind: "application" | "entity" | "draft";
  readonly appResourceType: string;
  readonly purpose: string;
  readonly retention: "long_term" | "temporary";
  readonly scene: string;
  readonly source: string;
  readonly uploadProfileCode: string;
}

/** The single call-origin label for every upload from this application. */
export const DRIVE_PC_UPLOAD_SOURCE = "sdkwork-drive-pc" as const;

const DRIVE_PC_APP_RESOURCE_ID_KIND = "application" as const;
const DRIVE_PC_RETENTION = "long_term" as const;

/** File uploaded into a Drive space from the desktop Drive file browser. */
export const DRIVE_PC_FILE_BROWSER_ENTRY_UPLOAD = {
  appResourceIdKind: DRIVE_PC_APP_RESOURCE_ID_KIND,
  appResourceType: "drive.file_browser_entry",
  purpose: "File uploaded into a Drive space from the desktop Drive file browser.",
  retention: DRIVE_PC_RETENTION,
  scene: "drive-file-upload",
  source: DRIVE_PC_UPLOAD_SOURCE,
  uploadProfileCode: "generic",
} as const satisfies DrivePcUploadDeclarationEntry;

/** Text document saved from the desktop Drive editor back into the Drive space. */
export const DRIVE_PC_EDITOR_DOCUMENT_UPLOAD = {
  appResourceIdKind: DRIVE_PC_APP_RESOURCE_ID_KIND,
  appResourceType: "drive.editor_document",
  purpose: "Text document saved from the desktop Drive editor back into the Drive space.",
  retention: DRIVE_PC_RETENTION,
  scene: "drive-text-save",
  source: DRIVE_PC_UPLOAD_SOURCE,
  uploadProfileCode: "text",
} as const satisfies DrivePcUploadDeclarationEntry;

/** Every declared upload purpose for this application. */
export const DRIVE_PC_UPLOAD_DECLARATIONS: readonly DrivePcUploadDeclarationEntry[] = [
  DRIVE_PC_FILE_BROWSER_ENTRY_UPLOAD,
  DRIVE_PC_EDITOR_DOCUMENT_UPLOAD,
];
