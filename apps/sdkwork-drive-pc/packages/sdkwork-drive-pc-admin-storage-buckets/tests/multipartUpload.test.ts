/* @vitest-environment jsdom */

import { afterEach, describe, expect, it, vi } from 'vitest';
import type {
  StorageProviderAdminService,
  StorageProviderUploadPartGrantView,
} from 'sdkwork-drive-pc-admin-storage-providers';
import {
  MULTIPART_MAX_PARTS,
  MULTIPART_PART_SIZE_BYTES,
  MultipartUploadError,
  planUploadParts,
  uploadObjectInParts,
} from '../src/utils/multipartUpload';

const PROVIDER_ID = 'provider-1';
const OBJECT_KEY = 'photos/big.bin';

/** 假 service：记录调用顺序与参数，便于断言"开启→签发→完成/中止"的链路。 */
function createService(options?: { abortFails?: boolean }) {
  const calls: string[] = [];
  const completedParts: { partNumber: number; etag: string }[][] = [];
  const presignBatches: number[][] = [];
  const service = {
    createMultipartUpload: vi.fn(async () => {
      calls.push('create');
      return { providerId: PROVIDER_ID, bucket: 'bucket-1', objectKey: OBJECT_KEY, uploadId: 'upload-1' };
    }),
    presignUploadParts: vi.fn(async (_providerId: string, input: { partNumbers: number[] }) => {
      calls.push('presign');
      presignBatches.push([...input.partNumbers]);
      return {
        providerId: PROVIDER_ID,
        bucket: 'bucket-1',
        objectKey: OBJECT_KEY,
        uploadId: 'upload-1',
        parts: input.partNumbers.map<StorageProviderUploadPartGrantView>((partNumber) => ({
          partNumber,
          method: 'PUT',
          url: `https://cos.example.com/bucket-1/${OBJECT_KEY}?partNumber=${partNumber}&uploadId=upload-1`,
          headers: { 'content-type': 'application/octet-stream', 'x-cos-signature': `sig-${partNumber}` },
          expiresAtEpochMs: 1_767_225_600_000,
        })),
      };
    }),
    completeMultipartUpload: vi.fn(async (_providerId: string, input: any) => {
      calls.push('complete');
      completedParts.push(input.parts);
      return {
        key: OBJECT_KEY,
        sizeBytes: 20 * 1024 * 1024,
        isFolder: false,
      };
    }),
    abortMultipartUpload: vi.fn(async () => {
      calls.push('abort');
      if (options?.abortFails) {
        throw new Error('abort failed');
      }
      return true;
    }),
  } as unknown as StorageProviderAdminService;
  return { calls, completedParts, presignBatches, service };
}

/** 假 fetch：按分片号返回带 ETag 的响应，可注入失败序列。 */
function createFetch(options?: {
  failTimes?: Map<number, number>;
  status?: number;
  omitEtag?: boolean;
  rejectWithTypeError?: boolean;
}) {
  const seen: { partNumber: number; headers: Record<string, string> }[] = [];
  const attempts = new Map<number, number>();
  const fetchImpl = vi.fn(async (url: string, init?: RequestInit) => {
    const partNumber = Number(new URL(url).searchParams.get('partNumber'));
    const count = (attempts.get(partNumber) ?? 0) + 1;
    attempts.set(partNumber, count);
    seen.push({ partNumber, headers: (init?.headers ?? {}) as Record<string, string> });

    if (options?.rejectWithTypeError) {
      throw new TypeError('Failed to fetch');
    }
    const remaining = options?.failTimes?.get(partNumber) ?? 0;
    if (remaining > 0) {
      options?.failTimes?.set(partNumber, remaining - 1);
      return new Response('', { status: options?.status ?? 500 });
    }
    if (options?.status && options.status >= 400) {
      return new Response('', { status: options.status });
    }
    return new Response('', {
      status: 200,
      headers: options?.omitEtag ? {} : { ETag: `"etag-part-${partNumber}"` },
    });
  });
  return { fetchImpl: fetchImpl as unknown as typeof fetch, seen, attempts };
}

afterEach(() => {
  vi.restoreAllMocks();
});

describe('planUploadParts', () => {
  it('splits a file into 8 MiB parts with a short last part', () => {
    const size = MULTIPART_PART_SIZE_BYTES * 2 + 1024;
    const parts = planUploadParts(size);
    expect(parts.map((part) => part.partNumber)).toEqual([1, 2, 3]);
    expect(parts[0]).toEqual({ partNumber: 1, start: 0, end: MULTIPART_PART_SIZE_BYTES });
    // 最后一片是余数，不是整片。
    expect(parts[2].end).toBe(size);
    expect(parts[2].end - parts[2].start).toBe(1024);
    // 区间必须首尾相接且完整覆盖文件。
    expect(parts[1].start).toBe(parts[0].end);
    expect(parts.reduce((total, part) => total + (part.end - part.start), 0)).toBe(size);
  });

  it('returns no parts for an empty object and refuses a file above the vendor part limit', () => {
    // 空对象走单次写入（那条通道也负责创建目录占位）。
    expect(planUploadParts(0)).toEqual([]);
    // 超过 10000 片：这条通道装不下，必须在开启上传之前就拒绝。
    expect(() => planUploadParts(MULTIPART_MAX_PARTS * MULTIPART_PART_SIZE_BYTES + 1)).toThrow(
      MultipartUploadError,
    );
    try {
      planUploadParts(MULTIPART_MAX_PARTS * MULTIPART_PART_SIZE_BYTES + 1);
    } catch (error) {
      expect((error as MultipartUploadError).kind).toBe('too-many-parts');
    }
  });
});

