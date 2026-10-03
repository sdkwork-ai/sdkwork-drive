import type { BucketObjectCategoryFilter } from './bucketObjectCategory';

/** 翻译函数签名（`useTranslation` 的 `t` 的结构子集）。 */
export type BucketCategoryTranslator = (
  key: string,
  params?: Record<string, string | number>,
) => string;

/**
 * 分类显示名。
 *
 * 分类筛选器与文件列表的「类型」列必须给出同一个词，所以映射只有这一份。
 */
export function bucketCategoryLabel(
  t: BucketCategoryTranslator,
  value: BucketObjectCategoryFilter,
): string {
  switch (value) {
    case 'all':
      return t('bucketsBrowserCategoryAll');
    case 'folder':
      return t('bucketsBrowserCategoryFolder');
    case 'image':
      return t('bucketsBrowserCategoryImage');
    case 'video':
      return t('bucketsBrowserCategoryVideo');
    case 'audio':
      return t('bucketsBrowserCategoryAudio');
    case 'document':
      return t('bucketsBrowserCategoryDocument');
    case 'archive':
      return t('bucketsBrowserCategoryArchive');
    default:
      return t('bucketsBrowserCategoryOther');
  }
}
