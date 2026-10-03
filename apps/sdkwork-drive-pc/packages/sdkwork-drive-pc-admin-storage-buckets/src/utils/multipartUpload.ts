import type {
  StorageProviderAdminService,
  StorageProviderObjectView,
} from 'sdkwork-drive-pc-admin-storage-providers';
import { MAX_OBJECT_CONTENT_BYTES } from 'sdkwork-drive-pc-commons';

/**
 * 分片直传（预签名 multipart）。
 *
 * 为什么需要它：单次内容接口上限 8 MiB，而且内容以 base64 装在 JSON 里——请求体会比文件大
 * 约 37%。大于 8 MiB 的对象只能走这条路：管理端只负责"开启 / 签发 / 完成 / 中止"，字节由
 * 浏览器直接 PUT 到厂商的预签名地址，既不经过我们的网关，也不占用服务端内存。
 *
 * 这里刻意不依赖任何 UI 框架：宿主只提供 service、AbortSignal 与进度回调，测试可以直接
 * 用假 service + 假 fetch 覆盖全部分支。
 */

/**
 * 超过这个大小就走分片：与单次内容接口的业务上限一致。
 *
 * 用 **8 MiB** 而不是"内容接口能接受的编码后大小"：接口上限是对象字节（base64 之后约
 * 11.2 MB），把阈值设在对象字节上，两条通道的分界才有唯一解释。
 */
export const MULTIPART_THRESHOLD_BYTES = MAX_OBJECT_CONTENT_BYTES;

/**
 * 单片大小 8 MiB。
 *
 * 厂商要求除最后一片外每片 ≥5 MiB，同时限制最多 10000 片：8 MiB 在"请求数"与"单片体积"
 * 之间取平衡，单对象上限约 80 GiB，远超管理端场景。
 */
export const MULTIPART_PART_SIZE_BYTES = 8 * 1024 * 1024;

/** 厂商的分片数上限（S3/COS 一致）。 */
export const MULTIPART_MAX_PARTS = 10_000;

/** 一批最多签发多少个分片授权：与契约的 `partNumbers` maxItems 一致。 */
export const MULTIPART_PRESIGN_BATCH_SIZE = 100;

/** 并发直传的分片数：3 片在"链路利用率"与"浏览器连接/内存"之间取平衡。 */
export const MULTIPART_CONCURRENCY = 3;

/** 单片重试次数（首次之外）：网络抖动与厂商 5xx 都靠它兜住。 */
export const MULTIPART_PART_ATTEMPTS = 3;

export interface MultipartUploadPart {
  partNumber: number;
  /** 分片在文件里的字节区间（`end` 不含）。 */
  start: number;
  end: number;
}

/** 上传失败的可判定原因：界面按它给出可执行的提示，而不是把厂商原文抛给用户。 */
export type MultipartUploadFailureKind =
  /** 浏览器被跨域策略拦下：桶的 CORS 没放行控制台源，或没允许 PUT。 */
  | 'cors'
  /** 分片传上去了但读不到 ETag：桶的 CORS 少了 ExposeHeaders: ETag。 */
  | 'etag-missing'
  /** 单片多次重试后仍失败。 */
  | 'part-failed'
  /** 完成阶段失败（厂商重组失败或服务端校验拒绝）。 */
  | 'complete-failed'
  /** 分片数超过厂商上限：文件太大，这条通道装不下。 */
  | 'too-many-parts'
  /** 操作被取消（用户取消或关闭弹窗）。 */
  | 'aborted';

export class MultipartUploadError extends Error {
  readonly kind: MultipartUploadFailureKind;
  /** 出问题的分片号（分片相关失败才有）。 */
  readonly partNumber?: number;
  /** 服务端/厂商的原始信息，用于日志与详情，不直接当文案。 */
  readonly cause?: unknown;

  constructor(
    kind: MultipartUploadFailureKind,
    message: string,
    options?: { partNumber?: number; cause?: unknown },
  ) {
    super(message);
    this.name = 'MultipartUploadError';
    this.kind = kind;
    this.partNumber = options?.partNumber;
    this.cause = options?.cause;
  }
}

/**
 * 把文件切成上传分片。
 *
 * 纯函数：分片规划是最容易出错、也最容易测的部分（边界、最后一片、上限），所以从 IO 里拆出来。
 */
