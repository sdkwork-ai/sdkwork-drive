import type { StorageProviderObjectView } from 'sdkwork-drive-pc-admin-storage-providers';

/** 可排序的列。名称是缺省，大小与修改时间是运营最常用来找"谁占地方/什么时候改的"。 */
export type BucketObjectSortKey = 'name' | 'size' | 'modified';

export type BucketObjectSortDirection = 'asc' | 'desc';

export interface BucketObjectSort {
  key: BucketObjectSortKey;
  direction: BucketObjectSortDirection;
}

export const DEFAULT_BUCKET_OBJECT_SORT: BucketObjectSort = { key: 'name', direction: 'asc' };

/**
 * 生成一个"点击表头"的下一状态。
 *
 * 同一列上点击只翻转方向；换列时给一个符合直觉的初始方向——名称从 A 开始、大小与时间
 * 从大/新开始（想看"最大的""最新的"是这两个列最常见的意图）。
 */
export function nextBucketObjectSort(
  current: BucketObjectSort,
  key: BucketObjectSortKey,
): BucketObjectSort {
  if (current.key === key) {
    return { key, direction: current.direction === 'asc' ? 'desc' : 'asc' };
  }
  return { key, direction: key === 'name' ? 'asc' : 'desc' };
}

/** 修改时间的可比较值；缺失或无法解析一律按"最旧"处理，避免它们冒到最前面。 */
function modifiedTimeOf(object: StorageProviderObjectView): number {
  if (!object.lastModifiedIso) {
    return Number.NEGATIVE_INFINITY;
  }
  const time = new Date(object.lastModifiedIso).getTime();
  return Number.isNaN(time) ? Number.NEGATIVE_INFINITY : time;
}

/**
 * 排序一页对象。
 *
 * 两条产品规则：
 * 1. **目录永远在前**：网盘里用户先找文件夹，再找文件；把目录混进文件按大小排序会让人
 *    找不到入口。目录内部仍按名称排（目录没有大小、也通常没有修改时间）。
 * 2. **比较必须稳定**：同大小/同时间的对象按名称兜底，否则每次渲染顺序都可能变，
 *    看起来像列表在自己跳动。
 *
 * `language` 交给 `localeCompare`：中文目录按拼音、英文按字母，都比按码点排更符合预期。
 */
export function sortBucketObjects(
  objects: readonly StorageProviderObjectView[],
  sort: BucketObjectSort,
  language: string,
): StorageProviderObjectView[] {
  const byName = (
    left: StorageProviderObjectView,
    right: StorageProviderObjectView,
  ): number => left.key.localeCompare(right.key, language, { numeric: true, sensitivity: 'base' });

  const direction = sort.direction === 'asc' ? 1 : -1;
  const compare = (
    left: StorageProviderObjectView,
    right: StorageProviderObjectView,
  ): number => {
    const leftIsFolder = left.isFolder ? 0 : 1;
    const rightIsFolder = right.isFolder ? 0 : 1;
    if (leftIsFolder !== rightIsFolder) {
      return leftIsFolder - rightIsFolder;
    }
    // 两个目录之间没有大小/时间可比，直接按名称。
    if (sort.key !== 'name' && !left.isFolder && !right.isFolder) {
      const leftValue = sort.key === 'size' ? left.sizeBytes : modifiedTimeOf(left);
      const rightValue = sort.key === 'size' ? right.sizeBytes : modifiedTimeOf(right);
      if (leftValue !== rightValue) {
        return (leftValue - rightValue) * direction;
      }
    }
    return byName(left, right);
  };

  return [...objects].sort(compare);
}
