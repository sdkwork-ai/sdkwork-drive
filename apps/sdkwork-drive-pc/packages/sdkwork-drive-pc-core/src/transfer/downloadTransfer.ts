import type { HostAdapter } from '../host/hostAdapter';
import type { DownloadGrantLike, DownloadJob } from '../types';
import {
  applyDownloadCompletionToJob,
  applyDownloadGrantToJob,
  applyDownloadProgressToJob,
  applyTransferFailure,
  resolveTransferOpenUrl,
} from '../types';

export interface ExecuteDownloadTransferOptions {
  signal?: AbortSignal;
  fetchImpl?: (input: string, init?: RequestInit) => Promise<Response>;
  onProgress?: (downloadedBytes: number, totalBytes: number) => void;
}

export interface ExecuteDownloadTransferResult {
  blob: Blob;
  fileName: string;
}

const MAX_IN_MEMORY_DOWNLOAD_BYTES = 64 * 1024 * 1024;

function formatInMemoryDownloadLimitError(receivedBytes: number): string {
  return `Download response is too large for in-memory handling (${receivedBytes} bytes; limit ${MAX_IN_MEMORY_DOWNLOAD_BYTES}). Use a browser that supports streamed save or the desktop client.`;
}

function assertInMemoryDownloadWithinLimit(receivedBytes: number): void {
  if (receivedBytes > MAX_IN_MEMORY_DOWNLOAD_BYTES) {
    throw new Error(formatInMemoryDownloadLimitError(receivedBytes));
  }
}

export interface SaveDownloadStreamAdapter {
  begin(fileName: string): Promise<string | null>;
  writeChunk(sessionId: string, chunk: Uint8Array): Promise<void>;
  finish(sessionId: string): Promise<boolean>;
  abort(sessionId: string): Promise<void>;
}

export function createHostDownloadStreamAdapter(host: HostAdapter): SaveDownloadStreamAdapter | undefined {
  if (!host.isNativeHost) {
    return undefined;
  }
  return {
    begin: (fileName) => host.beginDownloadSave(fileName),
    writeChunk: (sessionId, chunk) => host.writeDownloadChunk(sessionId, chunk),
    finish: (sessionId) => host.finishDownloadSave(sessionId),
    abort: (sessionId) => host.abortDownloadSave(sessionId),
  };
}

type BrowserWritableFileStream = {
  write(data: BufferSource): Promise<void>;
  close(): Promise<void>;
  abort(): Promise<void>;
};

type BrowserSaveFileHandle = {
  createWritable(): Promise<BrowserWritableFileStream>;
};

interface BrowserSaveFilePickerOptions {
  suggestedName?: string;
}

const BROWSER_DOWNLOAD_SESSION_ID = 'browser-download-session';

export async function createBrowserDownloadStreamAdapter(
  fileName: string,
): Promise<SaveDownloadStreamAdapter | undefined> {
  if (typeof globalThis.showSaveFilePicker !== 'function') {
    return undefined;
  }

  try {
    const handle = await globalThis.showSaveFilePicker({
      suggestedName: fileName,
    } satisfies BrowserSaveFilePickerOptions) as BrowserSaveFileHandle;
    const writable = await handle.createWritable();
    let closed = false;

    const closeWritable = async (mode: 'close' | 'abort') => {
      if (closed) {
        return;
      }
      closed = true;
      if (mode === 'close') {
        await writable.close();
        return;
      }
      await writable.abort();
    };

    return {
      begin: async () => BROWSER_DOWNLOAD_SESSION_ID,
      writeChunk: async (_sessionId, chunk) => {
        await writable.write(new Uint8Array(chunk).buffer);
      },
      finish: async () => {
        await closeWritable('close');
        return true;
      },
      abort: async () => {
        await closeWritable('abort');
      },
    };
  } catch {
    return undefined;
  }
}

declare global {
  interface Window {
    showSaveFilePicker?: (options?: BrowserSaveFilePickerOptions) => Promise<BrowserSaveFileHandle>;
  }

  // eslint-disable-next-line no-var
  var showSaveFilePicker: Window['showSaveFilePicker'];
}

export function isDriveAbortError(error: unknown): boolean {
  if (error instanceof DOMException && error.name === 'AbortError') {
    return true;
  }
  return error instanceof Error && (error.name === 'AbortError' || /\babort(?:ed)?\b/i.test(error.message));
}

function buildDownloadRequestInit(
  job: DownloadJob,
  grant: DownloadGrantLike,
  signal?: AbortSignal,
): RequestInit {
  const init: RequestInit = {
    method: grant.method || job.downloadMethod || 'GET',
    signal,
  };
  const resumeFromBytes = Math.max(0, Math.floor(job.downloadedSize || 0));
  if (resumeFromBytes > 0) {
    init.headers = {
      Range: `bytes=${resumeFromBytes}-`,
    };
  }
  return init;
}