export function planUploadParts(
  sizeBytes: number,
  partSizeBytes: number = MULTIPART_PART_SIZE_BYTES,
): MultipartUploadPart[] {
  if (!Number.isFinite(sizeBytes) || sizeBytes < 0) {
    throw new MultipartUploadError('part-failed', 'file size is not a finite non-negative number');
  }
  if (!Number.isFinite(partSizeBytes) || partSizeBytes <= 0) {
    throw new MultipartUploadError('part-failed', 'part size must be a positive number');
  }
  if (sizeBytes === 0) {
    // 空对象没有分片可传：调用方应当走单次写入（它也支持创建 0 字节对象/目录占位）。
    return [];
  }
  const parts: MultipartUploadPart[] = [];
  for (let start = 0; start < sizeBytes; start += partSizeBytes) {
    if (parts.length >= MULTIPART_MAX_PARTS) {
      throw new MultipartUploadError(
        'too-many-parts',
        `object would need more than ${MULTIPART_MAX_PARTS} parts`,
      );
    }
    parts.push({
      partNumber: parts.length + 1,
      start,
      end: Math.min(start + partSizeBytes, sizeBytes),
    });
  }
  return parts;
}

export interface MultipartUploadProgress {
  /** 已传完的分片数。 */
  uploadedParts: number;
  totalParts: number;
  /** 已传完的字节数（按分片计，不区分片内进度）。 */
  uploadedBytes: number;
  totalBytes: number;
}

export interface UploadObjectInPartsOptions {
  file: Blob;
  objectKey: string;
  providerId: string;
  service: StorageProviderAdminService;
  bucket?: string;
  region?: string;
  contentType?: string;
  signal?: AbortSignal;
  onProgress?: (progress: MultipartUploadProgress) => void;
  /** 测试可注入；生产用全局 fetch。 */
  fetchImpl?: typeof fetch;
  concurrency?: number;
  partSizeBytes?: number;
  attempts?: number;
}

function partEtagFromResponse(response: Response): string | undefined {
  const etag = response.headers.get('ETag') ?? response.headers.get('etag');
  return etag ? etag.trim() : undefined;
}

/** 单片直传：把签名头原样带上，读回 ETag。 */
async function uploadPart(
  grant: {
    partNumber: number;
    method: string;
    url: string;
    headers: Record<string, string>;
  },
  body: Blob,
  options: { fetchImpl: typeof fetch; signal?: AbortSignal; attempts: number },
): Promise<string> {
  let lastError: unknown;
  for (let attempt = 1; attempt <= options.attempts; attempt += 1) {
    if (options.signal?.aborted) {
      throw new MultipartUploadError('aborted', 'upload was cancelled', {
        partNumber: grant.partNumber,
      });
    }
    try {
      const response = await options.fetchImpl(grant.url, {
        method: grant.method || 'PUT',
        headers: grant.headers,
        body,
        signal: options.signal,
      });
      if (!response.ok) {
        // 4xx 是"再试也一样"（签名过期、桶策略拒绝），5xx/429 才值得重试。
        if (response.status < 500 && response.status !== 429) {
          throw new MultipartUploadError(
            'part-failed',
            `part ${grant.partNumber} was rejected with status ${response.status}`,
            { partNumber: grant.partNumber },
          );
        }
        lastError = new MultipartUploadError(
          'part-failed',
          `part ${grant.partNumber} failed with status ${response.status}`,
          { partNumber: grant.partNumber },
        );
      } else {
        const etag = partEtagFromResponse(response);
        if (!etag) {
          /*
           * 上传成功但读不到 ETag：几乎总是桶的 CORS 少了 `ExposeHeaders: ETag`。
           * 这时候重试没有意义（每次都会成功但都没有 ETag），直接判定并把原因说清楚——
           * 否则用户会看到"传完了但合不起来"，然后去查网络、查权限，查不到点子上。
           */
          throw new MultipartUploadError(
            'etag-missing',
            `part ${grant.partNumber} uploaded but the response exposed no ETag`,
            { partNumber: grant.partNumber },
          );
        }
        return etag;
      }
    } catch (error) {
      if (error instanceof MultipartUploadError) {
        // 判定类错误（ETag 缺失、4xx 拒绝、取消）不重试。
        throw error;
      }
      if (options.signal?.aborted) {
        throw new MultipartUploadError('aborted', 'upload was cancelled', {
          partNumber: grant.partNumber,
        });
      }
      /*
       * fetch 本身 reject：浏览器在跨域被拒时报 TypeError('Failed to fetch')，且**不会**
       * 给出状态码——这正是"桶没配 CORS"的典型表现。把它单独归类，界面才能给出可执行的
       * 提示（去桶里加 CORS 规则），而不是让人以为网络断了。
       */
      lastError = new MultipartUploadError(
        'cors',
        `part ${grant.partNumber} could not reach the bucket endpoint`,
        { partNumber: grant.partNumber, cause: error },
      );
    }
    if (attempt < options.attempts) {
      // 指数退避，让厂商的瞬时抖动有机会恢复；等待期间也要能被取消。
      await new Promise((resolve) => setTimeout(resolve, 200 * 2 ** (attempt - 1)));
    }
  }
  throw lastError instanceof MultipartUploadError
    ? lastError
    : new MultipartUploadError('part-failed', 'part upload failed');
}

