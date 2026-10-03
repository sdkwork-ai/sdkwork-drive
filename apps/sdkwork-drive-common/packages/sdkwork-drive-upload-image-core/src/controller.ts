import { resolveDriveUploadImageConstraints, validateDriveUploadImageFile } from "./validation";
import {
  DRIVE_UPLOAD_IMAGE_DEFAULT_ACCEPT,
  DRIVE_UPLOAD_IMAGE_DEFAULT_MAX_BYTES,
  type DriveUploadImageFileConstraints,
  type DriveUploadImageFileLike,
  type DriveUploadImageRejectionCode,
  type DriveUploadImageService,
  type DriveUploadImageValue,
} from "./types";
import { DriveUploadImageError } from "./types";
import { toDriveUploadImageFailure } from "./value";

/**
 * Framework-agnostic upload state machine shared by every platform shell.
 * React shells bind it with `useSyncExternalStore`; the mini-program shell
 * mirrors snapshots into `setData`; the Flutter shell mirrors it in a
 * `ChangeNotifier`. The controller never sees the SDK client and never
 * composes upload intent — it only drives the injected service.
 */

export type DriveUploadImageItemStatus =
  | "pending"
  | "uploading"
  | "uploaded"
  | "error"
  | "rejected";

export interface DriveUploadImageItem {
  id: string;
  status: DriveUploadImageItemStatus;
  fileName: string;
  sizeBytes: number;
  contentType: string | null;
  /** 0..100 while `uploading`; `null` otherwise. */
  progressPercent: number | null;
  /** Transient display URL: local object URL or resolved Drive preview. */
  previewUrl: string | null;
  value: DriveUploadImageValue | null;
  rejectionCode: DriveUploadImageRejectionCode | null;
  errorMessage: string | null;
}

export interface DriveUploadImageSnapshot {
  items: readonly DriveUploadImageItem[];
  values: readonly DriveUploadImageValue[];
  isUploading: boolean;
  hasPending: boolean;
}

/** Local preview adapter; hosts without object URLs pass `null`. */
export interface DriveUploadImagePreviewUrlAdapter {
  createObjectUrl(file: DriveUploadImageFileLike): string | null;
  revokeObjectUrl(url: string): void;
}

const nullPreviewUrlAdapter: DriveUploadImagePreviewUrlAdapter = {
  createObjectUrl: () => null,
  revokeObjectUrl: () => undefined,
};

function defaultPreviewUrlAdapter(): DriveUploadImagePreviewUrlAdapter {
  const urlLike = globalThis as {
    URL?: {
      createObjectURL?: (source: unknown) => string;
      revokeObjectURL?: (url: string) => void;
    };
  };
  const url = urlLike.URL;
  if (
    url !== undefined &&
    typeof url.createObjectURL === "function" &&
    typeof url.revokeObjectURL === "function"
  ) {
    return {
      createObjectUrl: (file) => url.createObjectURL?.(file) ?? null,
      revokeObjectUrl: (target) => url.revokeObjectURL?.(target),
    };
  }
  return nullPreviewUrlAdapter;
}

export interface DriveUploadImageControllerOptions extends DriveUploadImageFileConstraints {
  service: DriveUploadImageService;
  /** Maximum admitted images; `1` renders the single-field shells. */
  maxFiles?: number | undefined;
  /** Single-field mode: picking again replaces the stored image. */
  replaceOnMax?: boolean | undefined;
  /**
   * Supplies the entity anchor at upload time. Returning `null`/`undefined`
   * defers the upload until `uploadPending` carries one — the
   * persist-first-then-upload flow (`DRIVE_SPEC.md` §18.3).
   */
  resolveAppResourceId?: (() => string | null | undefined) | undefined;
  previewUrls?: DriveUploadImagePreviewUrlAdapter | null | undefined;
  onUploaded?: ((values: readonly DriveUploadImageValue[]) => void) | undefined;
  onRejected?:
    | ((rejection: { item: DriveUploadImageItem; code: DriveUploadImageRejectionCode }) => void)
    | undefined;
  onFailed?:
    | ((failure: { item: DriveUploadImageItem; error: DriveUploadImageError }) => void)
    | undefined;
}

interface InternalItem extends DriveUploadImageItem {
  file: DriveUploadImageFileLike | null;
  abort: AbortController | null;
}

interface QueuedUpload {
  itemId: string;
  appResourceId: string | null;
  resolve: (values: readonly DriveUploadImageValue[]) => void;
  reject: (error: DriveUploadImageError) => void;
}

