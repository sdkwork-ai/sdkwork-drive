import { scopeDriveStorageKey, type DriveStorageScope } from '../session/driveStorageScope';
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

function transferJobsStorageKey(scope?: DriveStorageScope): string {
  return scope
    ? scopeDriveStorageKey(DRIVE_TRANSFER_JOBS_STORAGE_KEY, scope)
    : DRIVE_TRANSFER_JOBS_STORAGE_KEY;
}

/**
 * Drop the list written before transfers were account-scoped.
 *
 * Those jobs carry no owner, so keeping them would either be dead bytes or, if
 * adopted, show one account's file names and space ids to the next one.
 */
function discardUnscopedTransferJobs(scope: DriveStorageScope | undefined): void {
  if (!scope || !isLocalStorageAvailable()) {
    return;
  }
  try {
    window.localStorage.removeItem(DRIVE_TRANSFER_JOBS_STORAGE_KEY);
  } catch {
    // Best-effort cleanup; a failed removal only leaves unread legacy bytes.
  }
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

export function loadPersistedTransferJobs(scope?: DriveStorageScope): DownloadJob[] {
  if (!isLocalStorageAvailable()) {
    return [];
  }
  discardUnscopedTransferJobs(scope);
  try {
    const raw = window.localStorage.getItem(transferJobsStorageKey(scope));
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
export function persistTransferJobs(jobs: DownloadJob[], scope?: DriveStorageScope): void {
  if (!isLocalStorageAvailable()) {
    return;
  }
  discardUnscopedTransferJobs(scope);
  try {
    const serializableJobs = jobs.map(sanitizeJobForStorage);
    window.localStorage.setItem(
      transferJobsStorageKey(scope),
      JSON.stringify(serializableJobs),
    );
  } catch {
    // Ignore storage persistence failures and keep runtime behavior intact.
  }
}
