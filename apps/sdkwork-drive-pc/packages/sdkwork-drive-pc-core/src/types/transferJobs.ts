import type { DriveFile, DownloadJob } from './file';

const DEFAULT_DOWNLOAD_SIZE_BYTES = 4_500_000;
const DEFAULT_FOLDER_ARCHIVE_SIZE_BYTES = 18_500_000;
const VIEW_SECTIONS = new Set(['recent', 'starred', 'shared', 'trash', 'transfer']);
let fallbackJobIdCounter = 0;

export interface CreateDownloadJobOptions {
  id?: string;
  packageName?: string;
  fallbackSizeBytes?: number;
}

export interface CreateUploadJobOptions {
  id?: string;
  fileId?: string;
  fallbackSizeBytes?: number;
  uploadSection?: string;
  uploadParentId?: string | null;
  uploadBlob?: File;
  uploadLocalPath?: string;
}

export interface NativeUploadJobDescriptor {
  path: string;
  name: string;
  size: number;
  modifiedAt: string;
  mimeType: string;
}

function buildUploadFileFingerprint(file: File): string {
  return `${file.name}:${file.size}:${file.lastModified}`;
}

export function buildNativeUploadJobFingerprint(
  descriptor: Pick<NativeUploadJobDescriptor, 'path' | 'size' | 'modifiedAt'>,
): string {
  return `${descriptor.path}:${descriptor.size}:${descriptor.modifiedAt}`;
}

export interface DownloadGrantLike {
  id?: string;
  packageName?: string;
  fileCount?: number;
  downloadUrl?: string;
  signedSourceUrl?: string;
  expiresAtEpochMs?: number;
  method?: string;
  totalBytes?: number;
  archiveSizeBytes?: number;
}

export function isActiveTransferStatus(status: DownloadJob['status']): boolean {
  return (
    status === 'connecting' ||
    status === 'compressing' ||
    status === 'downloading' ||
    status === 'uploading' ||
    status === 'checking'
  );
}

export function canControlTransferJob(job: Pick<DownloadJob, 'status'>): boolean {
  return canCancelTransferJob(job) || canPauseTransferJob(job) || canResumeTransferJob(job);
}

export function canCancelTransferJob(job: Pick<DownloadJob, 'status'>): boolean {
  return isActiveTransferStatus(job.status) || job.status === 'paused';
}

export function canPauseTransferJob(job: Pick<DownloadJob, 'status' | 'type'>): boolean {
  return job.type === 'download' && isActiveTransferStatus(job.status);
}

export function canResumeTransferJob(
  job: Pick<DownloadJob, 'status' | 'type' | 'downloadUrl' | 'signedSourceUrl' | 'expiresAtEpochMs'>,
): boolean {
  return job.type === 'download' && job.status === 'paused' && isDownloadGrantStillValid(job);
}

export function isDownloadGrantStillValid(
  job: Pick<DownloadJob, 'downloadUrl' | 'signedSourceUrl' | 'expiresAtEpochMs'>,
): boolean {
  if (!resolveTransferOpenUrl(job)) {
    return false;
  }
  if (job.expiresAtEpochMs && job.expiresAtEpochMs <= Date.now()) {
    return false;
  }
  return true;
}

export function pauseTransferJob(job: DownloadJob): DownloadJob {
  return {
    ...job,
    status: 'paused',
    speed: '--',
    timeRemaining: '',
  };
}

export function resumeTransferJob(job: DownloadJob): DownloadJob {
  return {
    ...job,
    status: 'downloading',
    speed: 'Downloading...',
    timeRemaining: 'Calculating...',
    errorMessage: undefined,
  };
}

export function downloadGrantFromJob(job: DownloadJob): DownloadGrantLike {
  return {
    id: job.fileId,
    packageName: job.fileName,
    downloadUrl: job.downloadUrl,
    signedSourceUrl: job.signedSourceUrl,
    method: job.downloadMethod,
    expiresAtEpochMs: job.expiresAtEpochMs,
    totalBytes: job.totalSize,
    archiveSizeBytes: job.downloadKind === 'bundle' ? job.totalSize : undefined,
  };
}

export function isCompletedTransferStatus(status: DownloadJob['status']): boolean {
  return status === 'ready' || status === 'completed';
}

export function cancelTransferJob(job: DownloadJob): DownloadJob {
  return {
    ...job,
    status: 'cancelled',
    speed: '--',
    timeRemaining: '',
  };
}

function makeJobId(): string {
  const generatedId = globalThis.crypto?.randomUUID?.();
  if (generatedId) {
    return generatedId;
  }
  fallbackJobIdCounter += 1;
  return `drive-job-${Date.now().toString(36)}-${fallbackJobIdCounter.toString(36)}`;
}