/**
 * Sequential upload queue: one image in flight per controller keeps progress
 * readable and makes state trivially reproducible across platforms.
 */
export class DriveUploadImageController {
  private readonly service: DriveUploadImageService;
  private readonly maxFiles: number;
  private readonly replaceOnMax: boolean;
  private readonly accept: readonly string[];
  private readonly maxSizeBytes: number;
  private readonly resolveAppResourceId: (() => string | null | undefined) | undefined;
  private readonly previewUrls: DriveUploadImagePreviewUrlAdapter;
  private readonly onUploaded: ((values: readonly DriveUploadImageValue[]) => void) | undefined;
  private readonly onRejected:
    | ((rejection: { item: DriveUploadImageItem; code: DriveUploadImageRejectionCode }) => void)
    | undefined;
  private readonly onFailed:
    | ((failure: { item: DriveUploadImageItem; error: DriveUploadImageError }) => void)
    | undefined;

  private readonly listeners = new Set<() => void>();
  private readonly items: InternalItem[] = [];
  private readonly queue: QueuedUpload[] = [];
  private readonly resolvedPreviews = new Map<string, string>();
  private snapshot: DriveUploadImageSnapshot;
  private nextItemId = 1;
  private draining = false;
  private destroyed = false;
  /** Entity anchor of the last accepted upload; the retry fallback. */
  private lastAnchor: string | null = null;

  constructor(options: DriveUploadImageControllerOptions) {
    this.service = options.service;
    this.maxFiles = Math.max(1, Math.trunc(options.maxFiles ?? 1));
    this.replaceOnMax = options.replaceOnMax ?? this.maxFiles === 1;
    this.accept = options.accept ?? DRIVE_UPLOAD_IMAGE_DEFAULT_ACCEPT;
    this.maxSizeBytes = options.maxSizeBytes ?? DRIVE_UPLOAD_IMAGE_DEFAULT_MAX_BYTES;
    this.resolveAppResourceId = options.resolveAppResourceId;
    this.previewUrls =
      options.previewUrls === undefined
        ? defaultPreviewUrlAdapter()
        : options.previewUrls === null
          ? nullPreviewUrlAdapter
          : options.previewUrls;
    this.onUploaded = options.onUploaded;
    this.onRejected = options.onRejected;
    this.onFailed = options.onFailed;
    this.snapshot = this.buildSnapshot();
  }

  subscribe = (listener: () => void): (() => void) => {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  };

  getSnapshot = (): DriveUploadImageSnapshot => {
    return this.snapshot;
  };

  getValues(): readonly DriveUploadImageValue[] {
    return this.snapshot.values;
  }

  /**
   * Reconciles externally-owned values (form load, parent reset). Identical
   * uri sequences are ignored so the parent's `onChange` round-trip does not
   * drop transient previews or re-fetch them. Does not fire `onUploaded`.
   */
  setValues(values: readonly DriveUploadImageValue[] | null | undefined): void {
    this.assertAlive();
    const incoming = values ?? [];
    const current = this.items.filter((item) => item.status === "uploaded");
    if (
      incoming.length === current.length &&
      incoming.every((value, index) => value.uri === current[index]?.value?.uri)
    ) {
      return;
    }
    for (const item of current) {
      this.dropItem(item.id, { notify: false });
    }
    for (const value of incoming) {
      this.items.push({
        id: `image-${this.nextItemId}`,
        status: "uploaded",
        fileName: this.fileNameOf(value),
        sizeBytes: 0,
        contentType: null,
        progressPercent: null,
        previewUrl: this.transientPreviewOf(value),
        value,
        rejectionCode: null,
        errorMessage: null,
        file: null,
        abort: null,
      });
      this.nextItemId += 1;
    }
    this.publish();
    void this.resolveMissingPreviews();
  }

