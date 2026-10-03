import React from 'react';
import {
  Database,
  File as FileIcon,
  FileArchive,
  FileText,
  FolderOpen,
  Image as ImageIcon,
  Music,
  Video,
} from 'lucide-react';
import {
  providerDisplayName,
  useTranslation,
} from 'sdkwork-drive-pc-admin-storage-providers';
import type { StorageProviderView } from 'sdkwork-drive-pc-admin-storage-providers';
import { formatDriveBytes } from 'sdkwork-drive-pc-commons';
import {
  BUCKET_OBJECT_CATEGORIES,
  type BucketObjectCategory,
  type BucketObjectCategoryFilter,
} from '../utils/bucketObjectCategory';
import { bucketCategoryLabel } from '../utils/bucketObjectCategoryLabel';

/** 分类图标：与文件列表行首图标同源，保证左栏与右栏读起来是同一套分类。 */
export function bucketCategoryIcon(category: BucketObjectCategory, size = 13): React.ReactNode {
  switch (category) {
    case 'folder':
      return <FolderOpen aria-hidden="true" size={size} />;
    case 'image':
      return <ImageIcon aria-hidden="true" size={size} />;
    case 'video':
      return <Video aria-hidden="true" size={size} />;
    case 'audio':
      return <Music aria-hidden="true" size={size} />;
    case 'document':
      return <FileText aria-hidden="true" size={size} />;
    case 'archive':
      return <FileArchive aria-hidden="true" size={size} />;
    default:
      return <FileIcon aria-hidden="true" size={size} />;
  }
}

export interface BucketCategoryRailProps {
  bucket: string;
  category: BucketObjectCategoryFilter;
  counts: Record<BucketObjectCategoryFilter, number>;
  loadedBytes: number;
  onSelectCategory: (category: BucketObjectCategoryFilter) => void;
  provider: StorageProviderView;
  /**
   * 实际生效的地域（请求真正带上的那个）。
   *
   * 桶清单是跨地域的，"配置里的区域"与"访问这个桶用的区域"可能不是同一个；左栏显示
   * 后者，运维才能在列表报 NoSuchBucket 时立刻看出端点对不对。缺省回落到配置区域。
   */
  region?: string;
}

/**
 * 文件管理器左栏：内容分类 + 存储桶信息。
 *
 * 分类计数按「当前页的对象」统计，所以文案里写明了范围——对象存储没有目录级元数据，
 * 把这一层说成整桶分类会是不诚实的数字。
 */
