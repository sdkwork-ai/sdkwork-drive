import { describe, expect, it, vi } from 'vitest';
import type { DownloadJob } from '../types';
import {
  applyDownloadCompletionToJob,
  applyDownloadGrantToJob,
  applyDownloadProgressToJob,
  createDownloadJobForFiles,
} from '../types';
import { executeDownloadTransfer, isDriveAbortError, runManagedDownloadTransfer } from './downloadTransfer';

const sourceFile = {
  id: 'file-001',
  name: 'Roadmap.pdf',
  type: 'file' as const,
  mimeType: 'application/pdf',
  size: 4096,
  updatedAt: '2026-01-01T00:00:00.000Z',
  ownerId: 'Ada',
};

describe('downloadTransfer', () => {
  it('recognizes abort errors for cancellable transfer flows', () => {
    expect(isDriveAbortError(new DOMException('aborted', 'AbortError'))).toBe(true);
    expect(isDriveAbortError(new Error('The operation was aborted'))).toBe(true);
    expect(isDriveAbortError(new Error('network failure'))).toBe(false);
  });

  it('reports byte progress while streaming a download body', async () => {
    const payload = new Uint8Array([1, 2, 3, 4, 5]);
    const fetchImpl = vi.fn(async () =>
      new Response(
        new ReadableStream({
          start(controller) {
            controller.enqueue(payload.subarray(0, 2));
            controller.enqueue(payload.subarray(2));
            controller.close();
          },
        }),
        {
          status: 200,
          headers: {
            'content-type': 'application/pdf',
            'content-length': String(payload.byteLength),
          },
        },
      ),
    );
    const progress: Array<[number, number]> = [];
    const job = createDownloadJobForFiles([sourceFile], { id: 'job-stream' });

    const result = await executeDownloadTransfer(
      job,
      {
        signedSourceUrl: 'https://storage.example.test/file-001',
        method: 'GET',
      },
      {
        fetchImpl,
        onProgress: (downloadedBytes, totalBytes) => {
          progress.push([downloadedBytes, totalBytes]);
        },
      },
    );

    expect(result.fileName).toBe('Roadmap.pdf');
    expect(result.blob.size).toBe(payload.byteLength);
    expect(progress.at(-1)).toEqual([payload.byteLength, payload.byteLength]);
    expect(fetchImpl).toHaveBeenCalledWith(
      'https://storage.example.test/file-001',
      expect.objectContaining({ method: 'GET' }),
    );
  });

  it('keeps grant progress at zero until bytes are transferred', () => {
    const job = createDownloadJobForFiles([sourceFile], { id: 'job-grant' });
    const granted = applyDownloadGrantToJob(job, {
      signedSourceUrl: 'https://storage.example.test/file-001',
      totalBytes: 8192,
    });
    const downloading = applyDownloadProgressToJob(granted, 2048, 8192);
    const completed = applyDownloadCompletionToJob(downloading, 8192);

    expect(granted).toMatchObject({
      status: 'ready',
      progress: 0,
      downloadedSize: 0,
      totalSize: 8192,
    });
    expect(downloading).toMatchObject({
      status: 'downloading',
      progress: 25,
      downloadedSize: 2048,
    });
    expect(completed).toMatchObject({
      status: 'completed',
      progress: 100,
      downloadedSize: 8192,
    });
  });

  it('drives managed download job states from grant through completion', async () => {
    const payload = new Uint8Array([9, 8, 7]);
    const fetchImpl = vi.fn(async () =>
      new Response(payload, {
        status: 200,
        headers: {
          'content-type': 'application/pdf',
          'content-length': String(payload.byteLength),
        },
      }),
    );
    const anchor = {
      href: '',
      download: '',
      rel: '',
      click: vi.fn(),
      remove: vi.fn(),
    };
    vi.stubGlobal('document', {
      createElement: vi.fn(() => anchor),
      body: {
        appendChild: vi.fn(),
        removeChild: vi.fn(),
      },
    });
    vi.stubGlobal('URL', {
      createObjectURL: vi.fn(() => 'blob:managed-download'),
      revokeObjectURL: vi.fn(),
    });

    const job = createDownloadJobForFiles([sourceFile], { id: 'job-managed' });
    const snapshots: DownloadJob[] = [];
    await runManagedDownloadTransfer({
      job,
      grant: {
        signedSourceUrl: 'https://storage.example.test/file-001',
        totalBytes: payload.byteLength,
      },
      fetchImpl,
      onJobUpdate: (updater) => {
        const current = snapshots.at(-1) ?? job;
        snapshots.push(updater(current));
      },
    });

    expect(snapshots[0]?.status).toBe('ready');
    expect(snapshots[0]?.progress).toBe(0);
    expect(snapshots.some((snapshot) => snapshot.status === 'downloading')).toBe(true);
    expect(snapshots.at(-1)).toMatchObject({
      status: 'completed',
      progress: 100,
      downloadedSize: payload.byteLength,
    });
    expect(anchor.click).toHaveBeenCalled();
  });

  it('uses native save handler when provided instead of browser anchor download', async () => {
    const payload = new Uint8Array([4, 3, 2, 1]);
    const fetchImpl = vi.fn(async () =>
      new Response(payload, {
        status: 200,
        headers: {
          'content-type': 'application/pdf',
          'content-length': String(payload.byteLength),
        },
      }),
    );
    const saveDownload = vi.fn(async (_fileName: string, _blob: Blob) => true);
    const anchor = {
      href: '',
      download: '',
      rel: '',
      click: vi.fn(),
      remove: vi.fn(),
    };
    vi.stubGlobal('document', {
      createElement: vi.fn(() => anchor),
      body: {
        appendChild: vi.fn(),
        removeChild: vi.fn(),
      },
    });

    const job = createDownloadJobForFiles([sourceFile], { id: 'job-native-save' });
    await runManagedDownloadTransfer({
      job,
      grant: {
        signedSourceUrl: 'https://storage.example.test/file-001',
        totalBytes: payload.byteLength,
      },
      fetchImpl,
      saveDownload,
      onJobUpdate: () => {},
    });

    expect(saveDownload).toHaveBeenCalledTimes(1);
    expect(saveDownload.mock.calls[0]?.[0]).toBe('Roadmap.pdf');
    expect(anchor.click).not.toHaveBeenCalled();
  });

  it('streams download chunks through native save session when saveDownloadStream is provided', async () => {
    const payload = new Uint8Array([9, 8, 7, 6, 5]);
    const fetchImpl = vi.fn(async () =>
      new Response(payload, {
        status: 200,
        headers: {
          'content-type': 'application/octet-stream',
          'content-length': String(payload.byteLength),
        },
      }),
    );
    const saveDownloadStream = {
      begin: vi.fn(async () => 'session-1'),
      writeChunk: vi.fn(async () => undefined),
      finish: vi.fn(async () => true),
      abort: vi.fn(async () => undefined),
    };
    const anchor = {
      href: '',
      download: '',
      rel: '',
      click: vi.fn(),
      remove: vi.fn(),
    };
    vi.stubGlobal('document', {
      createElement: vi.fn(() => anchor),
      body: {
        appendChild: vi.fn(),
        removeChild: vi.fn(),
      },
    });

    const job = createDownloadJobForFiles([sourceFile], { id: 'job-stream-save' });
    await runManagedDownloadTransfer({
      job,
      grant: {
        signedSourceUrl: 'https://storage.example.test/file-001',
        totalBytes: payload.byteLength,
      },
      fetchImpl,
      saveDownloadStream,
      onJobUpdate: () => {},
    });

    expect(saveDownloadStream.begin).toHaveBeenCalledWith('Roadmap.pdf');
    expect(saveDownloadStream.writeChunk).toHaveBeenCalled();
    expect(saveDownloadStream.finish).toHaveBeenCalledWith('session-1');
    expect(saveDownloadStream.abort).not.toHaveBeenCalled();
    expect(anchor.click).not.toHaveBeenCalled();
  });

  it('marks transfer failed when download fetch fails even if external open succeeds', async () => {
    const fetchImpl = vi.fn(async () => new Response(null, { status: 503 }));
    const onOpenExternal = vi.fn(async () => undefined);
    const job = createDownloadJobForFiles([sourceFile], { id: 'job-external-fallback' });
    const snapshots: DownloadJob[] = [];
    await runManagedDownloadTransfer({
      job,
      grant: {
        signedSourceUrl: 'https://storage.example.test/file-001',
        totalBytes: 1024,
      },
      fetchImpl,
      onOpenExternal,
      onJobUpdate: (updater) => {
        const current = snapshots.at(-1) ?? job;
        snapshots.push(updater(current));
      },
    });

    expect(onOpenExternal).toHaveBeenCalledWith('https://storage.example.test/file-001');
    expect(snapshots.at(-1)).toMatchObject({
      status: 'failed',
    });
    expect(snapshots.at(-1)?.errorMessage).toContain('opened the source URL');
  });

  it('rejects in-memory downloads larger than the configured byte limit', async () => {
    const oversizedChunk = new Uint8Array(64 * 1024 * 1024 + 1);
    const fetchImpl = vi.fn(async () =>
      new Response(
        new ReadableStream({
          start(controller) {
            controller.enqueue(oversizedChunk);
            controller.close();
          },
        }),
        {
          status: 200,
          headers: {
            'content-type': 'application/octet-stream',
          },
        },
      ),
    );
    const job = createDownloadJobForFiles([sourceFile], { id: 'job-oom' });

    await expect(
      executeDownloadTransfer(
        job,
        {
          signedSourceUrl: 'https://storage.example.test/file-001',
        },
        { fetchImpl },
      ),
    ).rejects.toThrow(/too large for in-memory handling/);
  });

  it('requests a byte range when resuming an interrupted download', async () => {
    const payload = new Uint8Array([1, 2, 3, 4]);
    const fetchImpl = vi.fn(async (_url: string, init?: RequestInit) =>
      new Response(payload, {
        status: 206,
        headers: {
          'content-type': 'application/octet-stream',
          'content-length': String(payload.byteLength),
        },
      }),
    );

    const job = createDownloadJobForFiles([sourceFile], {
      id: 'job-resume',
    });
    job.totalSize = 10;
    job.downloadedSize = 6;

    await executeDownloadTransfer(
      job,
      {
        signedSourceUrl: 'https://storage.example.test/file-001',
        totalBytes: 10,
      },
      { fetchImpl },
    );

    expect(fetchImpl).toHaveBeenCalledWith(
      'https://storage.example.test/file-001',
      expect.objectContaining({
        headers: {
          Range: 'bytes=6-',
        },
      }),
    );
  });
});
