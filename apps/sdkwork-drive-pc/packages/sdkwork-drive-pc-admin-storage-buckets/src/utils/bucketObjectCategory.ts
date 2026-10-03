/**
 * 存储桶对象的内容分类。
 *
 * 网盘式文件管理器的左栏按「内容分类」筛选，右栏是文件列表。对象存储只给到
 * objectKey 与 content_type，没有目录级元数据，所以分类在这里由 key 的后缀
 * （缺失时回落到 content_type）推导，并且只作用于当前目录已列出的对象——左栏
 * 因此显式说明统计范围，而不是暗示它是整个桶的全量分类。
 */

export type BucketObjectCategory =
  | 'folder'
  | 'image'
  | 'video'
  | 'audio'
  | 'document'
  | 'archive'
  | 'other';

/** 左栏展示顺序：文件夹在前，其余按「可预览性」从高到低。 */
export const BUCKET_OBJECT_CATEGORIES: readonly BucketObjectCategory[] = [
  'folder',
  'image',
  'video',
  'audio',
  'document',
  'archive',
  'other',
];

/** 供筛选器使用的联合：`all` 表示不过滤。 */
export type BucketObjectCategoryFilter = BucketObjectCategory | 'all';

const CATEGORY_EXTENSIONS: Record<
  Exclude<BucketObjectCategory, 'folder' | 'other'>,
  readonly string[]
> = {
  image: ['png', 'jpg', 'jpeg', 'gif', 'webp', 'bmp', 'svg', 'ico', 'tif', 'tiff', 'avif', 'heic'],
  video: ['mp4', 'mov', 'm4v', 'mkv', 'webm', 'avi', 'wmv', 'flv', 'mpeg', 'mpg', 'ts', '3gp'],
  audio: ['mp3', 'wav', 'flac', 'aac', 'ogg', 'oga', 'm4a', 'wma', 'opus', 'aiff', 'mid'],
  document: [
    'pdf', 'doc', 'docx', 'xls', 'xlsx', 'ppt', 'pptx', 'txt', 'md', 'markdown', 'csv', 'tsv',
    'rtf', 'odt', 'ods', 'odp', 'json', 'xml', 'yaml', 'yml', 'html', 'htm', 'log', 'ini', 'conf',
  ],
  archive: ['zip', 'rar', '7z', 'tar', 'gz', 'tgz', 'bz2', 'xz', 'zst', 'jar', 'war'],
};

const CONTENT_TYPE_PREFIX_CATEGORY: readonly (readonly [string, BucketObjectCategory])[] = [
  ['image/', 'image'],
  ['video/', 'video'],
  ['audio/', 'audio'],
  ['text/', 'document'],
  ['application/pdf', 'document'],
  ['application/json', 'document'],
  ['application/xml', 'document'],
  ['application/zip', 'archive'],
  ['application/x-tar', 'archive'],
  ['application/gzip', 'archive'],
];

/** key 的扩展名（小写、不含点）；没有扩展名时返回 `''`。 */
export function objectExtension(objectKey: string): string {
  const name = objectKey.split('/').filter(Boolean).at(-1) ?? objectKey;
  const dot = name.lastIndexOf('.');
  if (dot <= 0 || dot === name.length - 1) {
    return '';
  }
  return name.slice(dot + 1).toLowerCase();
}

export interface ClassifiableBucketObject {
  key: string;
  contentType?: string;
  isFolder: boolean;
}

export function resolveBucketObjectCategory(
  object: ClassifiableBucketObject,
): BucketObjectCategory {
  if (object.isFolder) {
    return 'folder';
  }
  const extension = objectExtension(object.key);
  if (extension) {
    for (const [category, extensions] of Object.entries(CATEGORY_EXTENSIONS)) {
      if (extensions.includes(extension)) {
        return category as BucketObjectCategory;
      }
    }
  }
  const contentType = object.contentType?.toLowerCase();
  if (contentType) {
    for (const [prefix, category] of CONTENT_TYPE_PREFIX_CATEGORY) {
      if (contentType.startsWith(prefix)) {
        return category;
      }
    }
  }
  return 'other';
}

export function matchesBucketObjectCategory(
  object: ClassifiableBucketObject,
  filter: BucketObjectCategoryFilter,
): boolean {
  return filter === 'all' || resolveBucketObjectCategory(object) === filter;
}

/** 每个分类在当前目录已加载对象中的数量（`all` 是总数）。 */
export function countBucketObjectsByCategory(
  objects: readonly ClassifiableBucketObject[],
): Record<BucketObjectCategoryFilter, number> {
  const counts: Record<BucketObjectCategoryFilter, number> = {
    all: objects.length,
    folder: 0,
    image: 0,
    video: 0,
    audio: 0,
    document: 0,
    archive: 0,
    other: 0,
  };
  for (const object of objects) {
    counts[resolveBucketObjectCategory(object)] += 1;
  }
  return counts;
}
