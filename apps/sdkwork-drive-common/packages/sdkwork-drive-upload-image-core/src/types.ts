import type {
  DriveUploaderProgress,
  DriveUploaderRequest,
  DriveUploaderUploadResult,
} from "@sdkwork/drive-app-sdk/uploader";

/**
 * Shared contract for the reusable Drive image-upload component family
 * (`sdkwork-drive-pc-upload-image`, `@sdkwork/drive-mobile-react-upload-image`,
 * `@sdkwork/drive-mp-upload-image`, `drive_upload_image_composed`).
 *
 * Layering follows `FRONTEND_CODE_SPEC.md` and `DRIVE_SPEC.md` §9/§18: the UI
 * component receives an injected `DriveUploadImageService` built by the host
 * application's service layer. The component never composes upload intent
 * (`appResourceType`, `scene`, `source`) inline and never sees the SDK client.
 */

/**
 * Standard Drive upload profiles this component may exercise. A profile is
 * chosen by content shape (`DRIVE_SPEC.md` §8.1); custom profile codes are
 * forbidden, and a generic image component only binds the image-shaped ones.
 */
export type DriveUploadImageProfile = "image" | "avatar" | "thumbnail";

/**
 * The host application's declared upload intent (`DRIVE_SPEC.md` §18). The
 * application service layer imports its `<application-root>/specs/`-mirrored
 * declaration constant and passes it here; every field is validated by
 * {@linkcode assertDriveUploadImageDeclaration} when the service is created.
 */
export interface DriveUploadImageDeclaration {
  appResourceType: string;
  appResourceIdKind: "application" | "entity" | "draft";
  scene: string;
  source: string;
  uploadProfileCode: DriveUploadImageProfile;
  retention: "long_term" | "temporary";
  /** Required when `retention` is `temporary` (`DRIVE_SPEC.md` §18.1). */
  retentionTtlSeconds?: number;
  purpose: string;
}

/** Drive identity block mirrored from `DRIVE_SPEC.md` §10 `metadata.drive`. */
export interface DriveUploadImageDriveMetadata {
  spaceId: string;
  nodeId: string;
  spaceType?: string;
  nodeVersion?: string;
  contentType?: string;
  /** Int64-as-string wire value (`API_SPEC.md` §13.6); never parsed to number. */
  contentLength?: string;
  originalFileName?: string;
  checksumSha256Hex?: string;
}

/**
 * Persist-safe upload value carried by business forms. It holds only the
 * stable Drive reference — never presigned or delivery URLs
 * (`DRIVE_SPEC.md` §9). Transient previews live in controller/component state.
 */
export interface DriveUploadImageValue {
  /** `drive://spaces/{spaceId}/nodes/{nodeId}` for Drive-backed content. */
  uri: string;
  source: "drive" | "external";
  metadata?: {
    drive?: DriveUploadImageDriveMetadata;
    [key: string]: unknown;
  };
}

/**
 * Minimal byte-source contract accepted by {@linkcode DriveUploadImageService}.
 * Non-Blob runtimes (mini-program) implement `readRange` against their
 * platform file API; browser `File` sources are served through their
 * `arrayBuffer` bytes, which stay bounded by the validated image size limit.
 */
export interface DriveUploadImageFileLike {
  readonly size: number;
  readonly type?: string;
  readonly name?: string;
  arrayBuffer?(): Promise<ArrayBuffer>;
  readRange?(offsetBytes: number, lengthBytes: number): Promise<ArrayBuffer>;
}

/** Normalized progress snapshot surfaced to the UI. `percent` is 0..100. */
export interface DriveUploadImageProgress {
  uploadedBytes: number;
  totalBytes: number;
  uploadedPartsCount: number;
  totalParts: number;
  status: DriveUploaderProgress["status"];
  percent: number;
}

export type DriveUploadImageRejectionCode =
  | "invalid-file-type"
  | "file-too-large"
  | "empty-file"
  | "too-many-files";

export type DriveUploadImageErrorCode =
  | DriveUploadImageRejectionCode
  | "invalid-declaration"
  | "missing-app-resource-id"
  | "invalid-upload-result"
  | "upload-failed"
  | "preview-unsupported"
  | "controller-destroyed";

/** Typed failure for every business error this package raises. */
export class DriveUploadImageError extends Error {
  readonly code: DriveUploadImageErrorCode;

  constructor(
    code: DriveUploadImageErrorCode,
    message: string,
    options?: { cause?: unknown },
  ) {
    super(message, options === undefined ? undefined : { cause: options.cause });
    this.name = "DriveUploadImageError";
    this.code = code;
  }
}

/** Structural slice of the composed Drive uploader this package binds. */
export interface DriveUploadImageUploaderLike {
  uploadImage(request: DriveUploaderRequest): Promise<DriveUploaderUploadResult>;
  uploadAvatar(request: DriveUploaderRequest): Promise<DriveUploaderUploadResult>;
  uploadThumbnail(request: DriveUploaderRequest): Promise<DriveUploaderUploadResult>;
}

/**
 * Bounded preview reader over Drive node content. The host binds it to
 * `drive.drive.nodes` so previews stay same-origin and size-capped
 * (`DRIVE_SPEC.md` §8); the result is a transient data URL, never persisted.
 */
export interface DriveImagePreviewReaderLike {
  readImagePreview(input: {
    nodeId: string;
    maxBytes: number;
    signal?: AbortSignal | undefined;
  }): Promise<string | null>;
}

/** Service the host service layer builds once and injects into components. */
export interface DriveUploadImageService {
  upload(input: {
    file: DriveUploadImageFileLike;
    appResourceId: string;
    signal?: AbortSignal | undefined;
    onProgress?: (progress: DriveUploadImageProgress) => void;
  }): Promise<DriveUploadImageValue>;
  resolvePreview(input: {
    uri: string;
    maxBytes?: number | undefined;
    signal?: AbortSignal | undefined;
  }): Promise<string | null>;
}

/** File constraints a component exposes as configurable props. */
export interface DriveUploadImageFileConstraints {
  /** MIME or extension accept list, e.g. `["image/*"]`, `[".png", "image/jpeg"]`. */
  accept?: readonly string[];
  /** Upload ceiling in bytes; the shared default is 5 MiB. */
  maxSizeBytes?: number;
}

export const DRIVE_UPLOAD_IMAGE_DEFAULT_MAX_BYTES = 5 * 1024 * 1024;
export const DRIVE_UPLOAD_IMAGE_DEFAULT_ACCEPT: readonly string[] = ["image/*"];
export const DRIVE_UPLOAD_IMAGE_PREVIEW_MAX_BYTES = 2 * 1024 * 1024;
