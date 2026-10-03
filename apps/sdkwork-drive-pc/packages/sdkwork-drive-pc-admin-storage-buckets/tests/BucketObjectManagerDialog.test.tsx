/* @vitest-environment jsdom */

import React from 'react';
import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { DriveAdminStorageSdkClient } from 'sdkwork-drive-pc-admin-core';
import { createStorageProviderAdminService } from 'sdkwork-drive-pc-admin-storage-providers';
import { BucketObjectManagerDialog } from '../src/components/BucketObjectManagerDialog';

afterEach(() => cleanup());

function createFakeClient(request: unknown) {
  return {
    metadata: {},
    operations: {},
    setTokenManager: () => undefined,
    request,
  } as unknown as DriveAdminStorageSdkClient;
}

const getSession = () => ({
  context: { tenantId: 'tenant-100', userId: 'user-100', actorId: 'operator-100' },
});

const PROVIDER = {
  id: 'provider-cos-b',
  providerKind: 'tencent_cos',
  displayName: 'COS Guangzhou',
  endpointUrl: 'https://cos.ap-guangzhou.myqcloud.com',
  region: 'ap-guangzhou',
  bucket: 'drive-guangzhou',
  pathStyle: false,
  credentialConfigured: true,
  status: 'active',
  version: 1,
  strictTls: true,
};

/** 被管理的桶不是配置桶：每个请求都要带上它，这就是这套弹窗存在的理由。 */
const MANAGED_BUCKET = 'image2-1253947560';

/** 前缀感知的列表桩：进入目录后返回的是该目录下的对象，而不是上一层的内容。 */
function createRequest() {
  return vi.fn(async ({ operationId, query }: any) => {
    if (operationId === 'storageProviders.objects.list') {
      const prefix: string = query.prefix ?? '';
      return {
        items: prefix
          ? [
              // 真实后端会把目录占位对象（key 恰好等于前缀）一起返回：
              // 它就是"文件夹"本身，不该在目录里显示成一行空文件。
              { objectKey: prefix, objectKind: 'object', contentLength: 0 },
              {
                objectKey: `${prefix}cover.png`,
                objectKind: 'object',
                contentLength: 1024,
                lastModifiedEpochMs: Date.UTC(2026, 0, 2),
              },
            ]
          : [
              { objectKey: 'photos/', objectKind: 'prefix' },
              {
                objectKey: 'photos/cover.png',
                objectKind: 'object',
                contentLength: 1024,
                lastModifiedEpochMs: Date.UTC(2026, 0, 2),
              },
              {
                objectKey: 'notes.txt',
                objectKind: 'object',
                contentType: 'text/plain',
                contentLength: 11,
              },
            ],
        pageInfo: { hasMore: false },
      };
    }
    if (operationId === 'storageProviders.objects.content.retrieve') {
      return {
        providerId: PROVIDER.id,
        bucket: MANAGED_BUCKET,
        objectKey: 'notes.txt',
        contentType: 'text/plain',
        sizeBytes: 11,
        encoding: 'base64',
        content: window.btoa('hello world'),
        checksumSha256: 'checksum',
      };
    }
    if (operationId === 'storageProviders.objects.content.update') {
      return {
        providerId: PROVIDER.id,
        bucket: MANAGED_BUCKET,
        objectKey: 'invoices/',
        objectKind: 'object',
        contentLength: 0,
      };
    }
    if (operationId === 'storageProviders.objects.copy') {
      return {
        providerId: PROVIDER.id,
        bucket: MANAGED_BUCKET,
        objectKey: 'photos/renamed.png',
        changed: true,
      };
    }
    if (operationId === 'storageProviders.objects.delete') {
      return undefined;
    }
    throw new Error(`Unexpected operation ${operationId}`);
  });
}

function renderDialog(request: unknown) {
  const client = createFakeClient(request);
  const service = createStorageProviderAdminService({
    adminStorageSdkClient: client,
    getSession,
  });
  render(
    <BucketObjectManagerDialog
      bucket={MANAGED_BUCKET}
      onOpenChange={() => undefined}
      open
      provider={PROVIDER}
      service={service}
    />,
  );
}

