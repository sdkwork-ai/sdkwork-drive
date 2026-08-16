import {
  applyDownloadGrantToJob,
  applyTransferFailure,
  applyUploadCompletionToJob,
  canCancelTransferJob,
  canControlTransferJob,
  canPauseTransferJob,
  canResumeTransferJob,
  pauseTransferJob,
  createDownloadJobForFiles,
  createRetryFilesForDownloadJob,
  createUploadJobForFile,
  createUploadJobForNativeFile,
  type DriveFile,
  isActiveTransferStatus,
  resolveDriveSectionForDerivedFile,
  resolveTransferOpenUrl,
} from 'sdkwork-drive-pc-types';
import { describe, expect, it } from 'vitest';

const sourceFile: DriveFile = {
  id: 'file-001',
  name: 'Roadmap.pdf',
  type: 'file',
  mimeType: 'application/pdf',
  size: 4096,
  updatedAt: '2026-01-01T00:00:00.000Z',
  ownerId: 'Ada',
};

describe('drive transfer job helpers', () => {
  it('creates a single-file download job from the selected file metadata', () => {
    const job = createDownloadJobForFiles([sourceFile], {
      id: 'job-001',
      fallbackSizeBytes: 10_000,
    });

    expect(job).toMatchObject({
      id: 'job-001',
      type: 'download',
      downloadKind: 'single',
      sourceNodeIds: ['file-001'],
      fileId: 'file-001',
      fileName: 'Roadmap.pdf',
      fileType: 'file',
      mimeType: 'application/pdf',
      totalSize: 4096,
      downloadedSize: 0,
      progress: 0,
      status: 'connecting',
    });
  });

  it('creates a multi-node archive job with deterministic package metadata', () => {
    const job = createDownloadJobForFiles(
      [
        sourceFile,
        {
          ...sourceFile,
          id: 'folder-001',
          name: 'Reports',
          type: 'folder',
          size: undefined,
        },
      ],
      {
        id: 'job-002',
        packageName: 'drive_export_2_items.zip',
        fallbackSizeBytes: 5_000_000,
      },
    );

    expect(job).toMatchObject({
      id: 'job-002',
      type: 'download',
      downloadKind: 'bundle',
      sourceNodeIds: ['file-001', 'folder-001'],
      fileId: 'batch-archive',
      fileName: 'drive_export_2_items.zip',
      fileType: 'file',
      mimeType: 'application/zip',
      totalSize: 18_504_096,
      status: 'connecting',
    });
  });

  it('stores Drive download grant URLs and expiry on the transfer job', () => {
    const job = createDownloadJobForFiles([sourceFile], {
      id: 'job-003',
    });

    const updated = applyDownloadGrantToJob(job, {
      downloadUrl: 'https://drive.example.test/download/file-001',
      signedSourceUrl: 'https://storage.example.test/file-001',
      expiresAtEpochMs: 1_800_000_000_000,
      method: 'GET',
      totalBytes: 8192,
    });

    expect(updated).toMatchObject({
      fileId: 'file-001',
      downloadUrl: 'https://drive.example.test/download/file-001',
      signedSourceUrl: 'https://storage.example.test/file-001',
      downloadMethod: 'GET',
      expiresAtEpochMs: 1_800_000_000_000,
      totalSize: 8192,
      downloadedSize: 0,
      progress: 0,
      status: 'ready',
      speed: 'Ready',
      timeRemaining: 'Available',
    });
  });

  it('stores Drive archive package ids and archive size on bundle transfer jobs', () => {
    const job = createDownloadJobForFiles([sourceFile, { ...sourceFile, id: 'file-002' }], {
      id: 'job-004',
      packageName: 'drive_export_2_items.zip',
    });

    const updated = applyDownloadGrantToJob(job, {
      id: 'package-001',
      downloadUrl: 'https://drive.example.test/download/package-001',
      expiresAtEpochMs: 1_800_000_000_000,
      method: 'GET',
      packageName: 'drive_export_2_items.zip',
      fileCount: 2,
      totalBytes: 8192,
      archiveSizeBytes: 4096,
    });

    expect(updated).toMatchObject({
      fileId: 'package-001',
      totalSize: 4096,
      downloadedSize: 0,
      progress: 0,
      downloadUrl: 'https://drive.example.test/download/package-001',
      status: 'ready',
    });
  });

  it('keeps cancelled transfers cancelled when late grants or failures arrive', () => {
    const job = createDownloadJobForFiles([sourceFile], {
      id: 'job-cancelled-late-grant',
    });
    const uploadJob = createUploadJobForFile(
      new File(['hello'], 'Roadmap.pdf', { type: 'application/pdf' }),
      {
        id: 'job-cancelled-late-upload',
        fileId: 'upload-temp-file',
      },
    );
    const cancelled = {
      ...job,
      status: 'cancelled' as const,
      speed: '--',
      timeRemaining: '',
    };
    const cancelledUpload = {
      ...uploadJob,
      status: 'cancelled' as const,
      speed: '--',
      timeRemaining: '',
    };

    expect(
      applyDownloadGrantToJob(cancelled, {
        downloadUrl: 'https://drive.example.test/download/file-001',
        expiresAtEpochMs: 1_800_000_000_000,
        method: 'GET',
      }),
    ).toEqual(cancelled);
    expect(
      applyTransferFailure(cancelled, 'Backend grant completed after cancellation.'),
    ).toEqual(cancelled);
    expect(
      applyUploadCompletionToJob(cancelledUpload, sourceFile),
    ).toEqual(cancelledUpload);
  });

  it('reconstructs minimal Drive files from transfer source ids for retry requests', () => {
    const single = createRetryFilesForDownloadJob(
      createDownloadJobForFiles([
        {
          ...sourceFile,
          type: 'folder',
          name: 'Reports',
        },
      ]),
    );
    const bundle = createRetryFilesForDownloadJob(
      createDownloadJobForFiles([sourceFile, { ...sourceFile, id: 'file-002' }], {
        packageName: 'drive_export_2_items.zip',
      }),
    );

    expect(single).toEqual([
      expect.objectContaining({
        id: 'file-001',
        name: 'Reports.zip',
        type: 'folder',
      }),
    ]);
    expect(bundle.map((file) => file.id)).toEqual(['file-001', 'file-002']);
  });

  it('keeps downloads pending until a real App SDK download grant is available', () => {
    const job = createDownloadJobForFiles([sourceFile], { id: 'job-005' });

    expect(job).toMatchObject({
      status: 'connecting',
      progress: 0,
      downloadedSize: 0,
    });
    expect(job).not.toHaveProperty('downloadUrl');
  });

  it('keeps upload retry context on upload transfer jobs', async () => {
    const sourceBlob = new File(['hello world'], 'notes.txt', { type: 'text/plain' });
    const job = createUploadJobForFile(sourceBlob, {
      id: 'job-upload-retry',
      fileId: 'upload-temp-file',
      uploadSection: 'my-storage',
      uploadParentId: 'folder-001',
      uploadBlob: sourceBlob,
    });

    expect(job).toMatchObject({
      id: 'job-upload-retry',
      type: 'upload',
      uploadSection: 'my-storage',
      uploadParentId: 'folder-001',
      fileName: 'notes.txt',
      status: 'uploading',
    });
    expect(job.uploadBlob).toBe(sourceBlob);
  });

  it('keeps native upload retry context on desktop transfer jobs', () => {
    const job = createUploadJobForNativeFile(
      {
        path: 'C:\\\\Users\\\\demo\\\\Roadmap.pdf',
        name: 'Roadmap.pdf',
        size: 4096,
        modifiedAt: '1718611200000',
        mimeType: 'application/pdf',
      },
      {
        id: 'job-native-upload',
        uploadSection: 'my-storage',
        uploadParentId: 'folder-001',
      },
    );

    expect(job).toMatchObject({
      type: 'upload',
      uploadSection: 'my-storage',
      uploadParentId: 'folder-001',
      uploadLocalPath: 'C:\\\\Users\\\\demo\\\\Roadmap.pdf',
      uploadFileFingerprint: 'C:\\\\Users\\\\demo\\\\Roadmap.pdf:4096:1718611200000',
      fileName: 'Roadmap.pdf',
    });
    expect(job.uploadBlob).toBeUndefined();
  });

  it('resolves derived files into the source file space before falling back to the active section', () => {
    expect(
      resolveDriveSectionForDerivedFile({
        ...sourceFile,
        spaceId: 'finance',
      }),
    ).toBe('finance');
    expect(resolveDriveSectionForDerivedFile(sourceFile, 'design')).toBe('design');
  });

  it('treats uploads as active cancellable transfers until backend confirmation', () => {
    const uploadJob = createUploadJobForFile(
      new File(['hello'], 'Roadmap.pdf', { type: 'application/pdf' }),
      {
        id: 'job-upload',
        fileId: 'temp-file',
      },
    );

    expect(isActiveTransferStatus('uploading')).toBe(true);
    expect(isActiveTransferStatus('paused')).toBe(false);
    expect(canControlTransferJob(uploadJob)).toBe(true);
    expect(uploadJob).toMatchObject({
      status: 'uploading',
      speed: 'Uploading...',
    });
  });

  it('supports download pause and resume when the grant is still valid', () => {
    const uploadJob = createUploadJobForFile(
      new File(['hello'], 'Roadmap.pdf', { type: 'application/pdf' }),
      {
        id: 'job-upload',
        fileId: 'temp-file',
      },
    );
    const downloadJob = applyDownloadGrantToJob(
      createDownloadJobForFiles([sourceFile], {
        id: 'job-download',
      }),
      {
        downloadUrl: 'https://example.test/file',
        expiresAtEpochMs: Date.now() + 60_000,
      },
    );
    const downloading = { ...downloadJob, status: 'downloading' as const };
    expect(canPauseTransferJob(downloading)).toBe(true);
    const paused = pauseTransferJob(downloading);
    expect(paused.status).toBe('paused');
    expect(canResumeTransferJob(paused)).toBe(true);
    expect(canPauseTransferJob({ ...uploadJob, status: 'uploading' })).toBe(false);
    expect(canResumeTransferJob({ ...uploadJob, status: 'paused' })).toBe(false);
  });

  it('keeps uploads pending backend confirmation and completes with the real Drive node id', () => {
    const uploadJob = createUploadJobForFile(
      new File(['hello'], 'Roadmap.pdf', { type: 'application/pdf' }),
      {
        id: 'job-upload-complete',
        fileId: 'temp-file',
      },
    );

    const completed = applyUploadCompletionToJob(uploadJob, {
      ...sourceFile,
      id: 'drive-node-001',
      size: 5,
    });

    expect(uploadJob).toMatchObject({
      fileId: 'temp-file',
      status: 'uploading',
      progress: 0,
      downloadedSize: 0,
    });
    expect(completed).toMatchObject({
      fileId: 'drive-node-001',
      status: 'completed',
      progress: 100,
      downloadedSize: 5,
      totalSize: 5,
    });
  });

  it('prefers signed source URLs when opening completed transfer grants', () => {
    expect(
      resolveTransferOpenUrl({
        downloadUrl: 'https://drive.example.test/download/package-001',
        signedSourceUrl: 'https://storage.example.test/package-001',
      }),
    ).toBe('https://storage.example.test/package-001');
    expect(
      resolveTransferOpenUrl({
        downloadUrl: 'https://drive.example.test/download/file-001',
      }),
    ).toBe('https://drive.example.test/download/file-001');
  });
});
