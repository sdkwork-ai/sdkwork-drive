/* @vitest-environment jsdom */

import React from 'react';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type {
  StorageProviderAdminService,
  StorageProviderObjectView,
  StorageProviderView,
} from 'sdkwork-drive-pc-admin-storage-providers';
import { BucketObjectManagerDialog } from '../src/components/BucketObjectManagerDialog';

/**
 * 分页行为。
 *
 * 对象列表接口是游标分页（只有 cursor + page_size，没有 offset、没有总数），所以这里的
 * 契约是：
 * - 显示的是"这一页"，不是"越滚越长的列表"；
 * - 下一页带上上一页给出的游标；
 * - 上一页用已经走过的页，不再打请求；
 * - 换每页条数 = 游标链作废，回到第一页。
 *
 * 没有 i18n provider 时 `t` 回落成词条名且不插值，所以页码/每页条数通过
 * `data-bucket-page` / `data-bucket-page-size` 读取，而不是比对渲染出来的字符串。
 */
const PROVIDER = {
  id: 'provider-1',
  displayName: 'MinIO',
  endpointUrl: 'https://minio.internal',
  region: 'cn-north-1',
} as unknown as StorageProviderView;

const PAGE_ONE = [
  { key: 'a-1.txt', isFolder: false, sizeBytes: 1 },
  { key: 'a-2.txt', isFolder: false, sizeBytes: 2 },
  { key: 'a-3.txt', isFolder: false, sizeBytes: 3 },
] as StorageProviderObjectView[];

const PAGE_TWO = [
  { key: 'b-1.txt', isFolder: false, sizeBytes: 4 },
  { key: 'b-2.txt', isFolder: false, sizeBytes: 5 },
] as StorageProviderObjectView[];

function createService() {
  const listObjects = vi.fn(
    async (
      _providerId: string,
      input?: { bucket?: string; pageSize?: number; pageToken?: string },
    ) => {
      if (input?.pageToken === 'cursor-1') {
        return { hasMore: false, items: PAGE_TWO, nextPageToken: undefined };
      }
      if (input?.pageToken !== undefined) {
        throw new Error(`unexpected cursor ${input.pageToken}`);
      }
      return { hasMore: true, items: PAGE_ONE, nextPageToken: 'cursor-1' };
    },
  );
  const service = {
    listObjects,
    readObjectContent: async () => ({ content: '', contentType: 'text/plain' }),
    writeObjectContent: async () => PAGE_ONE[0],
    deleteObject: async () => true,
  } as unknown as StorageProviderAdminService;
  return { listObjects, service };
}

function renderDialog(service: StorageProviderAdminService) {
  return render(
    <BucketObjectManagerDialog
      bucket="team-assets"
      onOpenChange={() => undefined}
      open
      provider={PROVIDER}
      service={service}
    />,
  );
}

/** 当前页码 / 每页条数：取自页脚的数据属性，与界面语言无关。 */
function readPage(container: HTMLElement): string | null {
  return document.querySelector('[data-bucket-page]')?.getAttribute('data-bucket-page') ?? null;
}

function readPageSize(container: HTMLElement): string | null {
  return (
    document.querySelector('[data-bucket-page-size]')?.getAttribute('data-bucket-page-size') ?? null
  );
}

afterEach(() => cleanup());

