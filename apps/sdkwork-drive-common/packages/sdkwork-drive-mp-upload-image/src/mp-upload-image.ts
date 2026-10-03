import {
  DriveUploadImageController,
  type DriveUploadImageControllerOptions,
  type DriveUploadImageService,
  type DriveUploadImageSnapshot,
  type DriveUploadImageValue,
} from "@sdkwork/drive-upload-image-core";
import { createWxDriveImagePicker, type WxDriveImagePicker } from "./wx-picker";
import { resolveGlobalWx, type WxMiniProgramLike } from "./wx-types";

/**
 * WeChat mini-program binding for the shared upload state machine. A page or
 * component owns one `MpDriveUploadImage` instance, mirrors snapshots into
 * `setData`, and calls `chooseAndUpload` from tap handlers. The upload
 * service is injected by the host's service layer (`DRIVE_SPEC.md` §18.3).
 */

export interface MpDriveUploadImageOptions
  extends Omit<DriveUploadImageControllerOptions, "service" | "previewUrls"> {
  service: DriveUploadImageService;
  /** Mini-program host; defaults to the global `wx` when present. */
  wx?: WxMiniProgramLike;
}

export interface MpDriveUploadImage {
  controller: DriveUploadImageController;
  getSnapshot(): DriveUploadImageSnapshot;
  subscribe(listener: () => void): () => void;
  chooseAndUpload(options: {
    appResourceId?: string | null;
    source?: Array<"album" | "camera">;
    count?: number;
  }): Promise<readonly DriveUploadImageValue[]>;
  chooseFromAlbum(options?: { appResourceId?: string | null; count?: number }): Promise<readonly DriveUploadImageValue[]>;
  takePhoto(options?: { appResourceId?: string | null }): Promise<readonly DriveUploadImageValue[]>;
  removeItem(itemId: string): void;
  retryItem(itemId: string): Promise<readonly DriveUploadImageValue[]>;
  uploadPending(options?: { appResourceId?: string | null }): Promise<readonly DriveUploadImageValue[]>;
  setValues(values: readonly DriveUploadImageValue[] | null | undefined): void;
  destroy(): void;
}

export function createMpDriveUploadImage(options: MpDriveUploadImageOptions): MpDriveUploadImage {
  const { wx: wxHost, ...controllerOptions } = options;
  const host = wxHost ?? resolveGlobalWx();
  if (host === null) {
    throw new TypeError(
      "createMpDriveUploadImage requires a WeChat mini-program host; pass `wx` explicitly in tests or non-WeChat runtimes.",
    );
  }
  const picker: WxDriveImagePicker = createWxDriveImagePicker(host);
  const controller = new DriveUploadImageController({
    ...controllerOptions,
    previewUrls: null,
  });

  const runPicked = async (
    picked: { appResourceId?: string | null; source: Array<"album" | "camera">; count: number },
  ): Promise<readonly DriveUploadImageValue[]> => {
    const files = await picker.pick({
      count: picked.count,
      sourceType: picked.source,
    });
    if (files.length === 0) {
      return controller.getValues();
    }
    await controller.addFiles(files);
    if (picked.appResourceId !== undefined && picked.appResourceId !== null) {
      return controller.uploadPending({ appResourceId: picked.appResourceId });
    }
    return controller.getValues();
  };

  return {
    controller,
    getSnapshot: () => controller.getSnapshot(),
    subscribe: (listener) => controller.subscribe(listener),
    chooseAndUpload: ({ appResourceId, source, count }) =>
      runPicked({
        appResourceId: appResourceId ?? null,
        source: source ?? ["album"],
        count: count ?? 1,
      }),
    chooseFromAlbum: (inner) =>
      runPicked({
        appResourceId: inner?.appResourceId ?? null,
        source: ["album"],
        count: inner?.count ?? 1,
      }),
    takePhoto: (inner) =>
      runPicked({
        appResourceId: inner?.appResourceId ?? null,
        source: ["camera"],
        count: 1,
      }),
    removeItem: (itemId) => controller.removeItem(itemId),
    retryItem: (itemId) => controller.retryItem(itemId),
    uploadPending: (inner) =>
      controller.uploadPending({ appResourceId: inner?.appResourceId ?? null }),
    setValues: (values) => controller.setValues(values),
    destroy: () => controller.destroy(),
  };
}

/**
 * Mirrors controller snapshots into a page/component `setData` with a stable
 * shape for the reference template (`templates/upload-image.wxml`).
 */
export function mapSnapshotForTemplate(
  snapshot: DriveUploadImageSnapshot,
  context: { maxFiles?: number | undefined; readOnly?: boolean | undefined } = {},
): {
  items: Array<{
    id: string;
    status: string;
    fileName: string;
    previewUrl: string | null;
    progressPercent: number | null;
    errorMessage: string | null;
  }>;
  canAddMore: boolean;
  isUploading: boolean;
  values: readonly DriveUploadImageValue[];
} {
  const maxFiles = Math.max(1, context.maxFiles ?? 1);
  return {
    items: snapshot.items.map((item) => ({
      id: item.id,
      status: item.status,
      fileName: item.fileName,
      previewUrl: item.previewUrl,
      progressPercent: item.progressPercent,
      errorMessage: item.errorMessage,
    })),
    canAddMore: (context.readOnly ?? false) === false && snapshot.values.length < maxFiles,
    isUploading: snapshot.isUploading,
    values: snapshot.values,
  };
}