  /**
   * Admits picked files: validates each, keeps admissible ones as pending
   * items with a local preview, and starts uploads for those that already
   * have an entity anchor.
   */
  async addFiles(files: readonly DriveUploadImageFileLike[]): Promise<void> {
    this.assertAlive();
    const admitted: InternalItem[] = [];
    for (const file of files) {
      const rejectionCode = validateDriveUploadImageFile(file, {
        accept: this.accept,
        maxSizeBytes: this.maxSizeBytes,
      });
      if (rejectionCode !== null) {
        const rejected = this.pushRejectedItem(file, rejectionCode);
        this.onRejected?.({ item: rejected, code: rejectionCode });
        continue;
      }
      if (this.replaceOnMax && this.maxFiles === 1) {
        for (const item of [...this.items]) {
          if (
            item.status === "uploaded" ||
            item.status === "pending" ||
            item.status === "uploading" ||
            item.status === "error"
          ) {
            this.dropItem(item.id, { notify: false });
          }
        }
      } else {
        const occupied = this.items.filter(
          (item) => item.status !== "rejected",
        ).length;
        if (occupied >= this.maxFiles) {
          const rejected = this.pushRejectedItem(file, "too-many-files");
          this.onRejected?.({ item: rejected, code: "too-many-files" });
          continue;
        }
      }
      const previewUrl = this.previewUrls.createObjectUrl(file);
      const item: InternalItem = {
        id: `image-${this.nextItemId}`,
        status: "pending",
        fileName: file.name ?? "image",
        sizeBytes: file.size,
        contentType: file.type ?? null,
        progressPercent: null,
        previewUrl,
        value: null,
        rejectionCode: null,
        errorMessage: null,
        file,
        abort: null,
      };
      this.nextItemId += 1;
      this.items.push(item);
      admitted.push(item);
    }
    this.publish();
    if (admitted.length > 0) {
      await this.startEligibleUploads();
    }
  }

  /**
   * Uploads every pending item with the given entity anchor (or the bound
   * resolver) and resolves with the full uploaded value set — the
   * persist-first flow calls this after the entity exists.
   */
  uploadPending(
    context: { appResourceId?: string | null } = {},
  ): Promise<readonly DriveUploadImageValue[]> {
    this.assertAlive();
    return new Promise((resolve, reject) => {
      this.queue.push({
        itemId: "",
        appResourceId: context.appResourceId ?? null,
        resolve,
        reject,
      });
      void this.drain();
    });
  }

  retryItem(itemId: string): Promise<readonly DriveUploadImageValue[]> {
    this.assertAlive();
    const item = this.items.find((candidate) => candidate.id === itemId);
    if (item === undefined || item.status !== "error") {
      return Promise.resolve(this.getValues());
    }
    item.status = "pending";
    item.errorMessage = null;
    item.progressPercent = null;
    this.publish();
    return this.uploadPending();
  }

  /** Removes any item (uploaded, pending, rejected, or failed). */
  removeItem(itemId: string): void {
    this.assertAlive();
    this.dropItem(itemId, { notify: true });
  }

  clear(): void {
    this.assertAlive();
    for (const item of [...this.items]) {
      this.dropItem(item.id, { notify: false });
    }
    this.publish();
  }

  /** Aborts in-flight uploads and releases every transient preview URL. */
  destroy(): void {
    if (this.destroyed) {
      return;
    }
    this.destroyed = true;
    for (const item of this.items) {
      item.abort?.abort();
      this.revokePreview(item);
    }
    this.items.length = 0;
    for (const entry of this.queue) {
      entry.reject(new DriveUploadImageError("controller-destroyed", "Controller destroyed."));
    }
    this.queue.length = 0;
    this.listeners.clear();
    this.snapshot = { items: [], values: [], isUploading: false, hasPending: false };
  }

  private assertAlive(): void {
    if (this.destroyed) {
      throw new DriveUploadImageError(
        "controller-destroyed",
        "This controller has been destroyed.",
      );
    }
  }

  private pushRejectedItem(
    file: DriveUploadImageFileLike,
    code: DriveUploadImageRejectionCode,
  ): InternalItem {
    const item: InternalItem = {
      id: `image-${this.nextItemId}`,
      status: "rejected",
      fileName: file.name ?? "image",
      sizeBytes: file.size,
      contentType: file.type ?? null,
      progressPercent: null,
      previewUrl: null,
      value: null,
      rejectionCode: code,
      errorMessage: null,
      file: null,
      abort: null,
    };
    this.nextItemId += 1;
    this.items.push(item);
    this.publish();
    return item;
  }

  private async startEligibleUploads(): Promise<void> {
    const anchor = this.resolveAppResourceId?.() ?? null;
    if (anchor === null) {
      return;
    }
    await this.uploadPending({ appResourceId: anchor });
  }

  private async drain(): Promise<void> {
    if (this.draining || this.destroyed) {
      return;
    }
    this.draining = true;
    try {
      while (this.queue.length > 0) {
        const entry = this.queue.shift();
        if (entry === undefined) {
          break;
        }
        try {
          const values = await this.uploadOnce(entry.appResourceId);
          entry.resolve(values);
        } catch (error) {
          entry.reject(
            error instanceof DriveUploadImageError
              ? error
              : toDriveUploadImageFailure(error),
          );
        }
      }
    } finally {
      this.draining = false;
    }
  }