function fileSizeOrFallback(file: DriveFile, fallbackSizeBytes: number): number {
  if (file.size && file.size > 0) {
    return file.size;
  }
  return file.type === 'folder' ? DEFAULT_FOLDER_ARCHIVE_SIZE_BYTES : fallbackSizeBytes;
}

function normalizeDerivedSection(section?: string): string {
  if (!section || VIEW_SECTIONS.has(section)) {
    return 'my-storage';
  }
  return section;
}

export function resolveDriveSectionForDerivedFile(file: DriveFile, activeSection?: string): string {
  return normalizeDerivedSection(file.spaceId || activeSection);
}

export function createDownloadJobForFiles(
  files: DriveFile[],
  options: CreateDownloadJobOptions = {},
): DownloadJob {
  if (files.length === 0) {
    throw new Error('At least one file is required to create a Drive transfer job.');
  }

  const fallbackSizeBytes = options.fallbackSizeBytes || DEFAULT_DOWNLOAD_SIZE_BYTES;
  if (files.length === 1) {
    const file = files[0];
    return {
      id: options.id || makeJobId(),
      type: 'download',
      downloadKind: 'single',
      sourceNodeIds: [file.id],
      fileId: file.id,
      fileName: file.type === 'folder' ? `${file.name}.zip` : file.name,
      fileType: file.type,
      mimeType: file.mimeType,
      totalSize: fileSizeOrFallback(file, fallbackSizeBytes),
      downloadedSize: 0,
      progress: 0,
      status: 'connecting',
      speed: 'Connecting...',
      timeRemaining: 'Calculating...',
    };
  }

  const packageName = options.packageName || `drive_export_${files.length}_items.zip`;
  return {
    id: options.id || makeJobId(),
    type: 'download',
    downloadKind: 'bundle',
    sourceNodeIds: files.map((file) => file.id),
    fileId: 'batch-archive',
    fileName: packageName,
    fileType: 'file',
    mimeType: 'application/zip',
    totalSize: files.reduce((sum, file) => sum + fileSizeOrFallback(file, fallbackSizeBytes), 0),
    downloadedSize: 0,
    progress: 0,
    status: 'connecting',
    speed: 'Connecting...',
    timeRemaining: 'Calculating...',
  };
}

export function createUploadJobForFile(file: File, options: CreateUploadJobOptions = {}): DownloadJob {
  const fallbackSizeBytes = options.fallbackSizeBytes || DEFAULT_DOWNLOAD_SIZE_BYTES;
  return {
    id: options.id || makeJobId(),
    type: 'upload',
    uploadSection: options.uploadSection,
    uploadParentId: options.uploadParentId,
    uploadBlob: options.uploadBlob,
    uploadFileFingerprint: buildUploadFileFingerprint(file),
    fileId: options.fileId || makeJobId(),
    fileName: file.name,
    fileType: 'file',
    mimeType: file.type || 'application/octet-stream',
    totalSize: file.size || fallbackSizeBytes,
    downloadedSize: 0,
    progress: 0,
    status: 'uploading',
    speed: 'Uploading...',
    timeRemaining: 'Waiting for backend confirmation',
  };
}

export function createUploadJobForNativeFile(
  descriptor: NativeUploadJobDescriptor,
  options: CreateUploadJobOptions = {},
): DownloadJob {
  const fallbackSizeBytes = options.fallbackSizeBytes || DEFAULT_DOWNLOAD_SIZE_BYTES;
  return {
    id: options.id || makeJobId(),
    type: 'upload',
    uploadSection: options.uploadSection,
    uploadParentId: options.uploadParentId,
    uploadLocalPath: descriptor.path,
    uploadFileFingerprint: buildNativeUploadJobFingerprint(descriptor),
    fileId: options.fileId || makeJobId(),
    fileName: descriptor.name,
    fileType: 'file',
    mimeType: descriptor.mimeType || 'application/octet-stream',
    totalSize: descriptor.size || fallbackSizeBytes,
    downloadedSize: 0,
    progress: 0,
    status: 'uploading',
    speed: 'Uploading...',
    timeRemaining: 'Waiting for backend confirmation',
  };
}