function resolveExpectedDownloadTotal(
  response: Response,
  grant: DownloadGrantLike,
  job: DownloadJob,
  resumeFromBytes: number,
): number {
  const headerLength = Number(response.headers.get('content-length'));
  const partialLength =
    Number.isFinite(headerLength) && headerLength > 0 ? headerLength : 0;
  const grantTotal = grant.archiveSizeBytes || grant.totalBytes || job.totalSize || 0;

  if (response.status === 206 && partialLength > 0) {
    return resumeFromBytes + partialLength;
  }
  if (partialLength > 0) {
    return partialLength;
  }
  return grantTotal;
}

async function streamResponseBodyToSession(
  response: Response,
  sessionId: string,
  streamAdapter: SaveDownloadStreamAdapter,
  expectedTotal: number,
  resumeFromBytes: number,
  onProgress?: (downloadedBytes: number, totalBytes: number) => void,
): Promise<number> {
  const reader = response.body?.getReader();
  if (!reader) {
    const blob = await response.blob();
    assertInMemoryDownloadWithinLimit(blob.size);
    await streamAdapter.writeChunk(sessionId, new Uint8Array(await blob.arrayBuffer()));
    onProgress?.(
      resumeFromBytes + blob.size,
      expectedTotal || resumeFromBytes + blob.size,
    );
    return resumeFromBytes + blob.size;
  }

  let receivedBytes = 0;
  while (true) {
    const { done, value } = await reader.read();
    if (done) {
      break;
    }
    await streamAdapter.writeChunk(sessionId, value);
    receivedBytes += value.byteLength;
    onProgress?.(
      resumeFromBytes + receivedBytes,
      expectedTotal || resumeFromBytes + receivedBytes,
    );
  }
  return resumeFromBytes + receivedBytes;
}

export async function executeDownloadTransfer(
  job: DownloadJob,
  grant: DownloadGrantLike,
  options: ExecuteDownloadTransferOptions = {},
): Promise<ExecuteDownloadTransferResult> {
  const url = resolveTransferOpenUrl(grant);
  if (!url) {
    throw new Error('Download grant did not include a URL.');
  }

  const fetchImpl = options.fetchImpl ?? fetch;
  const resumeFromBytes = Math.max(0, Math.floor(job.downloadedSize || 0));
  const response = await fetchImpl(url, buildDownloadRequestInit(job, grant, options.signal));
  if (!response.ok) {
    throw new Error(`Download failed with status ${response.status}.`);
  }
  if (resumeFromBytes > 0 && response.status === 200) {
    throw new Error('Download resume is not supported by the storage source.');
  }

  const expectedTotal = resolveExpectedDownloadTotal(response, grant, job, resumeFromBytes);

  const reader = response.body?.getReader();
  if (!reader) {
    const blob = await response.blob();
    assertInMemoryDownloadWithinLimit(resumeFromBytes + blob.size);
    options.onProgress?.(
      resumeFromBytes + blob.size,
      expectedTotal || resumeFromBytes + blob.size,
    );
    return {
      blob,
      fileName: job.fileName,
    };
  }

  const chunks: BlobPart[] = [];
  let receivedBytes = 0;
  while (true) {
    const { done, value } = await reader.read();
    if (done) {
      break;
    }
    receivedBytes += value.byteLength;
    assertInMemoryDownloadWithinLimit(resumeFromBytes + receivedBytes);
    chunks.push(value);
    options.onProgress?.(
      resumeFromBytes + receivedBytes,
      expectedTotal || resumeFromBytes + receivedBytes,
    );
  }

  const blob = new Blob(chunks, {
    type: response.headers.get('content-type') || job.mimeType || 'application/octet-stream',
  });
  options.onProgress?.(
    resumeFromBytes + blob.size,
    expectedTotal || resumeFromBytes + blob.size,
  );
  return {
    blob,
    fileName: job.fileName,
  };
}

export function triggerBrowserDownload(blob: Blob, fileName: string): void {
  const objectUrl = URL.createObjectURL(blob);
  const anchor = document.createElement('a');
  anchor.href = objectUrl;
  anchor.download = fileName;
  anchor.rel = 'noopener';
  document.body.appendChild(anchor);
  anchor.click();
  anchor.remove();
  globalThis.setTimeout(() => URL.revokeObjectURL(objectUrl), 0);
}

export interface RunManagedDownloadTransferParams {
  job: DownloadJob;
  grant: DownloadGrantLike;
  signal?: AbortSignal;
  fetchImpl?: (input: string, init?: RequestInit) => Promise<Response>;
  resumeExistingProgress?: boolean;
  onJobUpdate: (updater: (current: DownloadJob) => DownloadJob) => void;
  onOpenExternal?: (url: string) => void | Promise<void>;
  saveDownload?: (fileName: string, blob: Blob) => Promise<boolean>;
  saveDownloadStream?: SaveDownloadStreamAdapter;
}