describe('uploadObjectInParts', () => {
  it('opens, presigns in batches, uploads every part and completes in part order', async () => {
    const { calls, completedParts, presignBatches, service } = createService();
    const { fetchImpl, seen } = createFetch();
    const progress: number[] = [];
    const file = new Blob([new Uint8Array(20 * 1024 * 1024)]);

    await uploadObjectInParts({
      file,
      objectKey: OBJECT_KEY,
      providerId: PROVIDER_ID,
      service,
      fetchImpl,
      concurrency: 3,
      onProgress: (value) => progress.push(value.uploadedParts),
    });

    // 20 MiB / 8 MiB = 3 片，正好一批（并发 3），所以只签发一次。
    expect(calls).toEqual(['create', 'presign', 'complete']);
    expect(presignBatches).toEqual([[1, 2, 3]]);
    expect(seen.map((entry) => entry.partNumber).sort()).toEqual([1, 2, 3]);
    // 签名头必须原样回放，否则厂商 403。
    expect(seen[0].headers).toMatchObject({
      'content-type': 'application/octet-stream',
      'x-cos-signature': expect.stringContaining('sig-'),
    });
    // 完成时提交的是厂商返回的 ETag，且按分片号排序。
    expect(completedParts[0]).toEqual([
      { partNumber: 1, etag: '"etag-part-1"' },
      { partNumber: 2, etag: '"etag-part-2"' },
      { partNumber: 3, etag: '"etag-part-3"' },
    ]);
    // 进度是单调递增到 totalParts。
    expect(progress).toEqual([1, 2, 3]);
    expect(service.abortMultipartUpload).not.toHaveBeenCalled();
  });

  it('retries a transient part failure and still completes', async () => {
    const { calls, service } = createService();
    const { fetchImpl, attempts } = createFetch({ failTimes: new Map([[2, 1]]) });

    await uploadObjectInParts({
      file: new Blob([new Uint8Array(20 * 1024 * 1024)]),
      objectKey: OBJECT_KEY,
      providerId: PROVIDER_ID,
      service,
      fetchImpl,
      concurrency: 3,
    });

    expect(attempts.get(2)).toBe(2);
    expect(calls).toContain('complete');
    expect(service.abortMultipartUpload).not.toHaveBeenCalled();
  });

  it('classifies a blocked cross-origin request as a CORS failure and aborts the upload', async () => {
    const { service } = createService();
    const { fetchImpl } = createFetch({ rejectWithTypeError: true });

    await expect(
      uploadObjectInParts({
        file: new Blob([new Uint8Array(9 * 1024 * 1024)]),
        objectKey: OBJECT_KEY,
        providerId: PROVIDER_ID,
        service,
        fetchImpl,
      }),
    ).rejects.toMatchObject({ kind: 'cors' });
    // 失败的开启必须被中止，否则分片一直计费。
    expect(service.abortMultipartUpload).toHaveBeenCalledTimes(1);
  });

  it('classifies a part uploaded without an exposed ETag as etag-missing', async () => {
    const { service } = createService();
    const { fetchImpl } = createFetch({ omitEtag: true });

    await expect(
      uploadObjectInParts({
        file: new Blob([new Uint8Array(9 * 1024 * 1024)]),
        objectKey: OBJECT_KEY,
        providerId: PROVIDER_ID,
        service,
        fetchImpl,
      }),
    ).rejects.toMatchObject({ kind: 'etag-missing' });
    expect(service.abortMultipartUpload).toHaveBeenCalledTimes(1);
  });

  it('does not retry a 403 rejection from the vendor', async () => {
    const { service } = createService();
    const { fetchImpl, attempts } = createFetch({ status: 403 });

    await expect(
      uploadObjectInParts({
        file: new Blob([new Uint8Array(9 * 1024 * 1024)]),
        objectKey: OBJECT_KEY,
        providerId: PROVIDER_ID,
        service,
        fetchImpl,
      }),
    ).rejects.toMatchObject({ kind: 'part-failed' });
    // 4xx 重试没有意义，只发一次。
    expect(attempts.get(1)).toBe(1);
    expect(service.abortMultipartUpload).toHaveBeenCalledTimes(1);
  });

  it('aborts on cancellation and reports it as aborted', async () => {
    const { service } = createService();
    const controller = new AbortController();
    const { fetchImpl } = createFetch();
    const file = new Blob([new Uint8Array(20 * 1024 * 1024)]);

    const promise = uploadObjectInParts({
      file,
      objectKey: OBJECT_KEY,
      providerId: PROVIDER_ID,
      service,
      fetchImpl,
      signal: controller.signal,
      concurrency: 1,
      onProgress: (value) => {
        // 第一片传完就取消：模拟用户关掉弹窗。
        if (value.uploadedParts === 1) {
          controller.abort();
        }
      },
    });

    await expect(promise).rejects.toMatchObject({ kind: 'aborted' });
    expect(service.abortMultipartUpload).toHaveBeenCalledTimes(1);
    expect(service.completeMultipartUpload).not.toHaveBeenCalled();
  });

  it('still surfaces the original failure when the cleanup abort also fails', async () => {
    const { service } = createService({ abortFails: true });
    const { fetchImpl } = createFetch({ omitEtag: true });

    await expect(
      uploadObjectInParts({
        file: new Blob([new Uint8Array(9 * 1024 * 1024)]),
        objectKey: OBJECT_KEY,
        providerId: PROVIDER_ID,
        service,
        fetchImpl,
      }),
    ).rejects.toMatchObject({ kind: 'etag-missing' });
  });
});