/**
 * 分片直传一个对象：开启 →（并发）签发 + 直传 → 完成。
 *
 * 失败或取消时一定会调用 abort：厂商的分片在完成前一直计费，留下的"孤儿分片"是隐性成本。
 */
export async function uploadObjectInParts(
  options: UploadObjectInPartsOptions,
): Promise<StorageProviderObjectView> {
  const {
    file,
    objectKey,
    providerId,
    service,
    bucket,
    region,
    contentType,
    signal,
    onProgress,
    fetchImpl = fetch,
    concurrency = MULTIPART_CONCURRENCY,
    partSizeBytes = MULTIPART_PART_SIZE_BYTES,
    attempts = MULTIPART_PART_ATTEMPTS,
  } = options;

  const parts = planUploadParts(file.size, partSizeBytes);
  if (parts.length === 0) {
    throw new MultipartUploadError(
      'part-failed',
      'empty objects must use the single-request content channel',
    );
  }

  const scope = { bucket, region };
  const created = await service.createMultipartUpload(
    providerId,
    { objectKey, ...(contentType ? { contentType } : {}) },
    scope,
  );

  let uploadedParts = 0;
  let uploadedBytes = 0;
  const completed: { partNumber: number; etag: string }[] = [];

  const report = () =>
    onProgress?.({
      uploadedParts,
      totalParts: parts.length,
      uploadedBytes,
      totalBytes: file.size,
    });

  try {
    for (let index = 0; index < parts.length; index += concurrency) {
      if (signal?.aborted) {
        throw new MultipartUploadError('aborted', 'upload was cancelled');
      }
      const batch = parts.slice(index, index + concurrency);
      // 一批最多 100 个：契约的 partNumbers maxItems；并发只有 3，所以只会取到 3。
      const grants = await service.presignUploadParts(
        providerId,
        {
          objectKey,
          uploadId: created.uploadId,
          partNumbers: batch
            .slice(0, MULTIPART_PRESIGN_BATCH_SIZE)
            .map((part) => part.partNumber),
        },
        scope,
      );
      const grantByPart = new Map(grants.parts.map((grant) => [grant.partNumber, grant]));

      const results = await Promise.all(
        batch.map(async (part) => {
          const grant = grantByPart.get(part.partNumber);
          if (!grant || !grant.url) {
            throw new MultipartUploadError(
              'part-failed',
              `no upload grant was issued for part ${part.partNumber}`,
              { partNumber: part.partNumber },
            );
          }
          const etag = await uploadPart(
            grant,
            file.slice(part.start, part.end),
            { fetchImpl, signal, attempts },
          );
          uploadedParts += 1;
          uploadedBytes += part.end - part.start;
          report();
          return { partNumber: part.partNumber, etag };
        }),
      );
      completed.push(...results);
    }

    return await service.completeMultipartUpload(
      providerId,
      {
        objectKey,
        uploadId: created.uploadId,
        // 厂商按分片号重组：提交前排序，顺序由服务端保证，客户端不该猜。
        parts: [...completed].sort((left, right) => left.partNumber - right.partNumber),
      },
      scope,
    );
  } catch (error) {
    // 完成阶段失败也要清理：那次上传已经没有任何用处。
    try {
      await service.abortMultipartUpload(
        providerId,
        { objectKey, uploadId: created.uploadId },
        scope,
      );
    } catch {
      // 中止失败不能盖住原始错误：真正的失败原因更有价值，孤儿分片由运维清理。
    }
    if (error instanceof MultipartUploadError && error.kind === 'complete-failed') {
      throw error;
    }
    if (error instanceof MultipartUploadError) {
      throw error;
    }
    throw new MultipartUploadError('complete-failed', 'multipart upload failed', { cause: error });
  }
}