  private async uploadOnce(
    queueAnchor: string | null,
  ): Promise<readonly DriveUploadImageValue[]> {
    const anchor =
      queueAnchor ?? this.resolveAppResourceId?.() ?? this.lastAnchor ?? null;
    const pending = this.items.filter((item) => item.status === "pending");
    if (pending.length === 0) {
      return this.getValues();
    }
    if (anchor === null || anchor.trim() === "") {
      return this.getValues();
    }
    this.lastAnchor = anchor;
    for (const item of pending) {
      if (item.status !== "pending" || this.destroyed) {
        continue;
      }
      const file = item.file;
      if (file === null) {
        continue;
      }
      const abort = new AbortController();
      item.abort = abort;
      item.status = "uploading";
      item.progressPercent = 0;
      item.errorMessage = null;
      this.publish();
      try {
        const value = await this.service.upload({
          file,
          appResourceId: anchor,
          signal: abort.signal,
          onProgress: (progress) => {
            item.progressPercent = progress.percent;
            this.publish();
          },
        });
        item.status = "uploaded";
        item.value = value;
        item.progressPercent = null;
        item.file = null;
        item.abort = null;
      } catch (error) {
        item.abort = null;
        item.progressPercent = null;
        // A dropped item (replaced by a newer pick, or the controller was
        // destroyed) settles quietly; its promise owner is already gone.
        if (this.destroyed || !this.items.includes(item)) {
          continue;
        }
        item.status = "error";
        const failure = toDriveUploadImageFailure(error);
        item.errorMessage = failure.message;
        this.publish();
        this.onFailed?.({ item, error: failure });
        throw failure;
      }
      this.publish();
    }
    const values = this.getValues();
    this.onUploaded?.(values);
    return values;
  }

  private dropItem(
    itemId: string,
    context: { notify: boolean },
  ): void {
    const index = this.items.findIndex((item) => item.id === itemId);
    if (index < 0) {
      return;
    }
    const [item] = this.items.splice(index, 1);
    if (item !== undefined) {
      item.abort?.abort();
      this.revokePreview(item);
    }
    if (context.notify) {
      this.publish();
      this.onUploaded?.(this.getValues());
    }
  }

  private revokePreview(item: InternalItem): void {
    if (item.previewUrl !== null && item.value === null) {
      this.previewUrls.revokeObjectUrl(item.previewUrl);
    }
    item.previewUrl = null;
  }

  private fileNameOf(value: DriveUploadImageValue): string {
    const drive = value.metadata?.drive;
    if (drive !== undefined && typeof drive.originalFileName === "string") {
      return drive.originalFileName;
    }
    return value.uri;
  }

  private transientPreviewOf(value: DriveUploadImageValue): string | null {
    if (value.source === "drive") {
      return this.resolvedPreviews.get(value.uri) ?? null;
    }
    if (
      value.uri.startsWith("http://") ||
      value.uri.startsWith("https://") ||
      value.uri.startsWith("data:") ||
      value.uri.startsWith("blob:")
    ) {
      return value.uri;
    }
    return null;
  }

  private async resolveMissingPreviews(): Promise<void> {
    const missing = this.items.filter(
      (item) => item.status === "uploaded" && item.previewUrl === null && item.value !== null,
    );
    for (const item of missing) {
      const value = item.value;
      if (value === null) {
        continue;
      }
      try {
        const previewUrl = await this.service.resolvePreview({ uri: value.uri });
        if (previewUrl !== null) {
          this.resolvedPreviews.set(value.uri, previewUrl);
          item.previewUrl = previewUrl;
          this.publish();
        }
      } catch {
        item.previewUrl = null;
      }
    }
  }

  private buildSnapshot(): DriveUploadImageSnapshot {
    return {
      items: this.items.map((item) => ({ ...item })),
      values: this.items
        .filter(
          (item): item is InternalItem & { value: DriveUploadImageValue } =>
            item.status === "uploaded" && item.value !== null,
        )
        .map((item) => item.value),
      isUploading: this.items.some((item) => item.status === "uploading"),
      hasPending: this.items.some((item) => item.status === "pending"),
    };
  }

  private publish(): void {
    this.snapshot = this.buildSnapshot();
    for (const listener of this.listeners) {
      listener();
    }
  }
}
