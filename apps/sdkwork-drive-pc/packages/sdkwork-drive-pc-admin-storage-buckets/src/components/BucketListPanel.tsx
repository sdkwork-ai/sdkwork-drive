import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import {
  CheckCircle2,
  CircleAlert,
  Database,
  FolderOpen,
  LoaderCircle,
  RefreshCw,
  Search,
  ShieldCheck,
  Trash2,
} from 'lucide-react';
import {
  BADGE_BASE_CLASS,
  CARD_CLASS,
  DANGER_BUTTON_CLASS,
  GHOST_BUTTON_CLASS,
  INPUT_CLASS,
  PRIMARY_BUTTON_CLASS,
  SECONDARY_BUTTON_CLASS,
  formatDriveDate,
  formatMutationError,
  providerRegionLabel,
  useTranslation,
} from 'sdkwork-drive-pc-admin-storage-providers';
import type {
  StorageProviderAdminService,
  StorageProviderBucketListItemView,
  StorageProviderView,
} from 'sdkwork-drive-pc-admin-storage-providers';
import { ConfirmDialog } from '@sdkwork/ui-pc-react';

export interface BucketListPanelProps {
  /**
   * 打开某个桶的文件管理器。
   *
   * 地域随之一起交出去：桶清单是账号级读取、跨地域，读文件必须按这个桶自己的地域找
   * 端点，而不是按服务商配置里写的那个地域。
   */
  onBrowse: (bucket: string, region?: string) => void;
  provider: StorageProviderView;
  service: StorageProviderAdminService;
}

/**
 * 一个服务商账号下可见的存储桶列表。
 *
 * 列表来自厂商的 ListBuckets，所以运维看到的是账号里真实存在的桶；配置里设定的那个
 * 桶单独标记，并在同一行保留检查 / 初始化 / 删除这三个既有动作——它们只作用于配置桶，
 * 因为对象写入的目标桶由配置决定，任意桶的写入只有在浏览时按请求指定才成立。
 */
