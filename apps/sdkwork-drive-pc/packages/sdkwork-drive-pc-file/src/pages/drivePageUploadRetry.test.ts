import { describe, expect, it } from 'vitest';
import type { DownloadJob } from 'sdkwork-drive-pc-types';
import { buildUploadRetryMismatchMessage, isRetryUploadFileCompatible } from './DrivePage';

function makeUploadJob(overrides: Partial<DownloadJob> = {}): DownloadJob {
  return {
    id: 'job-upload-001',
    type: 'upload',
    fileId: 'temp-file',
    fileName: 'Roadmap.pdf',
    fileType: 'file',
    totalSize: 1024,
    downloadedSize: 0,
    progress: 0,
    status: 'failed',
    speed: '--',
    timeRemaining: '',
    ...overrides,
  };
}

describe('DrivePage upload retry compatibility', () => {
  it('accepts the original file name and size', () => {
    const selected = new File([new Uint8Array(1024)], 'Roadmap.pdf', {
      type: 'application/pdf',
    });
    const job = makeUploadJob({
      uploadFileFingerprint: `${selected.name}:${selected.size}:${selected.lastModified}`,
    });
    expect(isRetryUploadFileCompatible(job, selected)).toBe(true);
  });

  it('rejects files that do not match the original upload task', () => {
    const base = new File([new Uint8Array(1024)], 'Roadmap.pdf', {
      type: 'application/pdf',
    });
    const job = makeUploadJob({
      uploadFileFingerprint: `${base.name}:${base.size}:${base.lastModified}`,
    });
    const wrongName = new File([new Uint8Array(1024)], 'Other.pdf', {
      type: 'application/pdf',
    });
    const wrongSize = new File([new Uint8Array(2048)], 'Roadmap.pdf', {
      type: 'application/pdf',
    });

    expect(isRetryUploadFileCompatible(job, wrongName)).toBe(false);
    expect(isRetryUploadFileCompatible(job, wrongSize)).toBe(false);
  });

  it('falls back to name and size when fingerprint is missing', () => {
    const job = makeUploadJob();
    const selected = new File([new Uint8Array(1024)], 'Roadmap.pdf', {
      type: 'application/pdf',
    });
    const mismatch = new File([new Uint8Array(1024)], 'Roadmap-v2.pdf', {
      type: 'application/pdf',
    });

    expect(isRetryUploadFileCompatible(job, selected)).toBe(true);
    expect(isRetryUploadFileCompatible(job, mismatch)).toBe(false);
  });

  it('builds mismatch message with expected file details', () => {
    const message = buildUploadRetryMismatchMessage(
      makeUploadJob({
        fileName: 'Roadmap.pdf',
        totalSize: 2048,
      }),
    );
    expect(message).toContain('Roadmap.pdf');
    expect(message).toContain('2.0 KB');
    expect(message).toContain('modified');
  });

  it('uses fingerprint details when available in mismatch message', () => {
    const selected = new File([new Uint8Array(1024)], 'Roadmap.pdf', {
      type: 'application/pdf',
      lastModified: 1718611200000,
    });
    const message = buildUploadRetryMismatchMessage(
      makeUploadJob({
        fileName: 'LegacyName.pdf',
        totalSize: 1,
        uploadFileFingerprint: `${selected.name}:${selected.size}:${selected.lastModified}`,
      }),
    );
    expect(message).toContain('Roadmap.pdf');
    expect(message).toContain('1.0 KB');
    expect(message).toContain('2024-06-17 08:00:00Z');
  });
});
