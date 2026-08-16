import { afterEach, describe, expect, it } from 'vitest';
import type { DownloadJob } from '../types';
import {
  TRANSFER_INTERRUPTION_TRANSFER_RETRY,
  TRANSFER_INTERRUPTION_UPLOAD_NATIVE_RETRY,
  TRANSFER_INTERRUPTION_UPLOAD_RESELECT,
} from './transferInterruptionCodes';
import { loadPersistedTransferJobs, persistTransferJobs } from './transferJobStore';

const STORAGE_KEY = 'sdkwork.drive.pc.transfer.jobs.v1';

function installMockWindowStorage(): Map<string, string> {
  const backing = new Map<string, string>();
  const localStorage = {
    getItem(key: string): string | null {
      return backing.has(key) ? backing.get(key)! : null;
    },
    setItem(key: string, value: string): void {
      backing.set(key, value);
    },
    removeItem(key: string): void {
      backing.delete(key);
    },
  };
  Object.defineProperty(globalThis, 'window', {
    configurable: true,
    value: { localStorage },
  });
  return backing;
}

function makeJob(overrides: Partial<DownloadJob> = {}): DownloadJob {
  return {
    id: 'job-001',
    type: 'upload',
    fileId: 'file-001',
    fileName: 'Roadmap.pdf',
    fileType: 'file',
    totalSize: 1024,
    downloadedSize: 0,
    progress: 0,
    status: 'uploading',
    speed: 'Uploading...',
    timeRemaining: 'Calculating...',
    ...overrides,
  };
}

afterEach(() => {
  Reflect.deleteProperty(globalThis, 'window');
});

describe('transfer job store', () => {
  it('persists jobs without non-serializable upload blobs', () => {
    const backing = installMockWindowStorage();
    persistTransferJobs([
      makeJob({
        uploadBlob: new File(['hello'], 'Roadmap.pdf', { type: 'application/pdf' }),
      }),
    ]);

    const raw = backing.get(STORAGE_KEY);
    expect(raw).toBeTruthy();
    expect(raw).not.toContain('uploadBlob');
    const stored = JSON.parse(raw ?? '[]') as DownloadJob[];
    expect(stored[0].uploadBlob).toBeUndefined();
  });

  it('strips signed download URLs from completed jobs before persistence', () => {
    const backing = installMockWindowStorage();
    persistTransferJobs([
      makeJob({
        type: 'download',
        status: 'completed',
        downloadUrl: 'https://signed.example.test/download',
        signedSourceUrl: 'https://signed.example.test/source',
      }),
    ]);

    const raw = backing.get(STORAGE_KEY);
    const stored = JSON.parse(raw ?? '[]') as DownloadJob[];
    expect(stored[0].downloadUrl).toBeUndefined();
    expect(stored[0].signedSourceUrl).toBeUndefined();
  });

  it('retains signed download URLs for paused jobs so resume can continue', () => {
    const backing = installMockWindowStorage();
    persistTransferJobs([
      makeJob({
        type: 'download',
        status: 'paused',
        downloadUrl: 'https://signed.example.test/download',
        signedSourceUrl: 'https://signed.example.test/source',
        expiresAtEpochMs: Date.now() + 60_000,
      }),
    ]);

    const raw = backing.get(STORAGE_KEY);
    const stored = JSON.parse(raw ?? '[]') as DownloadJob[];
    expect(stored[0].downloadUrl).toBe('https://signed.example.test/download');
    expect(stored[0].signedSourceUrl).toBe('https://signed.example.test/source');
  });

  it('restores interrupted active jobs as failed retryable jobs', () => {
    const backing = installMockWindowStorage();
    backing.set(
      STORAGE_KEY,
      JSON.stringify([
        makeJob({
          status: 'uploading',
          uploadSection: 'my-storage',
          uploadParentId: 'folder-001',
        }),
      ]),
    );

    const restored = loadPersistedTransferJobs();
    expect(restored).toHaveLength(1);
    expect(restored[0]).toMatchObject({
      status: 'failed',
      errorMessage: TRANSFER_INTERRUPTION_UPLOAD_RESELECT,
      uploadSection: 'my-storage',
      uploadParentId: 'folder-001',
    });
    expect(restored[0].uploadBlob).toBeUndefined();
  });

  it('restores native upload interruptions without forcing file re-selection', () => {
    const backing = installMockWindowStorage();
    backing.set(
      STORAGE_KEY,
      JSON.stringify([
        makeJob({
          status: 'uploading',
          uploadLocalPath: 'C:\\\\Users\\\\demo\\\\Roadmap.pdf',
          uploadFileFingerprint: 'C:\\\\Users\\\\demo\\\\Roadmap.pdf:1024:1718611200000',
        }),
      ]),
    );

    const restored = loadPersistedTransferJobs();
    expect(restored[0]).toMatchObject({
      status: 'failed',
      errorMessage: TRANSFER_INTERRUPTION_UPLOAD_NATIVE_RETRY,
      uploadLocalPath: 'C:\\\\Users\\\\demo\\\\Roadmap.pdf',
    });
  });

  it('keeps download interruptions as generic retryable transfer failures', () => {
    const backing = installMockWindowStorage();
    backing.set(
      STORAGE_KEY,
      JSON.stringify([
        makeJob({
          type: 'download',
          status: 'downloading',
          fileId: 'node-download',
          sourceNodeIds: ['node-download'],
        }),
      ]),
    );

    const restored = loadPersistedTransferJobs();
    expect(restored).toHaveLength(1);
    expect(restored[0]).toMatchObject({
      type: 'download',
      status: 'failed',
      errorMessage: TRANSFER_INTERRUPTION_TRANSFER_RETRY,
      sourceNodeIds: ['node-download'],
    });
  });
});
