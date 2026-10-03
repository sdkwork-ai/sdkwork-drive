import type { DriveUploadImageFileLike } from "@sdkwork/drive-upload-image-core";
import { DriveUploadImageError } from "@sdkwork/drive-upload-image-core";
import type { WxMediaFile, WxMiniProgramLike } from "./wx-types";

/**
 * WeChat picker adapter: turns `wx.chooseMedia` results into byte sources the
 * shared upload controller can consume. Bytes stream through
 * `FileSystemManager.readFile` ranged reads, so nothing loads whole files
 * into memory up front (`DriveUploaderBlobLike.readRange` contract).
 */

export interface MpPickedImageFile extends DriveUploadImageFileLike {
  /** WeChat temporary file path, usable for `wx.previewImage` too. */
  readonly path: string;
}

export interface WxDriveImagePicker {
  pick(options: { count: number; sourceType: Array<"album" | "camera"> }): Promise<MpPickedImageFile[]>;
}

function fileNameOf(tempFilePath: string): string {
  const withoutQuery = tempFilePath.split("?")[0] ?? tempFilePath;
  const segments = withoutQuery.split("/");
  return segments[segments.length - 1] ?? withoutQuery;
}

export function createWxDriveImagePicker(wx: WxMiniProgramLike): WxDriveImagePicker {
  const readRange = (filePath: string, offsetBytes: number, lengthBytes: number): Promise<ArrayBuffer> =>
    new Promise((resolve, reject) => {
      wx.getFileSystemManager().readFile({
        filePath,
        position: offsetBytes,
        length: lengthBytes,
        success: (result) => resolve(result.data),
        fail: (error) =>
          reject(
            new DriveUploadImageError("upload-failed", `Failed to read picked image bytes: ${error.errMsg}`, {
              cause: error,
            }),
          ),
      });
    });

  const toPickedFile = (file: WxMediaFile): MpPickedImageFile => {
    const name = fileNameOf(file.tempFilePath);
    return {
      path: file.tempFilePath,
      size: file.size,
      name,
      readRange: (offsetBytes: number, lengthBytes: number) =>
        readRange(file.tempFilePath, offsetBytes, lengthBytes),
    };
  };

  return {
    pick({ count, sourceType }) {
      return new Promise((resolve, reject) => {
        wx.chooseMedia({
          count,
          mediaType: ["image"],
          sourceType,
          success: (result) => resolve((result.tempFiles ?? []).map(toPickedFile)),
          fail: (error) => {
            // The user dismissing the picker is not a failure.
            if (error.errMsg.includes("cancel")) {
              resolve([]);
              return;
            }
            reject(
              new DriveUploadImageError("upload-failed", `Image picker failed: ${error.errMsg}`, {
                cause: error,
              }),
            );
          },
        });
      });
    },
  };
}