async function executeDownloadToStream(
  job: DownloadJob,
  grant: DownloadGrantLike,
  streamAdapter: SaveDownloadStreamAdapter,
  options: Pick<ExecuteDownloadTransferOptions, 'signal' | 'fetchImpl' | 'onProgress'>,
): Promise<number> {
  const url = resolveTransferOpenUrl(grant);
  if (!url) {
    throw new Error('Download grant did not include a URL.');
  }

  const sessionId = await streamAdapter.begin(job.fileName);
  if (!sessionId) {
    return -1;
  }

  try {
    const fetchImpl = options.fetchImpl ?? fetch;
    const resumeFromBytes = Math.max(0, Math.floor(job.downloadedSize || 0));
    const response = await fetchImpl(url, buildDownloadRequestInit(job, grant, options.signal));
    if (!response.ok) {
      throw new Error(`Download failed with status ${response.status}.`);
    }
    if (resumeFromBytes > 0 && response.status === 200) {
      throw new Error('Download resume is not supported by the storage source.');
    }

    const expectedTotal = resolveExpectedDownloadTotal(response, grant, job, resumeFromBytes);

    const receivedBytes = await streamResponseBodyToSession(
      response,
      sessionId,
      streamAdapter,
      expectedTotal,
      resumeFromBytes,
      options.onProgress,
    );
    const saved = await streamAdapter.finish(sessionId);
    if (!saved) {
      throw new Error('Download save was not completed.');
    }
    return receivedBytes;
  } catch (error) {
    await streamAdapter.abort(sessionId).catch(() => undefined);
    throw error;
  }
}

export async function runManagedDownloadTransfer(
  params: RunManagedDownloadTransferParams,
): Promise<void> {
  const {
    job,
    grant,
    signal,
    fetchImpl,
    resumeExistingProgress = false,
    onJobUpdate,
    onOpenExternal,
    saveDownload,
    saveDownloadStream,
  } = params;
  const openUrl = resolveTransferOpenUrl(grant);

  if (!resumeExistingProgress) {
    onJobUpdate(() => applyDownloadGrantToJob(job, grant));
  } else {
    onJobUpdate((current) => ({
      ...current,
      status: 'downloading',
      speed: 'Downloading...',
      timeRemaining: 'Calculating...',
      errorMessage: undefined,
    }));
  }

  if (!openUrl) {
    onJobUpdate((current) =>
      applyTransferFailure(current, 'Download grant did not include a URL.'),
    );
    return;
  }

  try {
    const activeJob = resumeExistingProgress ? job : applyDownloadGrantToJob(job, grant);
    const streamAdapter =
      saveDownloadStream ?? (await createBrowserDownloadStreamAdapter(activeJob.fileName));

    if (streamAdapter) {
      const receivedBytes = await executeDownloadToStream(activeJob, grant, streamAdapter, {
        signal,
        fetchImpl,
        onProgress: (downloadedBytes, totalBytes) => {
          onJobUpdate((current) => applyDownloadProgressToJob(current, downloadedBytes, totalBytes));
        },
      });
      if (receivedBytes < 0) {
        onJobUpdate((current) => ({
          ...applyDownloadProgressToJob(current, 0, current.totalSize || 0),
          status: 'ready',
          speed: 'Ready',
          timeRemaining: 'Save cancelled',
        }));
        return;
      }
      onJobUpdate((current) => applyDownloadCompletionToJob(current, receivedBytes));
      return;
    }

    const { blob, fileName } = await executeDownloadTransfer(activeJob, grant, {
      signal,
      fetchImpl,
      onProgress: (downloadedBytes, totalBytes) => {
        onJobUpdate((current) => applyDownloadProgressToJob(current, downloadedBytes, totalBytes));
      },
    });
    if (saveDownload) {
      const saved = await saveDownload(fileName, blob);
      if (!saved) {
        onJobUpdate((current) => ({
          ...applyDownloadProgressToJob(current, blob.size, blob.size),
          status: 'ready',
          speed: 'Ready',
          timeRemaining: 'Save cancelled',
        }));
        return;
      }
    } else {
      triggerBrowserDownload(blob, fileName);
    }
    onJobUpdate((current) => applyDownloadCompletionToJob(current, blob.size));
  } catch (error) {
    if (isDriveAbortError(error)) {
      return;
    }

    if (onOpenExternal) {
      try {
        await onOpenExternal(openUrl);
        onJobUpdate((current) =>
          applyTransferFailure(
            current,
            'Download failed; opened the source URL in your browser instead.',
          ),
        );
        return;
      } catch {
        // Fall through to failure state when external open also fails.
      }
    }

    onJobUpdate((current) =>
      applyTransferFailure(
        current,
        error instanceof Error ? error.message : 'Download transfer failed',
      ),
    );
  }
}