describe('BucketObjectManagerDialog pagination', () => {
  it('shows one page at a time with a position line', async () => {
    const { listObjects, service } = createService();
    const { container } = renderDialog(service);

    expect(await screen.findByText('a-1.txt')).toBeTruthy();
    expect(readPage(container)).toBe('1');
    expect(readPageSize(container)).toBe('100');
    // 页脚同时给出本页条数与每页条数选择。
    expect(screen.getByRole('combobox', { name: 'bucketsBrowserPageSize' })).toBeTruthy();
    // 第一页没有上一页；还有下一页。
    expect(
      (screen.getByRole('button', { name: 'bucketsBrowserPagePrevious' }) as HTMLButtonElement)
        .disabled,
    ).toBe(true);
    expect(
      (screen.getByRole('button', { name: 'bucketsBrowserPageNext' }) as HTMLButtonElement).disabled,
    ).toBe(false);
    // 首次请求用默认每页条数，且不带游标。
    expect(listObjects.mock.calls[0][1]).toMatchObject({ bucket: 'team-assets', pageSize: 100 });
  });

  it('walks forward with the cursor and backward from memory', async () => {
    const { listObjects, service } = createService();
    const { container } = renderDialog(service);

    expect(await screen.findByText('a-1.txt')).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: 'bucketsBrowserPageNext' }));

    // 第二页：显示的是这一页的内容，第一页的行不再同时存在（不是"追加"）。
    expect(await screen.findByText('b-1.txt')).toBeTruthy();
    expect(screen.queryByText('a-1.txt')).toBeNull();
    expect(readPage(container)).toBe('2');
    expect(
      (screen.getByRole('button', { name: 'bucketsBrowserPageNext' }) as HTMLButtonElement).disabled,
    ).toBe(true);
    expect(listObjects.mock.calls[1][1]).toMatchObject({ pageToken: 'cursor-1' });

    const callsBeforeGoingBack = listObjects.mock.calls.length;
    fireEvent.click(screen.getByRole('button', { name: 'bucketsBrowserPagePrevious' }));

    expect(await screen.findByText('a-1.txt')).toBeTruthy();
    expect(screen.queryByText('b-1.txt')).toBeNull();
    expect(readPage(container)).toBe('1');
    // 上一页是瞬时的：已经走过的页留在内存里，不再打一次请求。
    expect(listObjects.mock.calls.length).toBe(callsBeforeGoingBack);
  });

  it('restarts the cursor chain when the page size changes', async () => {
    const { listObjects, service } = createService();
    const { container } = renderDialog(service);

    expect(await screen.findByText('a-1.txt')).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: 'bucketsBrowserPageNext' }));
    expect(await screen.findByText('b-1.txt')).toBeTruthy();

    fireEvent.change(screen.getByRole('combobox', { name: 'bucketsBrowserPageSize' }), {
      target: { value: '50' },
    });

    await waitFor(() => expect(readPage(container)).toBe('1'));
    expect(await screen.findByText('a-1.txt')).toBeTruthy();
    expect(readPageSize(container)).toBe('50');
    const lastCall = listObjects.mock.calls.at(-1)?.[1];
    expect(lastCall).toMatchObject({ pageSize: 50 });
    // 换页大小必须从第一页重新开始，不能沿用旧游标。
    expect(lastCall?.pageToken).toBeUndefined();
  });

  it('falls back to another region when the bucket is not in the listed one', async () => {
    /*
     * 桶清单是跨地域的，而厂商对"地域用错"的回答就是 NoSuchBucket。所以列目录必须按
     * 候选地域逐个试，并把生效的那个用在后续读写上。
     */
    const listObjects = vi.fn(
      async (_providerId: string, input?: { region?: string }) => {
        if (input?.region === 'ap-beijing') {
          throw new Error('The specified bucket does not exist.');
        }
        return { hasMore: false, items: PAGE_ONE, nextPageToken: undefined };
      },
    );
    const service = {
      listObjects,
      deleteObject: vi.fn(async () => true),
    } as unknown as StorageProviderAdminService;

    render(
      <BucketObjectManagerDialog
        bucket="cross-region"
        onOpenChange={() => undefined}
        open
        provider={{ ...PROVIDER, region: 'ap-guangzhou' } as StorageProviderView}
        region="ap-beijing"
        service={service}
      />,
    );

    // 第一次用桶清单给的地域（失败），第二次回落到服务商配置的地域（成功）。
    expect(await screen.findByText('a-1.txt')).toBeTruthy();
    expect(listObjects.mock.calls.map((call) => call[1]?.region)).toEqual([
      'ap-beijing',
      'ap-guangzhou',
    ]);
    // 左栏显示的是真正生效的地域，而不是配置里那个或清单里那个。
    expect(screen.getByText('ap-guangzhou')).toBeTruthy();
    expect(screen.queryByText('ap-beijing')).toBeNull();
  });

  it('reports which bucket and regions were tried when every region fails', async () => {
    const listObjects = vi.fn(async () => {
      throw new Error('The specified bucket does not exist.');
    });
    const service = { listObjects } as unknown as StorageProviderAdminService;

    render(
      <BucketObjectManagerDialog
        bucket="ghost-bucket"
        onOpenChange={() => undefined}
        open
        provider={{ ...PROVIDER, region: 'ap-guangzhou' } as StorageProviderView}
        region="ap-beijing"
        service={service}
      />,
    );

    // 厂商只说"桶不存在"，真正的信息是"用哪个桶、试过哪些地域"：必须一起给出来。
    expect(await screen.findByText(/bucketsBrowserListErrorContext/)).toBeTruthy();
    expect(listObjects).toHaveBeenCalledTimes(2);
  });

  it('sorts by name, size and modified time from the column headers', async () => {
    /*
     * 接口是游标分页且没有排序参数（S3/COS 只按键名字典序返回），所以排序在已加载的页上
     * 客户端完成：目录永远在前，其余按所选列排，同值按名称兜底。
     */
    const listObjects = vi.fn(async () => ({
      hasMore: false,
      items: [
        { key: 'big.bin', isFolder: false, sizeBytes: 9_000, lastModifiedIso: '2026-09-01T00:00:00Z' },
        { key: 'photos/', isFolder: true, sizeBytes: 0 },
        { key: 'small.txt', isFolder: false, sizeBytes: 10, lastModifiedIso: '2026-09-30T00:00:00Z' },
      ],
      nextPageToken: undefined,
    }));
    const service = { listObjects } as unknown as StorageProviderAdminService;
    renderDialog(service);

    expect(await screen.findByText('small.txt')).toBeTruthy();
    const rowNames = () =>
      Array.from(document.querySelectorAll('tbody tr')).map(
        // 第 0 格是复选框，第 1 格是名称（图标是 svg，不贡献文本）。
        (row) => row.querySelectorAll('td')[1]?.textContent?.trim() ?? '',
      );

    // 缺省：目录在前，其余按名称升序。
    expect(rowNames()).toEqual(['photos/', 'big.bin', 'small.txt']);

    // 按大小：同一列再点一次翻转方向；目录仍在最前。
    // 三个表头按钮共享同一个 aria-label（"按{字段}排序"），按列顺序取第 2 个 = 大小。
    const headers = screen.getAllByRole('button', { name: 'bucketsBrowserSortBy' });
    expect(headers.length).toBe(3);
    fireEvent.click(headers[1]);
    expect(rowNames()).toEqual(['photos/', 'big.bin', 'small.txt']);
    expect(document.querySelector('th[aria-sort="descending"]')).not.toBeNull();
    fireEvent.click(headers[1]);
    expect(rowNames()).toEqual(['photos/', 'small.txt', 'big.bin']);
    expect(document.querySelector('th[aria-sort="ascending"]')).not.toBeNull();

    // 按修改时间：从新到旧是默认方向（"最新改的"是这一列最常见的意图）。
    fireEvent.click(headers[2]);
    expect(rowNames()).toEqual(['photos/', 'small.txt', 'big.bin']);
    expect(document.querySelectorAll('th[aria-sort="descending"]').length).toBe(1);
  });

  it('keeps the header and toolbar out of the scrolling area', async () => {
    const { service } = createService();
    const { container } = renderDialog(service);
    expect(await screen.findByText('a-1.txt')).toBeTruthy();

    // 行区域是唯一的滚动容器；从它一路往上到弹窗正文都不许再出现纵向滚动，
    // 否则滚轮会先滚外层，表头就跟着一起走了。
    const rows = document.querySelector('[data-bucket-rows]');
    expect(rows).not.toBeNull();
    expect(rows?.className).toContain('overflow-y-auto');

    let node: HTMLElement | null = rows?.parentElement ?? null;
    const scrollers: string[] = [];
    while (node && node.getAttribute('data-slot') !== 'modal-content') {
      if (node.className.includes('overflow-y-auto') || node.className.includes('overflow-auto')) {
        scrollers.push(node.className);
      }
      node = node.parentElement;
    }
    // 右栏（aside）不在行区域的祖先链上，所以这里应当一个都没有。
    expect(scrollers).toEqual([]);
  });
});
