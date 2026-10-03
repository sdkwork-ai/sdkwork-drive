import React, { useCallback, useEffect, useMemo, useState } from 'react';
import { CircleAlert, LoaderCircle, Plus, RefreshCw, Settings2 } from 'lucide-react';
import type { DriveAdminStorageSdkClient } from 'sdkwork-drive-pc-admin-core';
import { isDriveRequestCancellationError, type SessionSnapshot } from 'sdkwork-drive-pc-core';
import {
  BADGE_BASE_CLASS,
  GHOST_BUTTON_CLASS,
  SECONDARY_BUTTON_CLASS,
  StorageProviderNavList,
  createStorageProviderAdminService,
  getProviderKindMeta,
  providerDisplayName,
  useProviderOptions,
  useTranslation,
} from 'sdkwork-drive-pc-admin-storage-providers';
import type {
  StorageProviderAdminService,
  StorageProviderView,
} from 'sdkwork-drive-pc-admin-storage-providers';
import { BucketListPanel } from '../components/BucketListPanel';
import { BucketObjectManagerDialog } from '../components/BucketObjectManagerDialog';
import { buildFilePreviewLabels } from '../utils/filePreviewDictionary';

/** 宿主包需要它来包装页面（例如注入路由跳转），所以类型必须从公开出口可见。 */
export interface StorageBucketsAdminPageProps {
  adminStorageSdkClient: DriveAdminStorageSdkClient;
  getSession: () => SessionSnapshot;
  /**
   * Quick-configure entry points. The bucket page is where operators notice a
   * missing or wrong configuration, so it must be able to hand them off to the
   * provider pages. Both are optional: a host that renders this page without
   * the provider surfaces mounted simply does not get the shortcuts.
   */
  onConfigureProviders?: () => void;
  onCreateProvider?: () => void;
}

interface BrowseTarget {
  bucket: string;
  /**
   * 桶所在地域，取自桶清单那一行。
   *
   * 清单是账号级读取、跨地域，所以点开的桶未必在服务商配置写的地域里；带上它，
   * 服务端才按厂商自己的端点规范落到这个桶所在地域去读写文件。
   */
  region?: string;
  provider: StorageProviderView;
}

/**
 * 账号名解析的上限与页大小。
 *
 * 账号中心默认一页 20 条、最多 200 条；跟页上限取 5 页（1000 个账号）足够覆盖真实规模，
 * 又不会为了一个徽标无限翻页。超出时徽标缺失，但桶列表不受影响。
 */
const ACCOUNT_LOOKUP_PAGE_SIZE = 200;
const ACCOUNT_LOOKUP_MAX_PAGES = 5;

/**
 * 存储桶管理页。
 *
 * 左栏是服务商配置，右栏是该账号下可见的存储桶；点开任意一个桶都会打开网盘式的
 * 文件管理弹窗（左侧内容分类、右侧文件列表，支持对象的新增 / 读取 / 更新 / 删除）。
 * 页面本身不构造 SDK 客户端：`createStorageProviderAdminService` 才是唯一持有
 * admin storage 传输的地方，桶弹窗复用它并只额外带上被管理的桶。
 */