export function createRetryFilesForDownloadJob(job: DownloadJob): DriveFile[] {
  const sourceNodeIds = job.sourceNodeIds && job.sourceNodeIds.length > 0 ? job.sourceNodeIds : [job.fileId];
  const fallbackType = job.fileType === 'folder' ? 'folder' : 'file';
  const now = new Date().toISOString();

  return sourceNodeIds.map((nodeId, index) => {
    const isSingleFolder = sourceNodeIds.length === 1 && fallbackType === 'folder';
    return {
      id: nodeId,
      name: sourceNodeIds.length === 1 ? job.fileName : `${job.fileName.replace(/\.zip$/i, '')}-${index + 1}`,
      type: isSingleFolder ? 'folder' : 'file',
      mimeType: sourceNodeIds.length === 1 ? job.mimeType : undefined,
      size: sourceNodeIds.length === 1 ? job.totalSize : undefined,
      updatedAt: now,
      ownerId: '',
    };
  });
}

export function resolveTransferOpenUrl(
  grant: Pick<DownloadGrantLike, 'downloadUrl' | 'signedSourceUrl'>,
): string | undefined {
  return grant.signedSourceUrl || grant.downloadUrl;
}

export function applyDownloadGrantToJob(job: DownloadJob, grant: DownloadGrantLike): DownloadJob {
  if (job.status === 'cancelled') {
    return job;
  }

  const grantedSize = grant.archiveSizeBytes || grant.totalBytes || job.totalSize;
  return {
    ...job,
    fileId: grant.id || job.fileId,
    downloadUrl: grant.downloadUrl || job.downloadUrl,
    signedSourceUrl: grant.signedSourceUrl || job.signedSourceUrl,
    downloadMethod: grant.method || job.downloadMethod || 'GET',
    expiresAtEpochMs: grant.expiresAtEpochMs || job.expiresAtEpochMs,
    totalSize: grantedSize,
    downloadedSize: 0,
    progress: 0,
    status: 'ready',
    speed: 'Ready',
    timeRemaining: 'Available',
    errorMessage: undefined,
  };
}

export function applyDownloadProgressToJob(
  job: DownloadJob,
  downloadedBytes: number,
  totalBytes: number,
): DownloadJob {
  if (job.status === 'cancelled') {
    return job;
  }

  const safeTotal = totalBytes > 0 ? totalBytes : job.totalSize;
  const safeDownloaded = safeTotal > 0 ? Math.max(0, Math.min(downloadedBytes, safeTotal)) : Math.max(0, downloadedBytes);
  const progress = safeTotal > 0 ? Math.min(100, Math.round((safeDownloaded / safeTotal) * 100)) : 0;
  return {
    ...job,
    totalSize: safeTotal > 0 ? safeTotal : job.totalSize,
    downloadedSize: safeDownloaded,
    progress,
    status: 'downloading',
    speed: 'Downloading...',
    timeRemaining: progress >= 100 ? 'Finishing...' : 'Calculating...',
    errorMessage: undefined,
  };
}

export function applyDownloadCompletionToJob(job: DownloadJob, actualSizeBytes?: number): DownloadJob {
  if (job.status === 'cancelled') {
    return job;
  }

  const totalSize = actualSizeBytes && actualSizeBytes > 0 ? actualSizeBytes : job.totalSize;
  return {
    ...job,
    totalSize,
    downloadedSize: totalSize,
    progress: 100,
    status: 'completed',
    speed: '--',
    timeRemaining: '',
    errorMessage: undefined,
  };
}

export function applyUploadProgressToJob(
  job: DownloadJob,
  uploadedBytes: number,
  totalBytes: number,
): DownloadJob {
  if (job.status === 'cancelled') {
    return job;
  }

  const safeTotal = totalBytes > 0 ? totalBytes : job.totalSize;
  const safeUploaded = Math.max(0, Math.min(uploadedBytes, safeTotal));
  const progress = safeTotal > 0 ? Math.round((safeUploaded / safeTotal) * 100) : 0;
  return {
    ...job,
    totalSize: safeTotal,
    downloadedSize: safeUploaded,
    progress,
    status: 'uploading',
    speed: 'Uploading...',
    timeRemaining: progress >= 100 ? 'Finalizing...' : 'Calculating...',
  };
}

export function applyUploadCompletionToJob(job: DownloadJob, file: DriveFile): DownloadJob {
  if (job.status === 'cancelled') {
    return job;
  }

  const totalSize = file.size || job.totalSize;
  return {
    ...job,
    fileId: file.id,
    fileName: file.name || job.fileName,
    mimeType: file.mimeType || job.mimeType,
    totalSize,
    downloadedSize: totalSize,
    progress: 100,
    status: 'completed',
    speed: '--',
    timeRemaining: '',
    errorMessage: undefined,
  };
}

export function applyTransferFailure(job: DownloadJob, message?: string): DownloadJob {
  if (job.status === 'cancelled') {
    return job;
  }

  return {
    ...job,
    status: 'failed',
    speed: '--',
    timeRemaining: '',
    errorMessage: message,
  };
}