export function BucketListPanel({ onBrowse, provider, service }: BucketListPanelProps) {
  const { t, language } = useTranslation();
  const [buckets, setBuckets] = useState<StorageProviderBucketListItemView[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [bucketExists, setBucketExists] = useState<boolean | null>(null);
  const [pendingDelete, setPendingDelete] = useState(false);
  const [initializeOutcome, setInitializeOutcome] = useState<'created' | 'consistent' | null>(null);
  const [query, setQuery] = useState('');
  /** 请求序号：宿主重渲染会换 `t` 的标识，从而换掉 `loadBuckets`；迟到的旧响应必须丢弃。 */
  const loadSeqRef = useRef(0);
  const tRef = useRef(t);
  tRef.current = t;

  const loadBuckets = useCallback(async () => {
    const seq = ++loadSeqRef.current;
    setLoading(true);
    setError(null);
    try {
      const items = await service.listBuckets(provider.id);
      if (seq !== loadSeqRef.current) {
        return;
      }
      setBuckets(items);
      // 厂商列表对配置桶是权威的：它出现即可判定可达，无需额外的 HEAD。
      setBucketExists((current) =>
        items.some((item) => item.bucket === provider.bucket) ? true : current,
      );
    } catch (err) {
      if (seq !== loadSeqRef.current) {
        return;
      }
      setError(formatMutationError(err, tRef.current('errorLoadBuckets')));
    } finally {
      if (seq === loadSeqRef.current) {
        setLoading(false);
      }
    }
  }, [provider.bucket, provider.id, service]);

  // 切换服务商配置即自动同步该账号的桶，与左侧选择保持一致。
  useEffect(() => {
    void loadBuckets();
  }, [loadBuckets]);

  const checkBucket = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await service.headBucket(provider.id);
      setBucketExists(result.exists);
    } catch (err) {
      setError(formatMutationError(err, t('errorCheckBucket')));
    } finally {
      setLoading(false);
    }
  }, [provider.id, service, t]);

  const initializeBucket = useCallback(async () => {
    setLoading(true);
    setError(null);
    setInitializeOutcome(null);
    try {
      const result = await service.initializeBucket(provider.id);
      setBucketExists(true);
      setInitializeOutcome(result.changed ? 'created' : 'consistent');
      await loadBuckets();
    } catch (err) {
      setError(formatMutationError(err, t('errorInitializeBucket')));
    } finally {
      setLoading(false);
    }
  }, [loadBuckets, provider.id, service, t]);

  const deleteBucket = useCallback(async () => {
    setPendingDelete(false);
    setLoading(true);
    setError(null);
    setInitializeOutcome(null);
    try {
      await service.deleteBucket(provider.id);
      setBucketExists(false);
      await loadBuckets();
    } catch (err) {
      setError(formatMutationError(err, t('errorDeleteBucket')));
    } finally {
      setLoading(false);
    }
  }, [loadBuckets, provider.id, service, t]);

  const normalizedQuery = query.trim().toLowerCase();
  const visibleBuckets = useMemo(
    () =>
      normalizedQuery
        ? buckets.filter((bucket) => bucket.bucket.toLowerCase().includes(normalizedQuery))
        : buckets,
    [buckets, normalizedQuery],
  );

  return (
    <>
      <section className={CARD_CLASS}>
        <div className="flex flex-wrap items-center gap-2 border-b border-neutral-100 px-5 py-3 dark:border-neutral-800">
          <Database aria-hidden="true" className="shrink-0 text-neutral-400" size={15} />
          <h3 className="shrink-0 text-sm font-semibold">{t('bucketsListTitle')}</h3>
          <span
            className={`${BADGE_BASE_CLASS} shrink-0 bg-neutral-100 text-neutral-600 dark:bg-neutral-800 dark:text-neutral-300`}
          >
            {t('bucketsListSummary', { count: buckets.length })}
          </span>

          <span className="flex-1" />

          <div className="relative w-full shrink-0 sm:w-56">
            <Search
              aria-hidden="true"
              className="pointer-events-none absolute left-2.5 top-1/2 -translate-y-1/2 text-neutral-400"
              size={14}
            />
            <input
              type="search"
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              placeholder={t('bucketsSearchPlaceholder')}
              aria-label={t('bucketsSearchPlaceholder')}
              className={`${INPUT_CLASS} h-8 pl-8 text-[13px]`}
            />
          </div>
          <button
            type="button"
            onClick={() => void loadBuckets()}
            disabled={loading}
            className={`${GHOST_BUTTON_CLASS} shrink-0`}
          >
            <RefreshCw aria-hidden="true" className={loading ? 'animate-spin' : undefined} size={14} />
            {t('refresh')}
          </button>
        </div>

        <div className="px-5 py-4">
          {error ? (
            <div className="mb-3 flex items-start gap-2 rounded-md border border-red-200 bg-red-50 px-3 py-2 text-xs text-red-700 dark:border-red-900 dark:bg-red-950/20 dark:text-red-300">
              <CircleAlert aria-hidden="true" className="mt-0.5 shrink-0" size={14} />
              <span className="flex-1">{error}</span>
            </div>
          ) : null}

          {/* 配置桶：写入目标与三个既有动作都在这一条里 */}
          <div className="flex flex-wrap items-center gap-x-4 gap-y-2 rounded-md border border-neutral-200 px-3 py-2 dark:border-neutral-700">
            <div className="flex min-w-0 items-center gap-2">
              <span className="shrink-0 text-xs text-neutral-500">{t('configuredBucket')}</span>
              <span className="truncate font-mono text-sm font-medium">{provider.bucket}</span>
              {bucketExists !== null ? (
                <span
                  className={`${BADGE_BASE_CLASS} shrink-0 ${
                    bucketExists
                      ? 'bg-emerald-100 text-emerald-700 dark:bg-emerald-900/40 dark:text-emerald-300'
                      : 'bg-red-100 text-red-700 dark:bg-red-900/40 dark:text-red-300'
                  }`}
                >
                  <span
                    className={`h-1.5 w-1.5 rounded-full ${bucketExists ? 'bg-emerald-500' : 'bg-red-500'}`}
                  />
                  {bucketExists ? t('bucketReachable') : t('bucketUnreachable')}
                </span>
              ) : null}
            </div>

            <span className="hidden flex-1 sm:block" />

            <div className="flex flex-wrap items-center gap-2">
              <button
                type="button"
                onClick={() => void checkBucket()}
                disabled={loading}
                className={SECONDARY_BUTTON_CLASS}
              >
                {loading ? (
                  <LoaderCircle aria-hidden="true" className="animate-spin" size={14} />
                ) : (
                  <Search aria-hidden="true" size={14} />
                )}
                {t('checkExists')}
              </button>
              <button
                type="button"
                onClick={() => void initializeBucket()}
                disabled={loading}
                title={t('initializeBucketHint')}
                className={PRIMARY_BUTTON_CLASS}
              >
                <ShieldCheck aria-hidden="true" size={14} />
                {t('initializeBucket')}
              </button>
              {bucketExists ? (
                <button
                  type="button"
                  onClick={() => setPendingDelete(true)}
                  disabled={loading}
                  className={DANGER_BUTTON_CLASS}
                >
                  <Trash2 aria-hidden="true" size={14} />
                  {t('deleteBucket')}
                </button>
              ) : null}
            </div>
          </div>

          {initializeOutcome ? (
            <p
              className={`mt-3 flex items-center gap-1.5 text-xs ${
                initializeOutcome === 'created'
                  ? 'text-emerald-600 dark:text-emerald-400'
                  : 'text-blue-600 dark:text-blue-400'
              }`}
            >
              <CheckCircle2 aria-hidden="true" size={13} />
              {initializeOutcome === 'created'
                ? t('bucketInitializedCreated')
                : t('bucketInitializedConsistent')}
            </p>
          ) : null}

          <p className="mt-3 text-[11px] leading-relaxed text-neutral-400 dark:text-neutral-500">
            {t('bucketsListHint')}
          </p>

          {visibleBuckets.length > 0 ? (
            <div className="mt-3 overflow-x-auto">
              {/*
                行高 48px，与文件管理弹窗的行、以及控制台其他台账同级：这一栏是运维扫读的
                台账，不是紧凑的辅助列表。行为也与之对齐——整行是按钮，可 Tab、可 Enter/Space
                打开，行内控件用 stopPropagation 避免重复触发。
              */}
              <table className="w-full border-separate border-spacing-0 text-[13px]">
                <thead>
                  <tr className="text-left text-[11px] font-medium uppercase tracking-wide text-neutral-400 dark:text-neutral-500">
                    <th className="border-b border-neutral-200 px-3 py-2 font-medium dark:border-neutral-700">
                      {t('bucketsColBucket')}
                    </th>
                    <th className="border-b border-neutral-200 px-3 py-2 font-medium dark:border-neutral-700">
                      {t('bucketsColRegion')}
                    </th>
                    <th className="border-b border-neutral-200 px-3 py-2 font-medium dark:border-neutral-700">
                      {t('bucketsColRole')}
                    </th>
                    <th className="hidden border-b border-neutral-200 px-3 py-2 font-medium dark:border-neutral-700 sm:table-cell">
                      {t('bucketsColCreated')}
                    </th>
                    <th className="border-b border-neutral-200 px-3 py-2 text-right font-medium dark:border-neutral-700">
                      {t('bucketsColAction')}
                    </th>
                  </tr>
                </thead>
                <tbody>
                  {visibleBuckets.map((bucket) => {
                    const isConfigured = bucket.configured;
                    /*
                      地域只认接口给的那一个值：厂商报了就是它，没报就留空。配置地域不是这个桶的
                      地域——账号级清单跨地域，配置里写的那一个只说明端点默认落在哪里，把它填进
                      这一列等于替厂商断言一个我们并不知道的事实。
                    */
                    const region = bucket.region;
                    const browse = () => onBrowse(bucket.bucket, bucket.region);
                    return (
                      <tr
                        key={bucket.bucket}
                        aria-current={isConfigured ? 'true' : undefined}
                        tabIndex={0}
                        className={`group h-12 cursor-pointer border-b border-neutral-100 outline-none transition-colors last:border-0 focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-blue-500/60 dark:border-neutral-800/70 ${
                          isConfigured
                            ? 'bg-blue-50/60 dark:bg-blue-950/20'
                            : 'hover:bg-neutral-50 dark:hover:bg-neutral-800/50'
                        }`}
                        onClick={browse}
                        onKeyDown={(event) => {
                          if (event.key === 'Enter' || event.key === ' ') {
                            event.preventDefault();
                            browse();
                          }
                        }}
                      >
                        <td className="px-3">
                          <div className="flex min-w-0 items-center gap-2">
                            <Database
                              aria-hidden="true"
                              className={`shrink-0 ${isConfigured ? 'text-blue-500 dark:text-blue-400' : 'text-neutral-400'}`}
                              size={14}
                            />
                            <span className="truncate font-mono font-medium text-neutral-900 group-hover:text-blue-600 group-hover:underline dark:text-neutral-100 dark:group-hover:text-blue-400">
                              {bucket.bucket}
                            </span>
                          </div>
                        </td>
                        {/* 所属地域：接口返回什么就显示什么（本地化地域名 + code），没有就留空。 */}
                        <td className="px-3 text-neutral-600 dark:text-neutral-300">
                          {region ? (
                            <span className="block truncate">
                              {providerRegionLabel(t, provider.providerKind, {
                                label: region,
                                value: region,
                              })}
                            </span>
                          ) : (
                            <span className="text-neutral-400">—</span>
                          )}
                        </td>
                        {/* 用途：每行都有值——这个桶是当前配置的写入目标，还是只能浏览。 */}
                        <td className="px-3">
                          {isConfigured ? (
                            <span
                              className={`${BADGE_BASE_CLASS} shrink-0 bg-blue-100 text-blue-700 dark:bg-blue-900/40 dark:text-blue-300`}
                            >
                              {t('bucketsRoleWriteTarget')}
                            </span>
                          ) : (
                            <span className="text-neutral-500 dark:text-neutral-400">
                              {t('bucketsRoleBrowseOnly')}
                            </span>
                          )}
                        </td>
                        {/* 创建时间按宿主语言格式化：服务层只给 ISO，格式化只在这一层发生。 */}
                        <td className="hidden px-3 tabular-nums text-neutral-500 dark:text-neutral-400 sm:table-cell">
                          {bucket.creationDateIso ? (
                            <time dateTime={bucket.creationDateIso}>
                              {formatDriveDate(bucket.creationDateIso, language)}
                            </time>
                          ) : (
                            '—'
                          )}
                        </td>
                        <td className="px-3">
                          <div className="flex justify-end">
                            <button
                              type="button"
                              title={t('bucketsBrowseFiles')}
                              onClick={(event) => {
                                // 行本身就是打开动作：按钮只负责让动作看得见，不再触发第二次。
                                event.stopPropagation();
                                browse();
                              }}
                              className="inline-flex h-8 items-center gap-1.5 rounded-md border border-neutral-300 bg-white px-2.5 text-xs font-medium text-neutral-700 shadow-sm transition-colors hover:bg-neutral-50 dark:border-neutral-600 dark:bg-neutral-800 dark:text-neutral-200 dark:hover:bg-neutral-700"
                            >
                              <FolderOpen aria-hidden="true" size={13} />
                              {t('bucketsBrowseFiles')}
                            </button>
                          </div>
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          ) : !loading && !error ? (
            <p className="mt-3 flex items-center gap-1.5 text-xs text-neutral-400">
              <CheckCircle2 aria-hidden="true" size={13} />
              {normalizedQuery ? t('bucketsNoMatch') : t('bucketsListEmpty')}
            </p>
          ) : null}
        </div>
      </section>

      <ConfirmDialog
        confirmLabel={t('deleteBucket')}
        description={t('deleteBucketConfirm', { bucket: provider.bucket })}
        onConfirm={() => {
          void deleteBucket();
        }}
        onOpenChange={(next) => {
          if (!next) {
            setPendingDelete(false);
          }
        }}
        open={pendingDelete}
        title={t('deleteBucket')}
        tone="danger"
      />
    </>
  );
}