describe('BucketObjectManagerDialog', () => {
  it('reads the managed bucket and lists its objects with a category rail', async () => {
    const request = createRequest();
    renderDialog(request);

    await waitFor(() => {
      expect(
        request.mock.calls.some(
          (call) =>
            call[0].operationId === 'storageProviders.objects.list'
            && call[0].query.bucket === MANAGED_BUCKET
            && call[0].query.delimiter === '/',
        ),
      ).toBe(true);
    });

    expect(await screen.findByText('photos/')).toBeTruthy();
    // 桶根层的对象展示的是相对当前前缀的路径。
    expect(await screen.findByText('photos/cover.png')).toBeTruthy();
    // 左栏分类：文件夹 1、图片 1，总数 2。
    expect(await screen.findByText('bucketsBrowserCategories')).toBeTruthy();
    expect(screen.getByRole('button', { name: /bucketsBrowserCategoryAll/ })).toBeTruthy();
    expect(screen.getByRole('button', { name: /bucketsBrowserCategoryImage/ })).toBeTruthy();
    expect(await screen.findByText('bucketsBrowserInfo')).toBeTruthy();
    // 桶名同时出现在头部身份条与左栏信息里，两处都应可读。
    expect(screen.getAllByText(MANAGED_BUCKET).length).toBeGreaterThan(0);
  });

  it('filters the visible objects by category without another request', async () => {
    const request = createRequest();
    renderDialog(request);

    expect(await screen.findByText('photos/cover.png')).toBeTruthy();
    const listCalls = request.mock.calls.filter(
      (call) => call[0].operationId === 'storageProviders.objects.list',
    ).length;

    fireEvent.click(screen.getByRole('button', { name: /bucketsBrowserCategoryImage/ }));

    // 分类只筛选已加载的对象，不会再打一次列表接口。
    expect(screen.queryByText('photos/')).toBeNull();
    expect(screen.getByText('photos/cover.png')).toBeTruthy();
    expect(
      request.mock.calls.filter((call) => call[0].operationId === 'storageProviders.objects.list')
        .length,
    ).toBe(listCalls);
  });

  it('navigates into a folder prefix inside the bucket', async () => {
    const request = createRequest();
    renderDialog(request);

    // 整行可点：文件夹行点击进入下一层，不再有单独的行内按钮。
    fireEvent.click(await screen.findByText('photos/'));

    await waitFor(() => {
      expect(
        request.mock.calls.some(
          (call) =>
            call[0].operationId === 'storageProviders.objects.list'
            && call[0].query.bucket === MANAGED_BUCKET
            && call[0].query.prefix === 'photos/',
        ),
      ).toBe(true);
    });

    // 目录占位对象（key === 前缀）不出现在它自己的目录里：否则会看到一行没有名字、
    // 0 B 的"文件"，而且它可被重命名/删除，等于能把目录本身弄丢。
    // 目录内的行只显示相对名字（不带前缀），所以这里期望的是 cover.png。
    expect(await screen.findByText('cover.png')).toBeTruthy();
    expect(document.querySelectorAll('tbody tr')).toHaveLength(1);
  });

  it('refuses a rename that would move the object into another prefix', async () => {
    const request = createRequest();
    renderDialog(request);

    expect(await screen.findByText('notes.txt')).toBeTruthy();
    // 按行取重命名按钮：排序改变行顺序时不会挑错对象。
    const notesRow = screen.getByText('notes.txt').closest('tr') as HTMLElement;
    fireEvent.click(within(notesRow).getByRole('button', { name: 'rename' }));
    const prompt = await screen.findByRole('dialog', { name: 'renameDialogTitle' });
    fireEvent.change(within(prompt).getByRole('textbox'), {
      target: { value: 'archive/notes.txt' },
    });
    fireEvent.click(within(prompt).getByRole('button', { name: 'confirm' }));

    // 名称里带分隔符会拼出另一个前缀：必须挡在请求之前。
    await waitFor(() => {
      expect(screen.getByText('bucketsBrowserInvalidName')).toBeTruthy();
    });
    expect(
      request.mock.calls.some(
        (call) => call[0].operationId === 'storageProviders.objects.copy',
      ),
    ).toBe(false);
  });

  it('renders a long object key as its own breaking block in the delete confirmation', async () => {
    /*
     * 对象名常常是一长串没有断点的字符。把「删除"{key}"？」整句交给浏览器时，为了塞下这个
     * 长词，浏览器会把中文断在任意两个字之间（曾经出现「删」独占一行）。名字必须是与句子
     * 分开的块级元素，并允许词内断行。
     */
    const longKey = 'c4af6ac5234440089ef10c63b015821.mp3';
    const request = vi.fn(async ({ operationId }: any) => {
      if (operationId === 'storageProviders.objects.list') {
        return {
          items: [{ objectKey: longKey, objectKind: 'object', contentLength: 11 }],
          pageInfo: { hasMore: false },
        };
      }
      if (operationId === 'storageProviders.objects.delete') {
        return undefined;
      }
      throw new Error(`Unexpected operation ${operationId}`);
    });
    renderDialog(request);

    // 直接从行内动作打开删除确认（点行本身是"预览"，会把列表挡在后面）。
    const row = (await screen.findByText(longKey)).closest('tr') as HTMLElement;
    fireEvent.click(within(row).getByRole('button', { name: 'del' }));

    const confirm = await screen.findByRole('dialog', { name: 'del' });
    const nameBlock = within(confirm).getByText(longKey);
    // 名字单独一块，且允许在词内断行。
    expect(nameBlock.className).toContain('block');
    expect(nameBlock.className).toContain('break-all');
    // 句子与名字是两个元素：句子不包含名字，名字也不包含句子。
    expect(within(confirm).getByText('deleteObjectConfirmLabel')).toBeTruthy();
  });

  it('explains a request-body 413 instead of echoing "Payload too large"', async () => {
    /*
     * 内容接口把对象字节以 base64 装进 JSON，请求体约为文件的 1.37 倍；网关的请求体上限
     * 比业务上限（8 MiB）更早生效，超限时只回一句 `Payload too large`（code 41301）。
     * 照抄这句话会让用户以为对象本身超了 8 MiB，必须换成说明体积换算与调参位置的文案。
     */
    const base = createRequest();
    const request = vi.fn(async (call: any) => {
      if (call.operationId === 'storageProviders.objects.content.update') {
        throw Object.assign(new Error('Payload too large'), {
          code: 41301,
          detail: 'Payload too large',
          status: 413,
        });
      }
      return base(call);
    });
    renderDialog(request);

    expect(await screen.findByText('notes.txt')).toBeTruthy();
    const input = document.querySelector<HTMLInputElement>('input[type="file"]');
    fireEvent.change(input as HTMLInputElement, {
      target: { files: [new File(['audio'], 'song.mp3', { type: 'audio/mpeg' })] },
    });

    await waitFor(() => {
      expect(request.mock.calls.some((call: any) => call[0].operationId === 'storageProviders.objects.content.update')).toBe(true);
    });
    // 文案点名"请求体"与 base64 放大，而不是把框架的原文摆给用户。
    expect(await screen.findByText(/bucketsBrowserUploadPayloadTooLarge/)).toBeTruthy();
    expect(screen.queryByText('Payload too large')).toBeNull();
  });
  it('asks before an upload that would overwrite an existing object', async () => {
    const request = createRequest();
    renderDialog(request);

    expect(await screen.findByText('notes.txt')).toBeTruthy();

    // 同名上传会整对象覆盖 notes.txt：必须先确认，确认之前一个写请求都不该发出去。
    const input = document.querySelector<HTMLInputElement>('input[type="file"]');
    const replacement = new File(['new content'], 'notes.txt', { type: 'text/plain' });
    fireEvent.change(input as HTMLInputElement, { target: { files: [replacement] } });

    const confirm = await screen.findByRole('dialog', {
      name: 'bucketsBrowserOverwriteTitle',
    });
    expect(
      request.mock.calls.some(
        (call) => call[0].operationId === 'storageProviders.objects.content.update',
      ),
    ).toBe(false);

    fireEvent.click(
      within(confirm).getByRole('button', { name: 'bucketsBrowserOverwriteConfirm' }),
    );

    await waitFor(() => {
      expect(
        request.mock.calls.some(
          (call) =>
            call[0].operationId === 'storageProviders.objects.content.update'
            && call[0].pathParams.objectKey === 'notes.txt'
            && call[0].query.bucket === MANAGED_BUCKET,
        ),
      ).toBe(true);
    });
  });

  it('creates a folder placeholder in the managed bucket', async () => {
    const request = createRequest();
    renderDialog(request);

    // 先等首次列表落地：加载中的工具栏按钮是禁用的，点它不会打开输入框。
    expect(await screen.findByText('photos/')).toBeTruthy();

    fireEvent.click(screen.getByRole('button', { name: 'newFolder' }));
    const prompt = await screen.findByRole('dialog', { name: 'newFolderDialogTitle' });
    fireEvent.change(within(prompt).getByRole('textbox'), { target: { value: 'invoices' } });
    fireEvent.click(within(prompt).getByRole('button', { name: 'confirm' }));

    await waitFor(() => {
      expect(
        request.mock.calls.some(
          (call) =>
            call[0].operationId === 'storageProviders.objects.content.update'
            && call[0].pathParams.objectKey === 'invoices/'
            && call[0].query.bucket === MANAGED_BUCKET
            && call[0].body.content === '',
        ),
      ).toBe(true);
    });
  });

  it('uploads a picked file into the managed bucket and refreshes the list', async () => {
    const request = createRequest();
    renderDialog(request);

    expect(await screen.findByText('photos/')).toBeTruthy();

    // 上传入口是真正的按钮（键盘可达）；file input 只是它触发的隐藏控件。
    const input = document.querySelector<HTMLInputElement>('input[type="file"]');
    expect(input).not.toBeNull();
    const file = new File(['hello upload'], 'notes2.txt', { type: 'text/plain' });
    fireEvent.change(input as HTMLInputElement, { target: { files: [file] } });

    await waitFor(() => {
      expect(
        request.mock.calls.some(
          (call) =>
            call[0].operationId === 'storageProviders.objects.content.update'
            && call[0].pathParams.objectKey === 'notes2.txt'
            && call[0].query.bucket === MANAGED_BUCKET
            // 内容以 base64 传输（内容接口只有文本与 base64 两种编码）。
            && call[0].body.encoding === 'base64',
        ),
      ).toBe(true);
    });
  });

  it('downloads a small object through the bucket-scoped content channel', async () => {
    const request = createRequest();
    // jsdom 没有 object URL 与真实的下载行为：只断言内容请求本身带上了桶名。
    Object.defineProperty(URL, 'createObjectURL', { configurable: true, value: () => 'blob:test' });
    Object.defineProperty(URL, 'revokeObjectURL', { configurable: true, value: () => undefined });
    renderDialog(request);

    expect(await screen.findByText('notes.txt')).toBeTruthy();
    const downloadButtons = await screen.findAllByRole('button', { name: 'download' });
    fireEvent.click(downloadButtons[0]);

    await waitFor(() => {
      expect(
        request.mock.calls.some(
          (call) =>
            call[0].operationId === 'storageProviders.objects.content.retrieve'
            && call[0].query.bucket === MANAGED_BUCKET,
        ),
      ).toBe(true);
    });
  });

  it('uploads several picked files sequentially into the managed bucket', async () => {
    const request = createRequest();
    renderDialog(request);

    expect(await screen.findByText('photos/')).toBeTruthy();

    const input = document.querySelector<HTMLInputElement>('input[type="file"]');
    expect(input?.multiple).toBe(true);
    const first = new File(['one'], 'first.txt', { type: 'text/plain' });
    const second = new File(['two'], 'second.txt', { type: 'text/plain' });
    fireEvent.change(input as HTMLInputElement, { target: { files: [first, second] } });

    await waitFor(() => {
      const written = request.mock.calls
        .filter((call) => call[0].operationId === 'storageProviders.objects.content.update')
        .map((call) => call[0].pathParams.objectKey);
      // 顺序写：两个对象都要落到当前目录，且都带桶名。
      expect(written).toEqual(['first.txt', 'second.txt']);
    });
    expect(
      request.mock.calls
        .filter((call) => call[0].operationId === 'storageProviders.objects.content.update')
        .every((call) => call[0].query.bucket === MANAGED_BUCKET),
    ).toBe(true);
  });

  it('sends a file above the single-request limit through the multipart channel', async () => {
    /*
     * 8 MiB 是单次内容接口的上限（内容还要 base64 装在 JSON 里，请求体再放大 1.37 倍）。
     * 更大的文件必须走分片直传：开启 → 签发 → 浏览器直传厂商 → 完成。
     * 断言的是"分流决策 + 直传链路"，而不是"报错说文件太大"。
     */
    const base = createRequest();
    const request = vi.fn(async (call: any) => {
      if (call.operationId === 'storageProviders.objects.multipartUpload.create') {
        return {
          providerId: PROVIDER.id,
          bucket: MANAGED_BUCKET,
          objectKey: 'huge.bin',
          uploadId: 'upload-1',
        };
      }
      if (call.operationId === 'storageProviders.objects.multipartUpload.parts.presign') {
        return {
          providerId: PROVIDER.id,
          bucket: MANAGED_BUCKET,
          objectKey: 'huge.bin',
          uploadId: 'upload-1',
          parts: (call.body.partNumbers as number[]).map((partNumber) => ({
            partNumber,
            method: 'PUT',
            url: `https://cos.example.com/${MANAGED_BUCKET}/huge.bin?partNumber=${partNumber}`,
            headers: { 'x-cos-signature': `sig-${partNumber}` },
            expiresAtEpochMs: '1767225600000',
          })),
        };
      }
      if (call.operationId === 'storageProviders.objects.multipartUpload.complete') {
        return { providerId: PROVIDER.id, bucket: MANAGED_BUCKET, objectKey: 'huge.bin', objectKind: 'object', contentLength: 64 * 1024 * 1024 };
      }
      if (call.operationId === 'storageProviders.objects.multipartUpload.abort') {
        return { providerId: PROVIDER.id, bucket: MANAGED_BUCKET, objectKey: 'huge.bin', changed: true };
      }
      return base(call);
    });
    // 分片直传打到厂商地址：用假 fetch 顶住，返回带 ETag 的响应。
    const uploaded: string[] = [];
    const fetchSpy = vi.spyOn(globalThis, 'fetch').mockImplementation(async (url: any) => {
      uploaded.push(String(url));
      return new Response('', { status: 200, headers: { ETag: '"etag-1"' } });
    });

    renderDialog(request);
    expect(await screen.findByText('photos/')).toBeTruthy();

    const input = document.querySelector<HTMLInputElement>('input[type="file"]');
    const huge = new File(['x'], 'huge.bin', { type: 'application/octet-stream' });
    // 只改体积声明：64 MiB 必须走分片，而不是单次写入。
    Object.defineProperty(huge, 'size', { value: 64 * 1024 * 1024 });
    fireEvent.change(input as HTMLInputElement, { target: { files: [huge] } });

    await waitFor(() => {
      expect(
        request.mock.calls.some(
          (call) => call[0].operationId === 'storageProviders.objects.multipartUpload.complete',
        ),
      ).toBe(true);
    });
    // 分片确实打到了厂商地址（字节不经过管理端 API）……
    expect(uploaded.length).toBeGreaterThan(0);
    expect(uploaded[0]).toContain('cos.example.com');
    // ……而单次写入通道一次都没有被使用。
    expect(
      request.mock.calls.some(
        (call) => call[0].operationId === 'storageProviders.objects.content.update',
      ),
    ).toBe(false);
    // 顺利完成不应中止（中止只用于失败/取消）。
    expect(
      request.mock.calls.some(
        (call) => call[0].operationId === 'storageProviders.objects.multipartUpload.abort',
      ),
    ).toBe(false);

    fetchSpy.mockRestore();
  });

  it('tells the operator to configure bucket CORS when the browser blocks a part', async () => {
    /*
     * 分片直传打到厂商域名，属于跨域请求：桶没放行控制台来源时，浏览器直接 reject
     * TypeError('Failed to fetch')——没有状态码、没有厂商报文。用户只会看到"上传失败"，
     * 所以界面必须把他指向"去桶里加 CORS 规则"这一步。
     */
    const base = createRequest();
    const request = vi.fn(async (call: any) => {
      if (call.operationId === 'storageProviders.objects.multipartUpload.create') {
        return {
          providerId: PROVIDER.id,
          bucket: MANAGED_BUCKET,
          objectKey: 'huge.bin',
          uploadId: 'upload-1',
        };
      }
      if (call.operationId === 'storageProviders.objects.multipartUpload.parts.presign') {
        return {
          providerId: PROVIDER.id,
          bucket: MANAGED_BUCKET,
          objectKey: 'huge.bin',
          uploadId: 'upload-1',
          parts: (call.body.partNumbers as number[]).map((partNumber) => ({
            partNumber,
            method: 'PUT',
            url: `https://cos.example.com/${MANAGED_BUCKET}/huge.bin?partNumber=${partNumber}`,
            headers: {},
            expiresAtEpochMs: '1767225600000',
          })),
        };
      }
      if (call.operationId === 'storageProviders.objects.multipartUpload.abort') {
        return {
          providerId: PROVIDER.id,
          bucket: MANAGED_BUCKET,
          objectKey: 'huge.bin',
          changed: true,
        };
      }
      return base(call);
    });
    const fetchSpy = vi.spyOn(globalThis, 'fetch').mockImplementation(async () => {
      throw new TypeError('Failed to fetch');
    });

    renderDialog(request);
    expect(await screen.findByText('photos/')).toBeTruthy();
    const input = document.querySelector<HTMLInputElement>('input[type="file"]');
    const huge = new File(['x'], 'huge.bin', { type: 'application/octet-stream' });
    Object.defineProperty(huge, 'size', { value: 9 * 1024 * 1024 });
    fireEvent.change(input as HTMLInputElement, { target: { files: [huge] } });

    // 无 i18n provider 时 t 返回词条名本身：断言落到 CORS 那条文案上。
    expect(await screen.findByText(/bucketsBrowserUploadCorsBlocked/)).toBeTruthy();
    // 失败的开启必须被中止，否则分片会一直计费。
    await waitFor(() => {
      expect(
        request.mock.calls.some(
          (call) => call[0].operationId === 'storageProviders.objects.multipartUpload.abort',
        ),
      ).toBe(true);
    });

    fetchSpy.mockRestore();
  });

  it('selects rows with checkboxes and deletes the selection after one confirmation', async () => {
    const request = createRequest();
    renderDialog(request);

    expect(await screen.findByText('notes.txt')).toBeTruthy();

    // 行内复选框：按行定位，避免依赖标签插值（无 i18n provider 时参数不会替换）。
    // 排序后的行顺序：目录在前，其余按名称 —— photos/、notes.txt、photos/cover.png。
    const rows = () => screen.getAllByRole('row');
    fireEvent.click(within(rows()[1]).getByRole('checkbox'));
    expect(await screen.findByText('bucketsBrowserSelected')).toBeTruthy();
    fireEvent.click(within(rows()[2]).getByRole('checkbox'));

    expect(screen.getByText('bucketsBrowserDeleteSelected')).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: 'bucketsBrowserDeleteSelected' }));

    const confirm = await screen.findByRole('dialog', {
      name: 'bucketsBrowserDeleteSelected',
    });
    expect(
      request.mock.calls.some((call) => call[0].operationId === 'storageProviders.objects.delete'),
    ).toBe(false);

    fireEvent.click(within(confirm).getByRole('button', { name: 'bucketsBrowserDeleteSelected' }));

    await waitFor(() => {
      const deleted = request.mock.calls
        .filter((call) => call[0].operationId === 'storageProviders.objects.delete')
        .map((call) => call[0].pathParams.objectKey);
      // 两个被选中的对象都要删，且都带桶名。
      expect(deleted.sort()).toEqual(['notes.txt', 'photos/']);
    });
  });

  it('selects everything visible with the header checkbox and clears with Escape', async () => {
    const request = createRequest();
    renderDialog(request);

    expect(await screen.findByText('notes.txt')).toBeTruthy();

    fireEvent.click(screen.getByRole('checkbox', { name: 'bucketsBrowserSelectAll' }));
    expect(await screen.findByText('bucketsBrowserSelected')).toBeTruthy();
    expect(screen.getByRole('checkbox', { name: 'bucketsBrowserSelectAll' })).toHaveProperty(
      'checked',
      true,
    );

    // Esc 清空选中：动作组回到新建/上传。
    fireEvent.keyDown(screen.getByText('notes.txt'), { key: 'Escape' });
    await waitFor(() => {
      expect(screen.queryByText('bucketsBrowserSelected')).toBeNull();
    });
    expect(screen.getByRole('button', { name: 'upload' })).toBeTruthy();
  });

  it('uploads dropped files into the current folder', async () => {
    const request = createRequest();
    renderDialog(request);

    expect(await screen.findByText('notes.txt')).toBeTruthy();

    const dropped = new File(['dropped'], 'dropped.txt', { type: 'text/plain' });
    const list = screen.getByText('notes.txt').closest('div[class*="overflow-y-auto"]');
    expect(list).not.toBeNull();
    // 拖入时先给出可落点的可视提示，否则用户不知道松手会发生什么。
    fireEvent.dragEnter(list as HTMLElement, { dataTransfer: { types: ['Files'] } });
    expect(screen.getByText('bucketsBrowserDropHint')).toBeTruthy();
    fireEvent.drop(list as HTMLElement, { dataTransfer: { files: [dropped], types: ['Files'] } });
    await waitFor(() => {
      expect(screen.queryByText('bucketsBrowserDropHint')).toBeNull();
    });

    await waitFor(() => {
      expect(
        request.mock.calls.some(
          (call) =>
            call[0].operationId === 'storageProviders.objects.content.update'
            && call[0].pathParams.objectKey === 'dropped.txt'
            && call[0].query.bucket === MANAGED_BUCKET,
        ),
      ).toBe(true);
    });
  });

  it('clears the selection when the operator navigates into a folder', async () => {
    const request = createRequest();
    renderDialog(request);

    expect(await screen.findByText('notes.txt')).toBeTruthy();
    fireEvent.click(within(screen.getAllByRole('row')[1]).getByRole('checkbox'));
    expect(await screen.findByText('bucketsBrowserSelected')).toBeTruthy();

    fireEvent.click(screen.getByText('photos/'));

    // 跨目录的选中集在界面上无从表达：换目录必须丢弃它，否则会误删看不见的对象。
    await waitFor(() => {
      expect(screen.queryByText('bucketsBrowserSelected')).toBeNull();
    });
  });

  it('renames an object with a bucket-scoped copy and delete', async () => {
    const request = createRequest();
    renderDialog(request);

    expect(await screen.findByText('photos/')).toBeTruthy();
    fireEvent.click(screen.getByText('photos/'));

    // 等新一页渲染完再点行内动作：加载中的按钮是禁用的，点它不会打开输入框。
    const coverRow = (await screen.findByText('cover.png')).closest('tr') as HTMLElement;
    fireEvent.click(within(coverRow).getByRole('button', { name: 'rename' }));

    const prompt = await screen.findByRole('dialog', { name: 'renameDialogTitle' });
    fireEvent.change(within(prompt).getByRole('textbox'), { target: { value: 'renamed.png' } });
    fireEvent.click(within(prompt).getByRole('button', { name: 'confirm' }));

    await waitFor(() => {
      expect(
        request.mock.calls.some(
          (call) =>
            call[0].operationId === 'storageProviders.objects.copy'
            && call[0].body.sourceObjectKey === 'photos/cover.png'
            && call[0].body.destinationObjectKey === 'photos/renamed.png'
            && call[0].body.sourceBucket === MANAGED_BUCKET
            && call[0].body.destinationBucket === MANAGED_BUCKET,
        ),
      ).toBe(true);
    });
    // copy 之后删除源对象：同桶改名不会留下双份，也不会误删到配置桶。
    await waitFor(() => {
      expect(
        request.mock.calls.some(
          (call) =>
            call[0].operationId === 'storageProviders.objects.delete'
            && call[0].pathParams.objectKey === 'photos/cover.png'
            && call[0].query.bucket === MANAGED_BUCKET,
        ),
      ).toBe(true);
    });
  });

  it('deletes the directory placeholder only after the confirmation', async () => {
    const request = createRequest();
    renderDialog(request);

    expect(await screen.findByText('photos/')).toBeTruthy();
    fireEvent.click(screen.getAllByRole('button', { name: 'del' })[0]);

    // 确认之前不能发删除请求。
    expect(
      request.mock.calls.some((call) => call[0].operationId === 'storageProviders.objects.delete'),
    ).toBe(false);

    const confirmDialog = document.querySelector('[data-sdk-ui="confirm-dialog"]') as HTMLElement;
    expect(confirmDialog).toBeTruthy();
    fireEvent.click(within(confirmDialog).getByRole('button', { name: 'del' }));

    await waitFor(() => {
      expect(
        request.mock.calls.some(
          (call) =>
            call[0].operationId === 'storageProviders.objects.delete'
            && call[0].pathParams.objectKey === 'photos/'
            && call[0].query.bucket === MANAGED_BUCKET,
        ),
      ).toBe(true);
    });
  });

  it('opens the preview for a clicked file row and reads that object', async () => {
    const request = createRequest();
    renderDialog(request);

    // 点文件行即预览：内容按被管理的桶读取，预览组件本身不认识对象存储。
    fireEvent.click(await screen.findByText('notes.txt'));

    await waitFor(() => {
      expect(
        request.mock.calls.some(
          (call) =>
            call[0].operationId === 'storageProviders.objects.content.retrieve'
            && call[0].pathParams.objectKey === 'notes.txt'
            && call[0].query.bucket === MANAGED_BUCKET,
        ),
      ).toBe(true);
    });
    expect(await screen.findByText('hello world')).toBeTruthy();
  });

  it('steps to the neighbouring file from the preview without leaving it', async () => {
    const request = createRequest();
    renderDialog(request);

    // 目录里是 photos/、notes.txt、photos/cover.png：可预览序列只含两个文件，
    // 按名称排序后 notes.txt 在前。
    fireEvent.click(await screen.findByText('notes.txt'));
    expect(await screen.findByText('hello world')).toBeTruthy();
    expect(await screen.findByText('1 / 2')).toBeTruthy();

    // 下一个文件：切到 cover.png，并立刻按新 key 取内容（不是换一个"位置"而已）。
    fireEvent.click(screen.getByRole('button', { name: 'Next file' }));

    await waitFor(() => {
      expect(
        request.mock.calls.some(
          (call) =>
            call[0].operationId === 'storageProviders.objects.content.retrieve'
            && call[0].pathParams.objectKey === 'photos/cover.png'
            && call[0].query.bucket === MANAGED_BUCKET,
        ),
      ).toBe(true);
    });
    expect(await screen.findByText('2 / 2')).toBeTruthy();

    // 到序列末端：下一个按钮禁用，不会绕回开头。
    expect(
      (screen.getByRole('button', { name: 'Next file' }) as HTMLButtonElement).disabled,
    ).toBe(true);
    // 序列只含文件：目录 photos/ 不在其中，所以"三个条目"里 total 是 2。
  });

  it('moves to the next file with the arrow key while the preview is open', async () => {
    const request = createRequest();
    renderDialog(request);

    fireEvent.click(await screen.findByText('notes.txt'));
    expect(await screen.findByText('hello world')).toBeTruthy();

    // 焦点在预览面板上时方向键归导航；输入框/播放器里则不会（各有各的主人）。
    const surface = screen.getByText('hello world').closest('section');
    expect(surface).not.toBeNull();
    fireEvent.keyDown(surface as HTMLElement, { key: 'ArrowRight' });

    await waitFor(() => {
      expect(
        request.mock.calls.some(
          (call) =>
            call[0].operationId === 'storageProviders.objects.content.retrieve'
            && call[0].pathParams.objectKey === 'photos/cover.png',
        ),
      ).toBe(true);
    });
  });

  it('opens the preview full screen with exactly one close affordance', async () => {
    const request = createRequest();
    renderDialog(request);

    fireEvent.click(await screen.findByText('notes.txt'));
    expect(await screen.findByText('hello world')).toBeTruthy();

    // 预览铺满视口：inset-0 覆盖库里的定宽定高，且把定位偏移归零——
    // 少了 translate-*-0，库里那对 -translate-1/2 会把整屏元素推出画面。
    const dialogs = screen.getAllByRole('dialog');
    const previewDialog = dialogs[dialogs.length - 1];
    expect(previewDialog.className).toContain('inset-0');
    expect(previewDialog.className).toContain('translate-x-0');
    expect(previewDialog.className).toContain('translate-y-0');
    expect(previewDialog.className).not.toContain('h-[90vh]');
    expect(previewDialog.className).toContain('rounded-none');
    // 关键：库里的定宽定高与居中定位必须真的被合并掉，否则 inset-0 与 left-1/2 同时生效
    // 会变成一个"贴左上角但只占半个屏"的怪东西。
    for (const stale of [
      'left-1/2',
      'top-1/2',
      '-translate-x-1/2',
      '-translate-y-1/2',
      'max-h-[min(88vh,56rem)]',
      'w-[min(96vw,96rem)]',
      'h-[min(96vh,64rem)]',
    ]) {
      expect(previewDialog.className).not.toContain(stale);
    }

    // 库里那个浮在 right-4 top-4 的 X 与文件浏览器的关闭按钮落在同一处，必须关掉：
    // 否则两个 X 叠在一起，用户点哪个都心里没底。
    // 预览打开时，整棵树里可被访问到的关闭按钮只剩一个，而且它在预览对话框内部
    // ——文件浏览器自己的 X 已被 Radix 移出无障碍树（也就不会再和预览的 X 抢位置）。
    const closeButtons = screen.getAllByRole('button', { name: 'Close' });
    expect(closeButtons).toHaveLength(1);
    expect(previewDialog.contains(closeButtons[0])).toBe(true);
  });
});
