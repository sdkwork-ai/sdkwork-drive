/* @vitest-environment jsdom */

import { describe, expect, it } from 'vitest';
import type { StorageProviderObjectView } from 'sdkwork-drive-pc-admin-storage-providers';
import {
  DEFAULT_BUCKET_OBJECT_SORT,
  nextBucketObjectSort,
  sortBucketObjects,
} from '../src/utils/sortBucketObjects';

function object(
  key: string,
  overrides: Partial<StorageProviderObjectView> = {},
): StorageProviderObjectView {
  return {
    key,
    isFolder: false,
    sizeBytes: 0,
    ...overrides,
  } as StorageProviderObjectView;
}

const FOLDER = object('photos/', { isFolder: true, sizeBytes: 0 });
const BIG = object('big.bin', { sizeBytes: 9_000, lastModifiedIso: '2026-09-01T10:00:00Z' });
const SMALL = object('small.txt', { sizeBytes: 10, lastModifiedIso: '2026-09-30T10:00:00Z' });
const NO_TIME = object('unknown.dat', { sizeBytes: 500 });

const ITEMS = [BIG, FOLDER, SMALL, NO_TIME];

describe('sortBucketObjects', () => {
  it('keeps folders first when sorting by size or modified time', () => {
    const bySize = sortBucketObjects(ITEMS, { key: 'size', direction: 'desc' }, 'zh-CN');
    expect(bySize[0]).toBe(FOLDER);
    expect(bySize.slice(1).map((item) => item.key)).toEqual([
      'big.bin',
      'unknown.dat',
      'small.txt',
    ]);

    const byTime = sortBucketObjects(ITEMS, { key: 'modified', direction: 'desc' }, 'zh-CN');
    expect(byTime[0]).toBe(FOLDER);
    // 没有修改时间的排最后（按"最旧"处理），不会冒到最前面。
    expect(byTime.at(-1)?.key).toBe('unknown.dat');
  });

  it('flips direction on the same key and picks a sensible one on a new key', () => {
    expect(nextBucketObjectSort(DEFAULT_BUCKET_OBJECT_SORT, 'name')).toEqual({
      key: 'name',
      direction: 'desc',
    });
    // 换到大小/时间时，先给"从大到小 / 从新到旧"——这是这两列最常见的意图。
    expect(nextBucketObjectSort(DEFAULT_BUCKET_OBJECT_SORT, 'size')).toEqual({
      key: 'size',
      direction: 'desc',
    });
    expect(nextBucketObjectSort({ key: 'size', direction: 'desc' }, 'modified')).toEqual({
      key: 'modified',
      direction: 'desc',
    });
    expect(nextBucketObjectSort({ key: 'size', direction: 'asc' }, 'name')).toEqual({
      key: 'name',
      direction: 'asc',
    });
  });

  it('sorts names the way a file manager does, with a stable tie-break', () => {
    const items = [
      object('b.txt', { sizeBytes: 1 }),
      object('a10.txt', { sizeBytes: 1 }),
      object('a2.txt', { sizeBytes: 1 }),
    ];
    const byName = sortBucketObjects(items, { key: 'name', direction: 'asc' }, 'en-US');
    // 数字按数值比较：a2 在 a10 之前，而不是字典序的 a10 < a2。
    expect(byName.map((item) => item.key)).toEqual(['a2.txt', 'a10.txt', 'b.txt']);

    // 同大小时按名称兜底：否则同尺寸的对象顺序会随刷新变化。
    const sameSize = [
      object('z.txt', { sizeBytes: 5 }),
      object('a.txt', { sizeBytes: 5 }),
    ];
    expect(
      sortBucketObjects(sameSize, { key: 'size', direction: 'desc' }, 'en-US').map((i) => i.key),
    ).toEqual(['a.txt', 'z.txt']);
  });

  it('does not mutate the input page', () => {
    const input = [...ITEMS];
    sortBucketObjects(input, { key: 'size', direction: 'desc' }, 'en-US');
    expect(input).toEqual(ITEMS);
  });
});
