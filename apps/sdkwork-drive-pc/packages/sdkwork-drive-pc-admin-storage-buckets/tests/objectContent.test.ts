/* @vitest-environment jsdom */

import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import type {
  StorageProviderAdminService,
  StorageProviderObjectView,
} from 'sdkwork-drive-pc-admin-storage-providers';
import {
  MAX_OBJECT_CONTENT_BYTES,
  downloadObjectBytes,
  readObjectBytes,
  readObjectText,
  writeObjectBytes,
  writeObjectText,
} from '../src/utils/objectContent';

/**
 * 内容通道的传输契约。
 *
 * 这一层是「桶名必须随每次请求一起走」的最后一道关口：预览与编辑器都只经它读写，
 * 一旦丢了 bucket，请求就会落到服务商配置的默认桶——那是最难发现的一类错误数据。
 */
const BUCKET = 'team-assets';
const PROVIDER_ID = 'provider-1';

function createService(overrides: Partial<StorageProviderAdminService> = {}) {
  const readObjectContent = vi.fn(async () => ({
    content: Buffer.from('hello world', 'utf8').toString('base64'),
    contentType: 'text/plain',
  }));
  const writeObjectContent = vi.fn(async () => undefined);
  const service = {
    readObjectContent,
    writeObjectContent,
    ...overrides,
  } as unknown as StorageProviderAdminService;
  return { readObjectContent, service, writeObjectContent };
}

const SCOPE = { bucket: BUCKET, providerId: PROVIDER_ID, service: createService().service };

beforeAll(() => {
  if (typeof URL.createObjectURL !== 'function') {
    Object.defineProperty(URL, 'createObjectURL', { configurable: true, value: () => 'blob:test' });
    Object.defineProperty(URL, 'revokeObjectURL', { configurable: true, value: () => undefined });
  }
});

afterEach(() => vi.restoreAllMocks());

describe('objectContent', () => {
  it('reads bytes with the bucket and forwards the abort signal', async () => {
    const { readObjectContent, service } = createService();
    const controller = new AbortController();

    const bytes = await readObjectBytes(
      { bucket: BUCKET, providerId: PROVIDER_ID, service },
      'docs/readme.txt',
      controller.signal,
    );

    expect(new TextDecoder().decode(bytes)).toBe('hello world');
    expect(readObjectContent).toHaveBeenCalledWith(PROVIDER_ID, 'docs/readme.txt', {
      bucket: BUCKET,
      signal: controller.signal,
    });
  });

  it('decodes text reads as UTF-8', async () => {
    const { service } = createService({
      readObjectContent: vi.fn(async () => ({
        content: Buffer.from('中文内容', 'utf8').toString('base64'),
      })) as unknown as StorageProviderAdminService['readObjectContent'],
    });

    await expect(readObjectText({ ...SCOPE, service }, 'notes.txt')).resolves.toBe('中文内容');
  });

  it('saves text back to the same object key and bucket, as utf8', async () => {
    const { service, writeObjectContent } = createService();

    await writeObjectText({ bucket: BUCKET, providerId: PROVIDER_ID, service }, 'docs/a.md', '# hi');

    expect(writeObjectContent).toHaveBeenCalledWith(
      PROVIDER_ID,
      'docs/a.md',
      { content: '# hi', encoding: 'utf8' },
      { bucket: BUCKET },
    );
  });

  it('keeps the declared content type on a base64 write', async () => {
    const { service, writeObjectContent } = createService();

    await writeObjectBytes(
      { bucket: BUCKET, providerId: PROVIDER_ID, service },
      'img/logo.png',
      new Uint8Array([1, 2, 3]),
      'image/png',
    );

    expect(writeObjectContent).toHaveBeenCalledWith(
      PROVIDER_ID,
      'img/logo.png',
      { content: 'AQID', encoding: 'base64', contentType: 'image/png' },
      { bucket: BUCKET },
    );
  });

  it('downloads the clicked object through the bucket-scoped content channel', async () => {
    const { readObjectContent, service } = createService();
    const click = vi
      .spyOn(HTMLAnchorElement.prototype, 'click')
      .mockImplementation(() => undefined);
    const object = {
      key: 'reports/2026-q3.pdf',
      sizeBytes: 3,
      contentType: 'application/pdf',
    } as StorageProviderObjectView;

    await downloadObjectBytes({ bucket: BUCKET, providerId: PROVIDER_ID, service }, object);

    expect(readObjectContent).toHaveBeenCalledWith(PROVIDER_ID, 'reports/2026-q3.pdf', {
      bucket: BUCKET,
    });
    expect(click).toHaveBeenCalledTimes(1);
  });

  it('documents the identical inline limit the backend enforces', () => {
    // 8 MiB 是后端内容接口的上限；前端只用这一个常量做提示与拦截。
    expect(MAX_OBJECT_CONTENT_BYTES).toBe(8 * 1024 * 1024);
  });
});
