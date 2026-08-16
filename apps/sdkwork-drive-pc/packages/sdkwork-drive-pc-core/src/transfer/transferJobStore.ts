import {
  applyTransferFailure,
  isActiveTransferStatus,
  type DownloadJob,
} from '../types';
import {
  TRANSFER_INTERRUPTION_TRANSFER_RETRY,
  TRANSFER_INTERRUPTION_UPLOAD_NATIVE_RETRY,
  TRANSFER_INTERRUPTION_UPLOAD_RESELECT,
} from './transferInterruptionCodes';

const DRIVE_TRANSFER_JOBS_STORAGE_KEY = 'sdkwork.drive.pc.transfer.jobs.v1';

function isLocalStorageAvailable(): boolean {
  return typeof window !== 'undefined' && Boolean(window.localStorage);
}

function sanitizeJobForStorage(job: DownloadJob): DownloadJob {
  const withoutBlob = {
    ...job,
    uploadBlob: undefined,
  };
  if (job.status === 'paused') {
    return withoutBlob;
  }
  return {
    ...withoutBlob,
    downloadUrl: undefined,
    signedSourceUrl: undefined,
  };
}

export function loadPersistedTransferJobs(): DownloadJob[] {
  if (!isLocalStorageAvailable()) {
    return [];
  }
  try {
    const raw = window.localStorage.getItem(DRIVE_TRANSFER_JOBS_STORAGE_KEY);
    if (!raw) {
      return [];
    }
    const parsed = JSON.parse(raw) as DownloadJob[];
    if (!Array.isArray(parsed)) {
      return [];
    }
    return parsed.map((job) => {
      const restored = sanitizeJobForStorage(job);
      if (isActiveTransferStatus(restored.status)) {
        return applyTransferFailure(
          restored,
          restored.type === 'upload'
            ? restored.uploadLocalPath
              ? TRANSFER_INTERRUPTION_UPLOAD_NATIVE_RETRY
              : TRANSFER_INTERRUPTION_UPLOAD_RESELECT
            : TRANSFER_INTERRUPTION_TRANSFER_RETRY,
        );
      }
      return restored;
    });
  } catch {
    return [];
  }
}
export function persistTransferJobs(jobs: DownloadJob[]): void {
  if (!isLocalStorageAvailable()) {
    return;
  }
  try {
    const serializableJobs = jobs.map(sanitizeJobForStorage);
    window.localStorage.setItem(
      DRIVE_TRANSFER_JOBS_STORAGE_KEY,
      JSON.stringify(serializableJobs),
    );
  } catch {
    // Ignore storage persistence failures and keep runtime behavior intact.
  }
}