export function BucketCategoryRail({
  bucket,
  category,
  counts,
  loadedBytes,
  onSelectCategory,
  provider,
  region,
}: BucketCategoryRailProps) {
  const { t } = useTranslation();
  const effectiveRegion = region && region !== '' ? region : provider.region;

  return (
    /*
     * `overflow-x-hidden` 是刻意的：`overflow-y-auto` 会把另一轴的计算值变成 `auto`，
     * 于是任何比 232px 宽一点点的子元素都会给左栏拉出一条横向滚动条（截图里的那条）。
     * 纵向滚动照旧，横向一律裁掉——栏内不存在需要横向滚动才能读完的内容。
     */
    <aside className="flex shrink-0 flex-col gap-4 overflow-x-hidden border-b border-neutral-200 bg-neutral-50 px-4 py-4 dark:border-neutral-800 dark:bg-neutral-900/60 lg:w-[232px] lg:overflow-y-auto lg:border-b-0 lg:border-r">
      <section>
        <h3 className="mb-2 text-[11px] font-semibold uppercase tracking-wide text-neutral-500 dark:text-neutral-400">
          {t('bucketsBrowserCategories')}
        </h3>
        <div className="flex flex-wrap gap-1 lg:flex-col">
          {(['all', ...BUCKET_OBJECT_CATEGORIES] as BucketObjectCategoryFilter[]).map((value) => {
            const active = value === category;
            return (
              <button
                key={value}
                type="button"
                aria-pressed={active}
                onClick={() => onSelectCategory(value)}
                className={`flex min-w-0 items-center gap-2 rounded-md px-2 py-1.5 text-left text-xs transition-colors ${
                  active
                    ? 'bg-blue-50 font-medium text-blue-700 dark:bg-blue-950/40 dark:text-blue-200'
                    : 'text-neutral-600 hover:bg-neutral-100 dark:text-neutral-300 dark:hover:bg-neutral-800'
                }`}
              >
                {value === 'all' ? (
                  <Database aria-hidden="true" size={13} />
                ) : (
                  bucketCategoryIcon(value)
                )}
                <span className="min-w-0 flex-1 truncate">{bucketCategoryLabel(t, value)}</span>
                <span className="shrink-0 tabular-nums text-neutral-400">{counts[value]}</span>
              </button>
            );
          })}
        </div>
        <p className="mt-2 text-[11px] leading-relaxed text-neutral-400 dark:text-neutral-500">
          {t('bucketsBrowserCategoryHint')}
        </p>
      </section>

      <section className="min-w-0">
        <h3 className="mb-2 text-[11px] font-semibold uppercase tracking-wide text-neutral-500 dark:text-neutral-400">
          {t('bucketsBrowserInfo')}
        </h3>
        {/*
          显式 `minmax(0,1fr)` 而不是默认的隐式轨道：隐式轨道按 max-content 定宽，
          而 `truncate` 的 max-content 就是整串文本宽度——于是那条长端点 URL 会把
          整个 <dl> 撑得比左栏还宽。轨道一旦被约束，`truncate` 才真的会省略号。
        */}
        <dl className="grid grid-cols-[minmax(0,1fr)] gap-1.5 text-[11px]">
          <div className="flex min-w-0 items-start gap-2">
            <dt className="shrink-0 text-neutral-400">{t('bucketsBrowserBucketLabel')}</dt>
            <dd
              className="min-w-0 flex-1 truncate font-mono text-neutral-700 dark:text-neutral-200"
              title={bucket}
            >
              {bucket}
            </dd>
          </div>
          <div className="flex min-w-0 items-start gap-2">
            <dt className="shrink-0 text-neutral-400">{t('bucketsBrowserProviderLabel')}</dt>
            <dd
              className="min-w-0 flex-1 truncate text-neutral-700 dark:text-neutral-200"
              title={providerDisplayName(t, provider)}
            >
              {providerDisplayName(t, provider)}
            </dd>
          </div>
          <div className="flex min-w-0 items-start gap-2">
            <dt className="shrink-0 text-neutral-400">{t('bucketsBrowserEndpointLabel')}</dt>
            <dd
              className="min-w-0 flex-1 truncate font-mono text-neutral-700 dark:text-neutral-200"
              title={provider.endpointUrl}
            >
              {provider.endpointUrl}
            </dd>
          </div>
          {effectiveRegion ? (
            <div className="flex min-w-0 items-start gap-2">
              <dt className="shrink-0 text-neutral-400">{t('bucketsBrowserRegionLabel')}</dt>
              <dd
                className="min-w-0 flex-1 truncate text-neutral-700 dark:text-neutral-200"
                title={
                  provider.region && effectiveRegion !== provider.region
                    ? t('bucketsBrowserRegionFromBucket')
                    : effectiveRegion
                }
              >
                {effectiveRegion}
              </dd>
            </div>
          ) : null}
          <div className="flex min-w-0 items-start gap-2">
            <dt className="shrink-0 text-neutral-400">{t('bucketsBrowserObjectsLabel')}</dt>
            <dd className="text-neutral-700 tabular-nums dark:text-neutral-200">{counts.all}</dd>
          </div>
          <div className="flex min-w-0 items-start gap-2">
            <dt className="shrink-0 text-neutral-400">{t('bucketsBrowserLoadedBytes')}</dt>
            <dd className="text-neutral-700 tabular-nums dark:text-neutral-200">
              {formatDriveBytes(loadedBytes)}
            </dd>
          </div>
        </dl>
      </section>
    </aside>
  );
}
