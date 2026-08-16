import type { Dispatch, SetStateAction } from "react";
import {
  applyTransferFailure,
  applyUploadCompletionToJob,
  applyUploadProgressToJob,
  canUploadDriveFileToSection,
  createUploadJobForFile,
  createUploadJobForNativeFile,
} from "sdkwork-drive-pc-types";
import type { DriveFileService } from "sdkwork-drive-pc-core";
import { isDriveAbortError, NativeLocalUploadFile } from "sdkwork-drive-pc-core";
import { isDriveConflictError } from "sdkwork-drive-pc-commons";
import type { DriveSection } from "../pages/DrivePage";
import type { DownloadJob } from "./DownloadManager";
import { MAX_PARALLEL_UPLOADS, runWithConcurrency } from "./fileBrowserBatchUtils";

function resolveUploadSourceName(source: File | NativeLocalUploadFile): string {
  return source.name?.trim() || "upload.bin";
}

export type FileBrowserUploadToast = (
  message: string,
  type: "success" | "err" | "info",
) => void;

type UploadJobInput = {
  source: File | NativeLocalUploadFile;
  job: DownloadJob;
};

export type QueueFileBrowserUploadsParams = {
  uploadJobs: UploadJobInput[];
  toastLabel: string;
  fileService: DriveFileService;
  activeSection: DriveSection;
  currentFolderId: string | null;
  setDownloadJobs: Dispatch<SetStateAction<DownloadJob[]>>;
  createUploadAbortController: (jobId: string) => AbortController;
  releaseUploadAbortController: (jobId: string, controller?: AbortController) => void;
  loadFiles: () => void;
  triggerToast: FileBrowserUploadToast;
  t: (key: string, params?: Record<string, string | number>) => string;
};

export function queueFileBrowserUploads({
  uploadJobs,
  toastLabel,
  fileService,
  activeSection,
  currentFolderId,
  setDownloadJobs,
  createUploadAbortController,
  releaseUploadAbortController,
  loadFiles,
  triggerToast,
  t,
}: QueueFileBrowserUploadsParams): void {
  if (uploadJobs.length === 0) {
    return;
  }

  let completedUploadCount = 0;
  let shouldRefreshAfterSettled = false;

  setDownloadJobs((prev) => [...uploadJobs.map(({ job }) => job), ...prev]);
  triggerToast(toastLabel, "info");

  void runWithConcurrency(uploadJobs, MAX_PARALLEL_UPLOADS, async ({ source, job }) => {
    const uploadController = createUploadAbortController(job.id);
    try {
      const uploadedFile = await fileService.uploadFile(source, activeSection, currentFolderId, {
        taskId: job.id,
        signal: uploadController.signal,
        onProgress: (uploadedBytes, totalBytes) => {
          setDownloadJobs((prev) =>
            prev.map((item) =>
              item.id === job.id
                ? applyUploadProgressToJob(item, uploadedBytes, totalBytes)
                : item,
            ),
          );
        },
      });
      completedUploadCount += 1;
      shouldRefreshAfterSettled = true;
      setDownloadJobs((prev) =>
        prev.map((item) =>
          item.id === job.id ? applyUploadCompletionToJob(item, uploadedFile) : item,
        ),
      );
      const localName = resolveUploadSourceName(source);
      if (uploadedFile.name !== localName) {
        triggerToast(t("fileBrowser.toastUploadRenamed", { name: uploadedFile.name }), "info");
      } else {
        triggerToast(t("fileBrowser.toastUploadSuccess", { name: uploadedFile.name }), "success");
      }
      loadFiles();
    } catch (err) {
      if (isDriveAbortError(err)) {
        return;
      }

      shouldRefreshAfterSettled = true;
      setDownloadJobs((prev) =>
        prev.map((item) =>
          item.id === job.id
            ? applyTransferFailure(
                item,
                err instanceof Error ? err.message : t("fileBrowser.toastUploadFailed"),
              )
            : item,
        ),
      );
      triggerToast(
        isDriveConflictError(err)
          ? t("fileBrowser.nameConflict")
          : err instanceof Error
            ? err.message
            : t("fileBrowser.toastUploadFailed"),
        "err",
      );
      loadFiles();
    } finally {
      releaseUploadAbortController(job.id, uploadController);
    }
  }).then(() => {
    if (shouldRefreshAfterSettled || completedUploadCount > 0) {
      loadFiles();
    }
  });
}

export function queueSelectedFilesForUpload(
  selectedFiles: File[],
  params: Omit<QueueFileBrowserUploadsParams, "uploadJobs" | "toastLabel">,
): void {
  if (selectedFiles.length === 0) {
    return;
  }
  if (!canUploadDriveFileToSection(params.activeSection)) {
    params.triggerToast(params.t("fileBrowser.sectionUploadUnsupported"), "err");
    return;
  }

  queueFileBrowserUploads({
    ...params,
    uploadJobs: selectedFiles.map((file) => ({
      source: file,
      job: createUploadJobForFile(file, {
        uploadSection: params.activeSection,
        uploadParentId: params.currentFolderId ?? null,
        uploadBlob: file,
      }),
    })),
    toastLabel:
      selectedFiles.length === 1
        ? params.t("fileBrowser.toastFileAdded", { name: selectedFiles[0]!.name })
        : `Added ${selectedFiles.length} files to active upload transfers`,
  });
}