export function StorageBucketsAdminPage({
  adminStorageSdkClient,
  getSession,
  onConfigureProviders,
  onCreateProvider,
}: StorageBucketsAdminPageProps) {
  const { t } = useTranslation();
  const service = useMemo<StorageProviderAdminService>(
    () => createStorageProviderAdminService({ adminStorageSdkClient, getSession }),
    [adminStorageSdkClient, getSession],
  );
  /**
   * 左栏就是服务商开关，读的是选项集而不是某一页表格：一个 20 行的窗口会让刚初始化
   * 的存储平面（25 个配置）把最后几个——包括默认绑定的那个——藏在没有翻页控件的
   * 列表之外。
   */
  const providerOptions = useProviderOptions(service, { status: 'active' });
  const providers = providerOptions.items;
  const [selectedProviderId, setSelectedProviderId] = useState('');
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState(false);
  const [browseTarget, setBrowseTarget] = useState<BrowseTarget | null>(null);

  const load = (signal?: AbortSignal) => {
    setLoading(true);
    setLoadError(false);
    providerOptions
      .reload(signal)
      .then((items) => {
        setSelectedProviderId((current) =>
          items.some((provider) => provider.id === current) ? current : (items[0]?.id ?? ''),
        );
      })
      .catch((err) => {
        if (!isDriveRequestCancellationError(err)) setLoadError(true);
      })
      .finally(() => setLoading(false));
  };

  useEffect(() => {
    const c = new AbortController();
    load(c.signal);
    return () => c.abort();
  }, [service]);

  const selectedProvider = providers.find((provider) => provider.id === selectedProviderId);
  const selectedMeta = selectedProvider
    ? getProviderKindMeta(selectedProvider.providerKind)
    : undefined;

  /**
   * 服务商配置绑定的账号名。
   *
   * 一套配置的端点、区域与凭证都来自它绑定的账号；同一个端点下挂着多个账号时，
   * 只显示端点是分不清"我现在看的是哪个账号的桶"。账号列表是辅助信息：取不到就
   * 不显示徽标，绝不让它阻塞或打断桶浏览器。
   *
   * 账号中心是游标分页的，所以这里跟页取（有上限），而不是只读第一页——只读第一页
   * 会让第 201 个之后的账号徽标静默消失，那种"有时有、有时没有"的表现最难排查。
   */
  const [accountNames, setAccountNames] = useState<ReadonlyMap<string, string>>(new Map());
  useEffect(() => {
    const controller = new AbortController();
    const collected = new Map<string, string>();
    const loadPage = async (pageToken?: string, page = 0): Promise<void> => {
      if (page >= ACCOUNT_LOOKUP_MAX_PAGES) {
        return;
      }
      const result = await service.listProviderAccountsPage({
        pageSize: ACCOUNT_LOOKUP_PAGE_SIZE,
        ...(pageToken ? { pageToken } : {}),
        signal: controller.signal,
      });
      for (const account of result.items) {
        collected.set(account.id, account.displayName);
      }
      setAccountNames(new Map(collected));
      if (result.nextPageToken) {
        await loadPage(result.nextPageToken, page + 1);
      }
    };
    loadPage().catch(() => {
      // 账号名只是增强信息，失败时静默降级（已收集到的部分仍然显示）。
    });
    return () => controller.abort();
  }, [service]);

  const selectedAccountName = selectedProvider?.providerAccountId
    ? accountNames.get(selectedProvider.providerAccountId)
    : undefined;

  const closeBrowser = useCallback(() => setBrowseTarget(null), []);
  // 预览文案来自本页字典；预览包对缺省字段自带英文兜底，所以这里不需要全量翻译。
  const previewLabels = useMemo(() => buildFilePreviewLabels(t), [t]);

  return (
    <main className="flex h-full flex-1 flex-col overflow-hidden bg-neutral-50 text-neutral-900 dark:bg-neutral-950 dark:text-neutral-100">
      <header
        aria-label={t('bucketsPageTitle')}
        className="flex shrink-0 flex-wrap items-center gap-2 border-b border-neutral-200 bg-white px-4 py-2.5 dark:border-neutral-800 dark:bg-neutral-900 sm:px-6"
      >
        <h1 className="shrink-0 text-sm font-semibold text-neutral-900 dark:text-neutral-100">
          {t('bucketsPageTitle')}
        </h1>
        <span
          className={`${BADGE_BASE_CLASS} shrink-0 bg-neutral-100 text-neutral-600 dark:bg-neutral-800 dark:text-neutral-300`}
        >
          {t('bucketsSummary', { count: providers.length })}
        </span>

        <span className="flex-1" />

        <div className="flex shrink-0 items-center gap-2">
          {onConfigureProviders ? (
            <button
              type="button"
              className={GHOST_BUTTON_CLASS}
              title={t('bucketsQuickConfigureHint')}
              onClick={onConfigureProviders}
            >
              <Settings2 aria-hidden="true" size={15} />
              <span className="hidden sm:inline">{t('bucketsQuickConfigure')}</span>
            </button>
          ) : null}
          {onCreateProvider ? (
            <button
              type="button"
              className={SECONDARY_BUTTON_CLASS}
              title={t('bucketsQuickCreateHint')}
              onClick={onCreateProvider}
            >
              <Plus aria-hidden="true" size={15} />
              <span className="hidden sm:inline">{t('bucketsQuickCreate')}</span>
            </button>
          ) : null}
          <span className="mx-0.5 hidden h-5 w-px shrink-0 bg-neutral-200 dark:bg-neutral-700 sm:block" />
          <button
            type="button"
            className={GHOST_BUTTON_CLASS}
            aria-label={t('refresh')}
            title={t('refresh')}
            disabled={loading}
            onClick={() => load()}
          >
            <RefreshCw aria-hidden="true" className={loading ? 'animate-spin' : undefined} size={15} />
          </button>
        </div>
      </header>

      {loadError || loading || providers.length === 0 ? (
        <div className="flex-1 overflow-auto p-4 sm:p-6">
          {loadError ? (
            <div className="flex items-start gap-3 rounded-lg border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-800 dark:border-red-900 dark:bg-red-950/30 dark:text-red-200">
              <CircleAlert aria-hidden="true" className="mt-0.5 shrink-0" size={16} />
              <span className="flex-1">{t('bucketsNoticeLoadFailed')}</span>
              <button type="button" className={SECONDARY_BUTTON_CLASS} onClick={() => load()}>
                {t('overviewRetry')}
              </button>
            </div>
          ) : loading ? (
            <div className="flex min-h-[280px] items-center justify-center rounded-lg border border-neutral-200 bg-white dark:border-neutral-800 dark:bg-neutral-900">
              <div className="flex items-center gap-3 text-sm text-neutral-500">
                <LoaderCircle aria-hidden="true" className="animate-spin" size={19} />
                {t('bucketsLoading')}
              </div>
            </div>
          ) : (
            <div className="flex flex-col items-start gap-4 rounded-lg border border-amber-200 bg-amber-50 px-5 py-6 text-sm text-amber-800 dark:border-amber-900 dark:bg-amber-950/30 dark:text-amber-200">
              <div className="flex items-start gap-3">
                <CircleAlert aria-hidden="true" className="mt-0.5 shrink-0" size={16} />
                <div>
                  <p className="font-medium">{t('bucketsNoActiveProvidersTitle')}</p>
                  <p className="mt-1 text-xs text-amber-700 dark:text-amber-300">
                    {t('bucketsNoActiveProviders')}
                  </p>
                </div>
              </div>
              <div className="flex flex-wrap items-center gap-2 pl-7">
                {onCreateProvider ? (
                  <button type="button" className={SECONDARY_BUTTON_CLASS} onClick={onCreateProvider}>
                    <Plus aria-hidden="true" size={15} />
                    {t('bucketsQuickCreate')}
                  </button>
                ) : null}
                {onConfigureProviders ? (
                  <button type="button" className={GHOST_BUTTON_CLASS} onClick={onConfigureProviders}>
                    <Settings2 aria-hidden="true" size={15} />
                    {t('bucketsQuickConfigure')}
                  </button>
                ) : null}
                <button type="button" className={GHOST_BUTTON_CLASS} onClick={() => load()}>
                  <RefreshCw aria-hidden="true" size={15} />
                  {t('refresh')}
                </button>
              </div>
            </div>
          )}
        </div>
      ) : (
        <div className="flex min-h-0 flex-1 flex-col lg:flex-row">
          <StorageProviderNavList
            providers={providers}
            selectedProviderId={selectedProviderId}
            onSelect={setSelectedProviderId}
            hasMore={providerOptions.hasMore}
            loadingMore={providerOptions.loading}
            onLoadMore={() => void providerOptions.loadMore()}
          />

          <section className="flex min-h-0 flex-1 flex-col lg:overflow-y-auto">
            {selectedProvider ? (
              <>
                {/* 身份条：下面的桶与文件属于哪套配置，滚动时始终可见。 */}
                <div className="sticky top-0 z-10 shrink-0 border-b border-neutral-200 bg-white/95 px-4 py-2 backdrop-blur dark:border-neutral-800 dark:bg-neutral-900/95 sm:px-6">
                  <div className="flex min-w-0 flex-wrap items-center gap-x-3 gap-y-1">
                    <span
                      title={selectedProvider.providerKind}
                      className={`inline-flex shrink-0 items-center rounded-md px-1.5 py-0.5 text-[10px] font-bold ${selectedMeta?.bgClass ?? ''} ${selectedMeta?.textClass ?? ''}`}
                    >
                      {selectedMeta?.icon}
                    </span>
                    <h2 className="shrink-0 text-sm font-semibold">
                      {providerDisplayName(t, selectedProvider)}
                    </h2>
                    <span
                      className={`${BADGE_BASE_CLASS} shrink-0 ${
                        selectedProvider.credentialConfigured
                          ? 'bg-emerald-50 text-emerald-700 dark:bg-emerald-950/40 dark:text-emerald-300'
                          : 'bg-amber-50 text-amber-700 dark:bg-amber-950/40 dark:text-amber-300'
                      }`}
                    >
                      <span
                        className={`h-1.5 w-1.5 rounded-full ${
                          selectedProvider.credentialConfigured ? 'bg-emerald-500' : 'bg-amber-500'
                        }`}
                      />
                      {selectedProvider.credentialConfigured
                        ? t('credentialSet')
                        : t('credentialMissing')}
                    </span>
                    <span className="hidden h-4 w-px shrink-0 bg-neutral-200 dark:bg-neutral-700 sm:block" />
                    {/* 桶来自哪个账号：同一端点下多个账号时，这是唯一能区分它们的标识。 */}
                    {selectedAccountName ? (
                      <>
                        <span
                          className={`${BADGE_BASE_CLASS} shrink-0 bg-violet-50 text-violet-700 dark:bg-violet-950/40 dark:text-violet-300`}
                          title={`${t('bucketsProviderAccount')}: ${selectedAccountName}`}
                        >
                          {t('bucketsProviderAccount')} · {selectedAccountName}
                        </span>
                        <span className="hidden h-4 w-px shrink-0 bg-neutral-200 dark:bg-neutral-700 sm:block" />
                      </>
                    ) : null}
                    <span className="min-w-0 truncate font-mono text-[11px] text-neutral-500 dark:text-neutral-400">
                      {selectedProvider.endpointUrl}
                    </span>
                    {selectedProvider.region ? (
                      <span className="shrink-0 text-[11px] text-neutral-500 dark:text-neutral-400">
                        · {selectedProvider.region}
                      </span>
                    ) : null}
                  </div>
                </div>

                <div className="grid content-start gap-4 p-4 sm:p-6">
                  <BucketListPanel
                    key={`buckets-${selectedProvider.id}`}
                    onBrowse={(bucket, region) =>
                      setBrowseTarget({ bucket, region, provider: selectedProvider })
                    }
                    provider={selectedProvider}
                    service={service}
                  />
                </div>
              </>
            ) : null}
          </section>
        </div>
      )}

      {browseTarget ? (
        <BucketObjectManagerDialog
          // 换桶要挂载一个全新的浏览器：前缀、分页游标与分类筛选都属于上一个桶。
          key={`${browseTarget.provider.id}-${browseTarget.bucket}`}
          accountName={
            browseTarget.provider.providerAccountId
              ? accountNames.get(browseTarget.provider.providerAccountId)
              : undefined
          }
          bucket={browseTarget.bucket}
          labels={previewLabels}
          onOpenChange={(open) => {
            if (!open) {
              closeBrowser();
            }
          }}
          open
          provider={browseTarget.provider}
          region={browseTarget.region}
          service={service}
        />
      ) : null}
    </main>
  );
}
