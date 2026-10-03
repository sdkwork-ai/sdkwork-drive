import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import {
  ArrowDown,
  ArrowLeft,
  ArrowUp,
  ArrowUpDown,
  ChevronLeft,
  ChevronRight,
  CircleAlert,
  Download,
  Eye,
  FolderPlus,
  LoaderCircle,
  PencilLine,
  RefreshCw,
  Search,
  Trash2,
  Upload,
  X,
} from 'lucide-react';
import {
  ConfirmDialog,
  Modal,
  ModalBody,
  ModalContent,
  ModalFooter,
  ModalHeader,
  ModalTitle,
} from '@sdkwork/ui-pc-react';
import {
  BADGE_BASE_CLASS,
  GHOST_BUTTON_CLASS,
  INPUT_CLASS,
  PRIMARY_BUTTON_CLASS,
  SECONDARY_BUTTON_CLASS,
  fileNameOf,
  formatMutationError,
  isPayloadTooLargeError,
  parentPrefixOf,
  providerDisplayName,
  relativeObjectPath,
  useTranslation,
} from 'sdkwork-drive-pc-admin-storage-providers';
import type {
  StorageProviderAdminService,
  StorageProviderObjectView,
  StorageProviderView,
} from 'sdkwork-drive-pc-admin-storage-providers';
import {
  FileKindIcon,
  formatByteSize,
  resolveFilePreviewKind,
  type FilePreviewLabelsInput,
  type FilePreviewNavigation,
} from 'sdkwork-drive-pc-file-preview';
import {
  countBucketObjectsByCategory,
  matchesBucketObjectCategory,
  resolveBucketObjectCategory,
  type BucketObjectCategoryFilter,
} from '../utils/bucketObjectCategory';
import {
  MAX_OBJECT_CONTENT_BYTES,
  downloadObjectBytes,
  objectContentRequestBytes,
} from '../utils/objectContent';
import {
  MULTIPART_THRESHOLD_BYTES,
  MultipartUploadError,
  planUploadParts,
  uploadObjectInParts,
} from '../utils/multipartUpload';
import { formatModifiedTime, modifiedTimeIso } from '../utils/formatModifiedTime';
import {
  DEFAULT_BUCKET_OBJECT_SORT,
  nextBucketObjectSort,
  sortBucketObjects,
  type BucketObjectSort,
  type BucketObjectSortKey,
} from '../utils/sortBucketObjects';
import { BucketCategoryRail } from './BucketCategoryRail';
import { BucketObjectPreviewDialog } from './BucketObjectPreviewDialog';

/** 目录占位对象（key 以 `/` 结尾）在列表里的稳定标识。 */
function isDirectoryPlaceholder(objectKey: string): boolean {
  return objectKey.endsWith('/');
}

type PromptKind = 'newFolder' | 'rename';

/**
 * 把上传失败翻译成"下一步做什么"。
 *
 * 分片直传的失败几乎都落在三种可判定的原因上：桶没放行跨域、桶的跨域没暴露 ETag、单片反复
 * 失败。把厂商/浏览器的原文摆给用户没有意义（`Failed to fetch` 就是一句话），必须说清是哪
 * 一步、以及去哪里改。
 */
function describeUploadFailure(
  error: unknown,
  t: (key: string, params?: Record<string, string | number>) => string,
  sizes: { limit: string; request: string },
): string {
  if (error instanceof MultipartUploadError) {
    switch (error.kind) {
      case 'cors':
        return t('bucketsBrowserUploadCorsBlocked');
      case 'etag-missing':
        return t('bucketsBrowserUploadEtagMissing');
      case 'too-many-parts':
        return t('bucketsBrowserUploadTooLarge');
      case 'aborted':
        return t('bucketsBrowserUploadAborted');
      case 'complete-failed':
        return t('bucketsBrowserUploadCompleteFailed');
      case 'part-failed':
      default:
        return t('bucketsBrowserUploadPartFailed', {
          part: error.partNumber ?? 0,
        });
    }
  }
  /*
   * 413 要单独解释：内容接口把对象字节放在 JSON 里以 base64 传输，请求体约为文件的 1.37 倍；
   * 网关的请求体上限比业务上限更早生效，超限时只回一句 `Payload too large`。照抄这句话会让
   * 用户以为对象本身超了 8 MiB，而文件可能只有 7 MB。
   */
  if (isPayloadTooLargeError(error)) {
    return t('bucketsBrowserUploadPayloadTooLarge', {
      limit: sizes.limit,
      request: sizes.request,
    });
  }
  return formatMutationError(error, t('uploadError'));
}
/**
 * 表头排序按钮：名称 / 大小 / 修改时间三列可排。
 *
 * 列头是按钮而不是整格可点，键盘 Tab 能走到、Enter/Space 能触发；方向用箭头图标与
 * `aria-sort` 同时表达（颜色不是唯一信号）。数字与时间列在窄屏隐藏，但排序状态仍由
 * `aria-sort` 保留在 DOM 上，屏幕阅读器不会漏掉"当前按哪个字段排"。
 */
function SortHeaderButton({
  active,
  direction,
  label,
  onClick,
  sortByLabel,
}: {
  active: boolean;
  direction: BucketObjectSort['direction'];
  label: string;
  onClick: () => void;
  sortByLabel: string;
}) {
  return (
    <button
      type="button"
      className={`-mx-1 inline-flex max-w-full items-center gap-1 rounded px-1 py-0.5 text-left uppercase tracking-wide transition-colors hover:text-neutral-700 dark:hover:text-neutral-200 ${
        active ? 'text-neutral-700 dark:text-neutral-100' : ''
      }`}
      aria-label={sortByLabel}
      title={sortByLabel}
      onClick={onClick}
    >
      <span className="truncate">{label}</span>
      {active ? (
        direction === 'asc' ? (
          <ArrowUp aria-hidden="true" className="shrink-0" size={12} />
        ) : (
          <ArrowDown aria-hidden="true" className="shrink-0" size={12} />
        )
      ) : (
        // 未激活的列给一个淡箭头，暗示"这一列可以点"。
        <ArrowUpDown
          aria-hidden="true"
          className="shrink-0 text-neutral-300 dark:text-neutral-600"
          size={12}
        />
      )}
    </button>
  );
}

/**
 * 每页条数。
 *
 * 对象列表接口的 `page_size` 上限是 200（PAGINATION_SPEC 的分页约束）。默认 100：
 * 一屏大约 12 行，100 条足够覆盖"翻着找"的场景，又不会让单次列表请求变重。
 */
const OBJECT_PAGE_SIZE_OPTIONS = [50, 100, 200] as const;
const DEFAULT_OBJECT_PAGE_SIZE = 100;

/** 一页的加载结果：内容 + 下一页游标 + 是否还有更多。 */
interface LoadedPage {
  hasMore: boolean;
  items: StorageProviderObjectView[];
  nextToken: string | null;
}

/**
 * 一次"下载所选"最多触发多少个下载。
 *
 * 内容接口一次只读一个对象，浏览器对连续的程序化下载也会拦截；给一个明确上限，并且
 * 在超出时告诉用户去厂商控制台批量取，比点一下弹出一堆被拦下的下载要好。
 */
const MAX_BULK_DOWNLOAD_COUNT = 20;

interface ObjectPrompt {
  kind: PromptKind;
  objectKey?: string;
  initialValue: string;
}

export interface BucketObjectManagerDialogProps {
  /**
   * 该配置绑定的服务商账号名（可选）。
   *
   * 桶来自账号的凭证，同一个端点下挂多个账号时，弹窗头部不写账号就分不清在看谁的桶。
   * 名字由宿主解析（账号中心接口），拿不到就不显示。
   */
  accountName?: string;
  /** 被管理的存储桶，随每个请求发送，而不是读服务商配置里设定的桶。 */
  bucket: string;
  /** 预览文案：只需给出翻译过的字段，其余由预览包的英文兜底补齐。 */
  labels?: FilePreviewLabelsInput;
  onOpenChange: (open: boolean) => void;
  open: boolean;
  /** 提供凭证的服务商配置。 */
  provider: StorageProviderView;
  /**
   * 桶所在地域（桶清单那一行的 `region`）。
   *
   * 桶清单是账号级读取、跨地域，所以弹窗里的每一次对象读写都要按这个桶自己的地域找
   * 端点：缺省时服务端仍用配置里写的地域，跨地域的桶就会打不开。
   */
  region?: string;
  service: StorageProviderAdminService;
}

/**
 * 网盘式存储桶文件管理器。
 *
 * 版面按「内容优先」组织：头部只留一行身份信息（标题 + 桶 + 端点），底部不再有
 * 页脚——关闭入口是对话框自身的关闭按钮与 Esc，省下的整条高度都留给文件列表。
 * 左栏是内容分类与存储桶信息，右栏是当前目录列表；点任意文件即在同尺寸的预览
 * 对话框里打开（预览组件来自 `sdkwork-drive-pc-file-preview`），点文件夹进入下一层。
 */
export function BucketObjectManagerDialog({
  accountName,
  bucket,
  labels,
  onOpenChange,
  open,
  provider,
  region,
  service,
}: BucketObjectManagerDialogProps) {
  const { t, language } = useTranslation();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [currentPrefix, setCurrentPrefix] = useState('');
  /**
   * 实际生效的地域。
   *
   * 初始值是打开这个桶时带来的地域（桶清单里的 region），首次列目录成功后会被改成
   * "真的能访问这个桶的那个地域"；后续所有读写、下载都用它，避免列表通了、删除又失败。
   */
  const [activeRegion, setActiveRegion] = useState<string | undefined>(region);
  /**
   * 已访问过的页，按顺序保存。
   *
   * 列表接口是游标分页（只有 cursor + page_size，没有 offset、没有总数），所以"第 N 页"
   * 只能靠走过一遍得到。留住在内存里，上一页就是瞬时的，也才敢让用户来回翻。
   */
  const [pages, setPages] = useState<readonly LoadedPage[]>([]);
  const [pageIndex, setPageIndex] = useState(0);
  const [pageSize, setPageSize] = useState(DEFAULT_OBJECT_PAGE_SIZE);
  const [category, setCategory] = useState<BucketObjectCategoryFilter>('all');
  /** 排序：缺省按名称升序；大小/时间列点击后先给"从大到小 / 从新到旧"。 */
  const [sort, setSort] = useState<BucketObjectSort>(DEFAULT_BUCKET_OBJECT_SORT);
  /** 「加载全部」进行中：用于禁用按钮并显示进度。 */
  const [loadingAll, setLoadingAll] = useState(false);
  const [query, setQuery] = useState('');
  const [deleteTarget, setDeleteTarget] = useState<StorageProviderObjectView | null>(null);
  /** 待确认的覆盖写：重命名或上传会替换同名对象，必须先问一句。 */
  const [overwrite, setOverwrite] = useState<
    | { destination: string; kind: 'rename'; objectKey: string }
    | { files: readonly File[]; kind: 'upload' }
    | null
  >(null);
  const [previewTarget, setPreviewTarget] = useState<StorageProviderObjectView | null>(null);
  const [prompt, setPrompt] = useState<ObjectPrompt | null>(null);
  const [promptValue, setPromptValue] = useState('');
  const [promptBusy, setPromptBusy] = useState(false);
  const [uploading, setUploading] = useState(false);
  /**
   * 多选：按对象 key 记录，而不是按下标。
   *
   * 列表会因刷新重排、因分页追加，按下标记会选中错的行；key 在桶内唯一且稳定。
   */
  const [selectedKeys, setSelectedKeys] = useState<ReadonlySet<string>>(() => new Set());
  const [selectionToDelete, setSelectionToDelete] = useState(false);
  /** 拖拽进入：只用来画高亮，不参与业务判断。 */
  const [dragActive, setDragActive] = useState(false);
  /** 多选上传的进度：`total` 为 0 表示没有进行中的批次。 */
  const [uploadTotal, setUploadTotal] = useState(0);
  const [uploadDone, setUploadDone] = useState(0);
  /** 当前文件的分片直传进度（0-100）；走单次通道时保持 0。 */
  const [uploadPercent, setUploadPercent] = useState(0);
  /**
   * 进行中的分片直传的取消句柄。
   *
   * 关闭弹窗必须中止它：厂商的分片在完成前一直计费，用户关掉界面却把分片留在桶里，是最
   * 容易被忽略的一笔成本。
   */
  const uploadAbortRef = useRef<AbortController | null>(null);
  const [downloadingKey, setDownloadingKey] = useState<string | null>(null);
  const fileInputRef = useRef<HTMLInputElement | null>(null);
  /** 预览里是否保存过内容：决定关闭预览后要不要重取列表。 */
  const savedInPreviewRef = useRef(false);
  /** 请求序号：分页与目录切换并发时丢弃过期响应。 */
  const loadSeqRef = useRef(0);
  /** 已拉取过的目标（服务商 + 桶），用于打开后只自动加载一次。 */
  const loadedTargetRef = useRef<string | null>(null);
  /** 页数组的镜像：loadObjects 需要读"上一页的游标"，但不该因此重建回调。 */
  const pagesRef = useRef<readonly LoadedPage[]>(pages);
  pagesRef.current = pages;
  /** 行区域：换页后回到顶部，否则用户会以为新一页只有半屏。 */
  const rowsRef = useRef<HTMLDivElement | null>(null);

  const loadObjects = useCallback(
    async (
      prefix: string,
      index = 0,
      options?: {
        /**
         * 变更后的刷新：只更新内容，不把视图拉回这个前缀。
         *
         * 上传/删除期间的列表刷新是"补齐数据"，不是"导航"。若它也写 `currentPrefix`，
         * 用户在等待期间点进子目录就会被一次迟到的刷新拽回原处。
         */
        keepView?: boolean;
        /** 换目录 / 换每页条数：丢掉已经走过的页。 */
        reset?: boolean;
        /**
         * 保留当前错误提示。
         *
         * 变更失败后的刷新是"补齐真实状态"，不是"用户主动重试"：刷新顺手把错误清掉，
         * 用户就永远看不到失败原因（上传 413、删除被拒都发生过）。只有用户动作
         * （点刷新、翻页、换目录）才清空错误。
         */
        preserveError?: boolean;
        /**
         * 本次请求的每页条数。
         *
         * 改每页条数时必须在同一次调用里带上新值：`loadObjects` 是当前渲染的回调，
         * 它捕获的还是旧的 `pageSize`，光 `setPageSize` 之后再调用会按旧值发请求。
         */
        pageSize?: number;
      },
    ) => {
      const seq = ++loadSeqRef.current;
      const requestedPageSize = options?.pageSize ?? pageSize;
      setLoading(true);
      if (!options?.preserveError) {
        setError(null);
      }
      /*
       * 地域先试一遍候选列表：桶所在的地域 → 打开时带来的地域 → 服务商配置的地域。
       *
       * 桶清单是跨地域的（一个账号下的桶可能散在多个地域），而 S3/COS 这类对象存储对
       * "用错地域的端点访问某个桶"的回答是 `NoSuchBucket`（"The specified bucket does not
       * exist."），不是任何提示地域错误的文案——只看这句话会以为桶不存在。清单里的
       * region 字段并不总是可靠（可能缺失，也可能就是配置地域），所以这里按顺序试到
       * 成功为止，并把生效的那个记住给后续的读写请求用。列目录无副作用，重试是安全的。
       */
      const candidates = [activeRegion, region, provider.region].filter(
        (value, position, all): value is string =>
          typeof value === 'string' && value !== '' && all.indexOf(value) === position,
      );
      const attempts: (string | undefined)[] = candidates.length > 0 ? candidates : [undefined];
      let lastError: unknown;
      try {
        /*
         * 游标分页：第 N 页的游标就是第 N-1 页返回的 nextToken。
         *
         * 对象列表接口只有 cursor + page_size，没有 offset、也没有总数，所以"跳到第 17 页"
         * 在协议层就不存在。这里保留已经走过的页（`pages`），于是"上一页"是瞬时且准确的，
         * "下一页"用上一页的游标继续向前——这正是游标分页能给到的最好的分页体验。
         */
        const token = index > 0 ? pagesRef.current[index - 1]?.nextToken ?? null : null;
        let result: Awaited<ReturnType<StorageProviderAdminService['listObjects']>> | undefined;
        let usedRegion: string | undefined;
        for (const candidate of attempts) {
          try {
            result = await service.listObjects(provider.id, {
              bucket,
              pageSize: requestedPageSize,
              prefix,
              pageToken: token ?? undefined,
              // 桶清单跨地域：列这个桶的文件要按它自己的地域找端点。
              ...(candidate === undefined ? {} : { region: candidate }),
            });
            usedRegion = candidate;
            break;
          } catch (attemptError) {
            lastError = attemptError;
          }
        }
        if (result === undefined) {
          throw lastError ?? new Error('listObjects failed');
        }
        if (seq !== loadSeqRef.current) {
          return;
        }
        if (usedRegion !== undefined && usedRegion !== activeRegion) {
          setActiveRegion(usedRegion);
        }
        /**
         * 目录占位对象（key 恰好等于当前前缀）不展示。
         *
         * 后端把「文件夹」实现成同名前缀的空对象，列出该前缀时会把它自己带回来；不过滤
         * 的话，进入刚建好的空目录会看到一行没有名字、0 B 的"文件"，而且它能被重命名 ——
         * 那会把 `invoices/` 复制成 `invoices/xxx` 再删掉占位对象，目录就此消失。
         */
        const items = result.items.filter((item) => item.key !== prefix);
        const page: LoadedPage = {
          hasMore: result.hasMore,
          items,
          nextToken: result.nextPageToken || null,
        };
        setPages((previous) => {
          const base = options?.reset ? [] : [...previous];
          base[index] = page;
          // 丢掉被重新加载那一页之后的所有页：它们的游标可能已经失效。
          return base.slice(0, index + 1);
        });
        setPageIndex(index);
        if (!options?.keepView) {
          setCurrentPrefix(prefix);
        }
      } catch (err) {
        if (seq !== loadSeqRef.current) {
          return;
        }
        /*
         * 报错必须带上"用哪个桶、试过哪些地域"。
         *
         * 厂商对"地域用错"的回答就是 `NoSuchBucket`，光看这句话会去查桶是否存在，
         * 而真正的原因是端点不对。把上下文摆在同一条消息里，运维一眼能判断。
         */
        setError(
          `${formatMutationError(err, t('errorLoadObjects'))}（${t(
            'bucketsBrowserListErrorContext',
            {
              bucket,
              regions: attempts.filter(Boolean).join(' / ') || t('bucketsBrowserRegionDefault'),
            },
          )}）`,
        );
      } finally {
        if (seq === loadSeqRef.current) {
          setLoading(false);
        }
      }
    },
    [activeRegion, bucket, pageSize, provider.id, provider.region, region, service, t],
  );

  /**
   * 打开即定位到桶根。
   *
   * 守卫按「已拉取的目标」而不是回调标识去重：宿主语言每次重渲染都会换 `t` 的标识，
   * `loadObjects` 随之换新，按标识做依赖会重复请求。关闭时忘掉目标，这样重新打开
   * 同一个桶读到的也不是上一次的陈旧列表。
   */
  useEffect(() => {
    if (!open) {
      loadedTargetRef.current = null;
      // 关闭即中止进行中的分片直传：不中止就会在桶里留下计费的孤儿分片。
      uploadAbortRef.current?.abort();
      uploadAbortRef.current = null;
      return;
    }
    const target = `${provider.id}\u0000${bucket}`;
    if (loadedTargetRef.current === target) {
      return;
    }
    loadedTargetRef.current = target;
    void loadObjects('', 0, { reset: true });
  }, [bucket, loadObjects, open, provider.id]);

  const navigateToFolder = (prefix: string) => {
    setCategory('all');
    setQuery('');
    setPreviewTarget(null);
    // 换目录即丢弃选中：跨目录的选中集在界面上无从表达，留着只会误删。
    clearSelection();
    void loadObjects(prefix, 0, { reset: true });
  };

  /**
   * 翻页。
   *
   * 目标是"已访问过的页"就直接显示（瞬时、无请求）；是下一页才带上游标去取。
   * 换页后把行区域滚回顶部：否则新一页看起来只有半屏内容。
   */
  const goToPage = (index: number) => {
    if (index < 0 || loading) {
      return;
    }
    // 直接写 scrollTop 而不是调 `scrollTo`：后者在部分宿主（含 jsdom）上不存在，
    // 一次未捕获的 TypeError 会让点击处理器在翻页之前就中断。
    if (rowsRef.current) {
      rowsRef.current.scrollTop = 0;
    }
    if (pagesRef.current[index]) {
      setPageIndex(index);
      return;
    }
    if (index === pagesRef.current.length) {
      void loadObjects(currentPrefix, index, { keepView: true });
    }
  };

  const navigateUp = () => {
    // 与点进目录保持同一套语义：换目录就清掉筛选与搜索，否则"上一层"会带着
    // 上一层的过滤条件，看起来像目录里的东西凭空少了。
    setCategory('all');
    setQuery('');
    setPreviewTarget(null);
    clearSelection();
    void loadObjects(parentPrefixOf(currentPrefix.replace(/\/$/, '')), 0, { reset: true });
  };

  const openPrompt = (next: ObjectPrompt) => {
    setPromptValue(next.initialValue);
    setPrompt(next);
  };

  /**
   * 新建/重命名的名字校验。
   *
   * 目录分隔符与 `.`/`..` 必须挡在提交之前：`archive/a.txt` 会把对象搬到另一个前缀，
   * `..` 会拼出指向上层的 key。这里只允许"当前目录里的一个名字"。
   */
  const validateEntryName = (value: string): string | null => {
    if (value === '' || value === '.' || value === '..') {
      return t('bucketsBrowserInvalidName');
    }
    if (value.includes('/') || value.includes('\\')) {
      return t('bucketsBrowserInvalidName');
    }
    return null;
  };

  /** 真正执行重命名；覆盖确认走同一个入口，避免两套逻辑。 */
  const performRename = async (objectKey: string, destination: string) => {
    try {
      await service.renameObject(provider.id, objectKey, destination, { bucket, region: activeRegion });
    } catch (err) {
      // 复制成功但删除失败时两个 key 都会存在：文案必须承认"可能部分完成"，
      // 并且刷新列表让用户看见真实状态，而不是只看一句"重命名失败"。
      setError(formatMutationError(err, t('renameError')));
    } finally {
      void loadObjects(currentPrefix, pageIndex, { keepView: true, preserveError: true });
    }
  };

  const submitPrompt = async () => {
    if (!prompt) {
      return;
    }
    const value = promptValue.trim();
    const invalid = validateEntryName(value);
    if (invalid) {
      setError(invalid);
      return;
    }
    setPromptBusy(true);
    setError(null);
    try {
      if (prompt.kind === 'newFolder') {
        await service.writeObjectContent(
          provider.id,
          `${currentPrefix}${value}/`,
          { content: '' },
          { bucket, region: activeRegion },
        );
      } else if (prompt.objectKey) {
        // 重命名只换当前目录里的名字：目标 key 由当前前缀拼出，而不是用完整 key
        // 覆盖，否则嵌套目录会被拍平。
        const destination = `${currentPrefix}${value}`;
        if (destination === prompt.objectKey) {
          setPrompt(null);
          return;
        }
        // 覆盖保护：目标已存在时先问一句，复制是整对象替换、不可撤销。
        if (pageItems.some((object) => object.key === destination)) {
          setOverwrite({ destination, kind: 'rename', objectKey: prompt.objectKey });
          setPrompt(null);
          return;
        }
        setPrompt(null);
        await performRename(prompt.objectKey, destination);
        return;
      }
      setPrompt(null);
      await loadObjects(currentPrefix, pageIndex, { keepView: true, preserveError: true });
    } catch (err) {
      setError(
        formatMutationError(
          err,
          prompt.kind === 'newFolder' ? t('newFolderError') : t('renameError'),
        ),
      );
    } finally {
      setPromptBusy(false);
    }
  };

  /**
   * 上传：≤ 8 MiB 走单次内容接口（base64），更大的对象走预签名分片直传。
   *
   * 顺序写而不是并发写：并发会同时打开多个大文件读 + 多个请求，浏览器内存与后端连接都
   * 不可控；顺序写还能在第一个失败时立刻停下，避免"传了一半"更难解释的状态。体积校验
   * 放在读文件之前一次性做完，超限的文件根本不会被读进内存。
   *
   * 分片通道的字节直接进厂商（预签名 URL），所以 8 MiB 不再是上传上限；单次接口仍用于小
   * 对象——它更简单，且不依赖桶的跨域配置。
   */
  const uploadFiles = async (
    files: readonly File[],
    options?: { overwriteConfirmed?: boolean },
  ) => {
    if (files.length === 0) {
      return;
    }
    // 单次通道装不下的文件先做一次"这条路走不走得通"的预检：分片数超厂商上限时必须在
    // 开启上传之前拒绝，否则会在桶里留下一个永远完不成的孤儿上传。
    const unusable = files.filter((file) => {
      try {
        planUploadParts(file.size);
        return false;
      } catch {
        return true;
      }
    });
    if (unusable.length > 0) {
      setError(`${t('bucketsBrowserUploadTooLarge')} (${unusable.map((file) => file.name).join(', ')})`);
      if (fileInputRef.current) {
        fileInputRef.current.value = '';
      }
      return;
    }
    // 覆盖保护：上传是整对象替换，同名对象会被静默盖掉。
    const colliding = files.filter((file) =>
      pageItems.some((object) => object.key === `${currentPrefix}${file.name}`),
    );
    if (colliding.length > 0 && !options?.overwriteConfirmed) {
      setOverwrite({ files, kind: 'upload' });
      if (fileInputRef.current) {
        fileInputRef.current.value = '';
      }
      return;
    }

    const controller = new AbortController();
    uploadAbortRef.current = controller;
    setUploading(true);
    setUploadTotal(files.length);
    setUploadDone(0);
    setUploadPercent(0);
    setError(null);
    try {
      for (const [index, file] of files.entries()) {
        const objectKey = `${currentPrefix}${file.name}`;
        if (file.size > MULTIPART_THRESHOLD_BYTES) {
          // 大对象：分片直传。进度按"已传分片/总片数"折算百分比。
          setUploadPercent(0);
          await uploadObjectInParts({
            file,
            objectKey,
            providerId: provider.id,
            service,
            bucket,
            region: activeRegion,
            ...(file.type ? { contentType: file.type } : {}),
            signal: controller.signal,
            onProgress: (progress) => {
              setUploadPercent(
                progress.totalBytes > 0
                  ? Math.floor((progress.uploadedBytes / progress.totalBytes) * 100)
                  : 0,
              );
            },
          });
        } else {
          const base64 = await readFileAsBase64(file);
          await service.writeObjectContent(
            provider.id,
            objectKey,
            {
              content: base64,
              encoding: 'base64',
              ...(file.type ? { contentType: file.type } : {}),
            },
            { bucket, region: activeRegion },
          );
        }
        setUploadDone(index + 1);
        setUploadPercent(0);
      }
    } catch (err) {
      setError(describeUploadFailure(err, t, {
        limit: formatByteSize(MAX_OBJECT_CONTENT_BYTES),
        request: formatByteSize(objectContentRequestBytes(MAX_OBJECT_CONTENT_BYTES)),
      }));
    } finally {
      uploadAbortRef.current = null;
      setUploading(false);
      setUploadTotal(0);
      setUploadDone(0);
      setUploadPercent(0);
      if (fileInputRef.current) {
        fileInputRef.current.value = '';
      }
      // 成功要刷新（新对象入列），失败也要刷新（前几个可能已经写进去了）。
      void loadObjects(currentPrefix, pageIndex, { keepView: true, preserveError: true });
    }
  };

  /** 下载：整块内容读回来后交给浏览器保存（≤ 8 MiB，与读取接口上限一致）。 */
  const downloadObject = useCallback(
    async (object: StorageProviderObjectView) => {
      setDownloadingKey(object.key);
      setError(null);
      try {
        await downloadObjectBytes({ bucket, providerId: provider.id, region: activeRegion, service }, object);
      } catch (err) {
        setError(formatMutationError(err, t('downloadError')));
      } finally {
        setDownloadingKey(null);
      }
    },
    [activeRegion, bucket, provider.id, service, t],
  );

  const deleteObject = useCallback(
    async (objectKey: string) => {
      setDeleteTarget(null);
      setLoading(true);
      setError(null);
      try {
        await service.deleteObject(provider.id, objectKey, { bucket, region: activeRegion });
      } catch (err) {
        setError(formatMutationError(err, t('errorDeleteObject')));
      } finally {
        // 无论成败都重取：删除失败可能已经生效，列表必须显示真实状态。
        // `keepView` 保证这次刷新不会把用户拽回他刚才所在的目录。
        void loadObjects(currentPrefix, pageIndex, { keepView: true, preserveError: true });
      }
    },
    [bucket, currentPrefix, loadObjects, provider.id, service, t],
  );

  /** 当前页的内容；分页之后"列表"就是这一页。 */
  const pageItems = useMemo(() => pages[pageIndex]?.items ?? [], [pages, pageIndex]);
  const hasMore = pages[pageIndex]?.hasMore ?? false;
  /** 已经走过的所有页里的对象：批量下载要能覆盖用户在前面几页勾选的对象。 */
  const loadedObjects = useMemo(
    () => pages.flatMap((page) => page.items ?? []),
    [pages],
  );

  const counts = useMemo(() => countBucketObjectsByCategory(pageItems), [pageItems]);
  const loadedBytes = useMemo(
    () => pageItems.reduce((total, object) => (object.isFolder ? total : total + object.sizeBytes), 0),
    [pageItems],
  );
  const normalizedQuery = query.trim().toLowerCase();
  const visibleObjects = useMemo(
    () =>
      sortBucketObjects(
        pageItems.filter(
          (object) =>
            matchesBucketObjectCategory(object, category)
            && (!normalizedQuery || fileNameOf(object.key).toLowerCase().includes(normalizedQuery)),
        ),
        sort,
        language,
      ),
    [category, language, normalizedQuery, pageItems, sort],
  );
  const breadcrumbSegments = currentPrefix.split('/').filter(Boolean);

  /**
   * 预览序列 = 当前可见的文件（不含目录），顺序与列表一致。
   *
   * 按"当前可见"而不是"整个目录"：用户刚筛成图片，左右切换就该只在图片之间走；
   * 按目录全量算会让切换跳到看不见的对象上，像是列表和预览各说各话。
   */
  const previewableObjects = useMemo(
    () => visibleObjects.filter((object) => !object.isFolder),
    [visibleObjects],
  );
  const previewNavigation = useMemo<FilePreviewNavigation | undefined>(() => {
    if (!previewTarget || previewTarget.isFolder) {
      return undefined;
    }
    const index = previewableObjects.findIndex((object) => object.key === previewTarget.key);
    if (index < 0) {
      // 列表已刷新且这个对象不在了（例如刚被删掉）：不提供切换，免得跳到一个不存在的位置。
      return undefined;
    }
    return {
      index,
      total: previewableObjects.length,
      ...(index > 0 ? { onPrevious: () => setPreviewTarget(previewableObjects[index - 1]) } : {}),
      ...(index < previewableObjects.length - 1
        ? { onNext: () => setPreviewTarget(previewableObjects[index + 1]) }
        : {}),
    };
  }, [previewTarget, previewableObjects]);

  const clearSelection = useCallback(() => setSelectedKeys(new Set()), []);

  const toggleSelection = useCallback((objectKey: string) => {
    setSelectedKeys((current) => {
      const next = new Set(current);
      if (next.has(objectKey)) {
        next.delete(objectKey);
      } else {
        next.add(objectKey);
      }
      return next;
    });
  }, []);

  /**
   * 批量删除：顺序删、逐个计数。
   *
   * 单个 key 的删除接口没有事务语义，所以失败时必须说清"删掉了几个"，而不是笼统报错——
   * 部分成功在对象存储里是常态。
   */
  const deleteSelection = useCallback(async () => {
    const keys = [...selectedKeys];
    setSelectionToDelete(false);
    if (keys.length === 0) {
      return;
    }
    setLoading(true);
    setError(null);
    let deleted = 0;
    try {
      for (const key of keys) {
        await service.deleteObject(provider.id, key, { bucket, region: activeRegion });
        deleted += 1;
      }
      clearSelection();
    } catch (err) {
      setError(
        `${formatMutationError(err, t('errorDeleteObject'))} (${t('bucketsBrowserDeletedCount', {
          deleted,
          total: keys.length,
        })})`,
      );
      // 已删掉的从选中集里摘掉，剩下的留在选中态方便重试。
      setSelectedKeys(new Set(keys.slice(deleted)));
    } finally {
      void loadObjects(currentPrefix, pageIndex, { keepView: true, preserveError: true });
    }
  }, [activeRegion, bucket, clearSelection, currentPrefix, loadObjects, provider.id, selectedKeys, service, t]);

  /**
   * 批量下载：逐个读内容并交给浏览器保存。
   *
   * 超过内联上限的对象在这里读不了，跳过并一次性说明；单次动作也做了上限，避免一次点击
   * 触发上百个下载（浏览器会拦截，用户也拿不到东西）。
   */
  const downloadSelection = useCallback(async () => {
    // 选中集是跨页保留的，所以从"所有已加载的页"里取对象，而不是只看当前页。
    const selected = loadedObjects.filter((object) => selectedKeys.has(object.key));
    const folders = selected.filter((object) => object.isFolder);
    const files = selected.filter((object) => !object.isFolder);
    const tooLarge = files.filter((object) => object.sizeBytes > MAX_OBJECT_CONTENT_BYTES);
    const downloadable = files
      .filter((object) => object.sizeBytes <= MAX_OBJECT_CONTENT_BYTES)
      .slice(0, MAX_BULK_DOWNLOAD_COUNT);

    setError(null);
    if (folders.length > 0 || tooLarge.length > 0 || downloadable.length < files.length) {
      setError(
        t('bucketsBrowserSelectionSkipped', {
          folders: folders.length,
          limit: formatByteSize(MAX_OBJECT_CONTENT_BYTES),
          tooLarge: tooLarge.length,
        }),
      );
    }
    for (const object of downloadable) {
      await downloadObject(object);
    }
  }, [downloadObject, loadedObjects, selectedKeys, t]);

  /** `aria-sort` 只认单个激活列；未激活一律 none。 */
  const ariaSortFor = (key: BucketObjectSortKey): 'ascending' | 'descending' | 'none' => {
    if (sort.key !== key) {
      return 'none';
    }
    return sort.direction === 'asc' ? 'ascending' : 'descending';
  };

  /**
   * 一次"加载全部"最多走多少页。
   *
   * 排序要覆盖整个目录就必须先把所有页取回来，而游标分页没有总数，所以只能设上限：
   * 20 页 × 每页 200 = 最多 4000 个对象，足够覆盖常见的"一个目录几百上千个文件"，
   * 又不会让一次点击变成无界请求。到顶时明确说明，而不是假装已经加载完。
   */
  const LOAD_ALL_MAX_PAGES = 20;

  /**
   * 加载全部页，让排序覆盖整个目录。
   *
   * 顺序翻页（用游标链）而不是并发请求：游标本身就是串行的，而且顺序加载能让进度可见、
   * 中途失败时已加载的部分仍然可用。
   */
  const loadAllPages = async () => {
    if (loadingAll || loading || !hasMore) {
      return;
    }
    setLoadingAll(true);
    setError(null);
    try {
      let index = pagesRef.current.length;
      let more = pagesRef.current[pagesRef.current.length - 1]?.hasMore ?? false;
      while (more && index < LOAD_ALL_MAX_PAGES) {
        await loadObjects(currentPrefix, index, { keepView: true });
        index += 1;
        more = pagesRef.current[index - 1]?.hasMore ?? false;
      }
      if (more) {
        setError(
          t('bucketsBrowserLoadAllLimit', {
            count: pagesRef.current.reduce((total, page) => total + (page.items?.length ?? 0), 0),
          }),
        );
      }
    } finally {
      setLoadingAll(false);
    }
  };
  const openPreview = (object: StorageProviderObjectView) => {
    if (object.isFolder) {
      navigateToFolder(object.key);
      return;
    }
    setPreviewTarget(object);
  };

  return (
    <>
      <Modal open={open} onOpenChange={onOpenChange}>
        <ModalContent
          align="center"
          className="h-[90vh] max-h-[90vh] w-[90vw] max-w-[90vw] grid-rows-[auto_minmax(0,1fr)]"
          size="full"
        >
          {/* 头部压成一行：标题 + 桶 + 端点，其余交给正文。关闭入口是右上角的 X 与 Esc。 */}
          <ModalHeader className="flex-row flex-wrap items-center gap-x-3 gap-y-1 px-4 py-2.5 pr-12">
            <ModalTitle className="shrink-0 text-sm font-semibold">
              {t('bucketsBrowseTitle', { bucket })}
            </ModalTitle>
            <span
              className={`${BADGE_BASE_CLASS} shrink-0 bg-neutral-100 font-mono text-neutral-600 dark:bg-neutral-800 dark:text-neutral-300`}
            >
              {bucket}
            </span>
            <span className="min-w-0 flex-1 truncate text-[11px] text-neutral-500 dark:text-neutral-400">
              {providerDisplayName(t, provider)} · {provider.endpointUrl}
              {provider.region ? ` · ${provider.region}` : ''}
              {/* 桶来自账号的凭证：头部必须写明是哪个账号，否则多账号端点下无从分辨。 */}
              {accountName ? ` · ${t('bucketsProviderAccount')} ${accountName}` : ''}
            </span>
          </ModalHeader>

          {/*
            `overflow-hidden`：正文本身不滚动。
            
            之前这里继承的是可滚动容器，于是弹窗里同时存在三层滚动（正文 / 右栏 / 行区域）：
            滚轮到底先滚正文，表头跟着一起走——看起来就是"滚动条和 header 一起滚动"。
            现在只有行区域滚动（见下方 rowsRef），表头与工具栏固定。
          */}
          <ModalBody className="min-h-0 overflow-hidden px-3 py-3">
            <div className="flex h-full min-h-0 flex-col overflow-hidden rounded-lg border border-neutral-200 dark:border-neutral-800 lg:flex-row">
              <BucketCategoryRail
                bucket={bucket}
                category={category}
                counts={counts}
                loadedBytes={loadedBytes}
                onSelectCategory={setCategory}
                provider={provider}
                region={activeRegion}
              />

              <section className="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden">
                <div className="flex shrink-0 flex-wrap items-center gap-2 border-b border-neutral-100 px-4 py-2 dark:border-neutral-800">
                  <nav
                    aria-label={t('files')}
                    className="flex min-w-0 flex-1 flex-wrap items-center gap-0.5 text-xs"
                  >
                    <button
                      type="button"
                      onClick={() => navigateToFolder('')}
                      disabled={loading || uploading}
                      className="rounded px-1.5 py-1 font-medium text-neutral-600 transition-colors hover:bg-neutral-100 hover:text-neutral-900 disabled:cursor-not-allowed disabled:opacity-50 dark:text-neutral-300 dark:hover:bg-neutral-800 dark:hover:text-white"
                    >
                      {t('root')}
                    </button>
                    {breadcrumbSegments.map((segment, index) => (
                      <React.Fragment key={`${segment}-${index}`}>
                        <span aria-hidden="true" className="text-neutral-400">
                          /
                        </span>
                        <button
                          type="button"
                          className="max-w-[12rem] truncate rounded px-1.5 py-1 font-mono text-blue-600 transition-colors hover:bg-blue-50 disabled:cursor-not-allowed disabled:opacity-50 dark:text-blue-400 dark:hover:bg-blue-950/30"
                          disabled={loading}
                          onClick={() =>
                            navigateToFolder(`${breadcrumbSegments.slice(0, index + 1).join('/')}/`)
                          }
                        >
                          {segment}
                        </button>
                      </React.Fragment>
                    ))}
                  </nav>

                  <div className="relative w-full min-w-0 shrink-0 sm:w-52">
                    <Search
                      aria-hidden="true"
                      className="pointer-events-none absolute left-2.5 top-1/2 -translate-y-1/2 text-neutral-400"
                      size={14}
                    />
                    <input
                      type="search"
                      value={query}
                      onChange={(event) => setQuery(event.target.value)}
                      placeholder={t('bucketsBrowserSearchPlaceholder')}
                      aria-label={t('bucketsBrowserSearchPlaceholder')}
                      className={`${INPUT_CLASS} h-8 pl-8 text-[13px]`}
                    />
                  </div>

                  <div className="flex shrink-0 flex-wrap items-center gap-1.5">
                    {/* 当前页里"筛选后可见 / 本页总数"：分页后这个比例比"已加载总数"更有意义。 */}
                    <span className="hidden text-[11px] text-neutral-400 sm:inline dark:text-neutral-500">
                      {t('bucketsBrowserVisible', {
                        shown: visibleObjects.length,
                        total: pageItems.length,
                      })}
                    </span>
                    {currentPrefix ? (
                      <button
                        type="button"
                        onClick={navigateUp}
                        disabled={loading || uploading}
                        className={GHOST_BUTTON_CLASS}
                      >
                        <ArrowLeft aria-hidden="true" size={14} />
                        <span className="hidden sm:inline">{t('up')}</span>
                      </button>
                    ) : null}
                    {/*
                      选中态就地替换动作组，而不是新增一行：多选是临时状态，为它常驻一条
                      工具栏等于永久少吃一行列表高度。
                    */}
                    {selectedKeys.size > 0 ? (
                      <>
                        <span className="shrink-0 text-[11px] font-medium text-neutral-600 dark:text-neutral-300">
                          {t('bucketsBrowserSelected', { count: selectedKeys.size })}
                        </span>
                        <button
                          type="button"
                          onClick={() => void downloadSelection()}
                          disabled={downloadingKey !== null || loading}
                          className={SECONDARY_BUTTON_CLASS}
                          title={t('bucketsBrowserDownloadSelected')}
                        >
                          <Download aria-hidden="true" size={14} />
                          <span className="hidden sm:inline">
                            {t('bucketsBrowserDownloadSelected')}
                          </span>
                        </button>
                        <button
                          type="button"
                          onClick={() => setSelectionToDelete(true)}
                          disabled={loading || uploading}
                          className="inline-flex h-9 shrink-0 items-center justify-center gap-1.5 rounded-md border border-red-200 bg-red-50 px-3 text-sm font-medium text-red-700 transition-colors hover:bg-red-100 disabled:cursor-not-allowed disabled:opacity-50 dark:border-red-900/60 dark:bg-red-950/30 dark:text-red-300 dark:hover:bg-red-950/50"
                        >
                          <Trash2 aria-hidden="true" size={14} />
                          <span className="hidden sm:inline">
                            {t('bucketsBrowserDeleteSelected')}
                          </span>
                        </button>
                        <button
                          type="button"
                          onClick={clearSelection}
                          className={GHOST_BUTTON_CLASS}
                          aria-label={t('bucketsBrowserClearSelection')}
                          title={t('bucketsBrowserClearSelection')}
                        >
                          <X aria-hidden="true" size={14} />
                        </button>
                      </>
                    ) : (
                      <>
                        <button
                          type="button"
                          onClick={() => openPrompt({ kind: 'newFolder', initialValue: '' })}
                          disabled={loading || uploading}
                          className={SECONDARY_BUTTON_CLASS}
                        >
                          <FolderPlus aria-hidden="true" size={14} />
                          <span className="hidden sm:inline">{t('newFolder')}</span>
                        </button>
                        {/*
                          上传必须是真正的按钮：包一层 `<label>` + `display:none` 的 file input
                          只能鼠标点击，键盘用户根本 Tab 不到它（`aria-disabled` 在 label 上也不生效）。
                          这里用按钮触发 input.click()，input 用 sr-only 保持在无障碍树里。
                        */}
                        <button
                          type="button"
                          onClick={() => fileInputRef.current?.click()}
                          disabled={uploading}
                          className={PRIMARY_BUTTON_CLASS}
                          title={t('bucketsBrowserUploadLimitHint', {
                            limit: formatByteSize(MAX_OBJECT_CONTENT_BYTES),
                          })}
                        >
                          {uploading ? (
                            <LoaderCircle aria-hidden="true" className="animate-spin" size={14} />
                          ) : (
                            <Upload aria-hidden="true" size={14} />
                          )}
                          <span className="hidden sm:inline">
                            {uploading
                              ? uploadTotal > 1
                                ? `${t('uploading')} ${uploadDone + 1}/${uploadTotal}`
                                : // 分片直传时给出百分比：大文件要传很久，没有进度用户会以为卡住了。
                                  uploadPercent > 0
                                  ? t('bucketsBrowserUploadProgress', { percent: uploadPercent })
                                  : t('uploading')
                              : t('upload')}
                          </span>
                        </button>
                        <input
                          ref={fileInputRef}
                          className="sr-only"
                          type="file"
                          multiple
                          tabIndex={-1}
                          disabled={uploading}
                          aria-label={t('upload')}
                          onChange={(event) => {
                            const files = Array.from(event.target.files ?? []);
                            if (files.length > 0) {
                              void uploadFiles(files);
                            }
                          }}
                        />
                        <button
                          type="button"
                          onClick={() => void loadObjects(currentPrefix, pageIndex, { keepView: true })}
                          disabled={loading || uploading}
                          className={GHOST_BUTTON_CLASS}
                          aria-label={t('refresh')}
                          title={t('refresh')}
                        >
                          <RefreshCw
                            aria-hidden="true"
                            className={loading ? 'animate-spin' : undefined}
                            size={14}
                          />
                        </button>
                      </>
                    )}
                  </div>
                </div>

                {/*
                  拖拽上传：网盘里最常用的入口之一。拖拽只改变高亮，落点逻辑与"选择文件"
                  完全共用同一条 uploadFiles（含体积预检与覆盖确认）。
                */}
                <div
                  ref={rowsRef}
                  data-bucket-rows
                  className="relative min-h-0 flex-1 overflow-y-auto"
                  onKeyDown={(event) => {
                    const target = event.target as HTMLElement | null;
                    // 输入框里的按键属于输入框：不要抢 Ctrl+A（全选文本）与 Delete（删字符）。
                    if (
                      target
                      && (target.tagName === 'INPUT'
                        || target.tagName === 'TEXTAREA'
                        || target.isContentEditable)
                    ) {
                      return;
                    }
                    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'a') {
                      event.preventDefault();
                      setSelectedKeys(new Set(visibleObjects.map((object) => object.key)));
                      return;
                    }
                    if (event.key === 'Escape' && selectedKeys.size > 0) {
                      event.preventDefault();
                      clearSelection();
                      return;
                    }
                    if (
                      (event.key === 'Delete' || event.key === 'Backspace')
                      && selectedKeys.size > 0
                    ) {
                      event.preventDefault();
                      setSelectionToDelete(true);
                    }
                  }}
                  onDragEnter={(event) => {
                    if (event.dataTransfer?.types.includes('Files')) {
                      setDragActive(true);
                    }
                  }}
                  onDragOver={(event) => {
                    if (event.dataTransfer?.types.includes('Files')) {
                      event.preventDefault();
                      setDragActive(true);
                    }
                  }}
                  onDragLeave={(event) => {
                    // 只有真正离开容器才取消高亮：子元素之间移动也会触发 dragleave。
                    if (!event.currentTarget.contains(event.relatedTarget as Node | null)) {
                      setDragActive(false);
                    }
                  }}
                  onDrop={(event) => {
                    if (!event.dataTransfer?.files?.length) {
                      return;
                    }
                    event.preventDefault();
                    setDragActive(false);
                    void uploadFiles(Array.from(event.dataTransfer.files));
                  }}
                >
                  {dragActive ? (
                    <div className="pointer-events-none absolute inset-2 z-20 flex items-center justify-center rounded-lg border-2 border-dashed border-blue-400 bg-blue-50/85 text-xs font-medium text-blue-700 dark:border-blue-500 dark:bg-blue-950/70 dark:text-blue-200">
                      {t('bucketsBrowserDropHint')}
                    </div>
                  ) : null}
                  {error ? (
                    <div className="m-3 flex items-start gap-2 rounded-md border border-red-200 bg-red-50 px-3 py-2 text-xs text-red-700 dark:border-red-900 dark:bg-red-950/20 dark:text-red-300">
                      <CircleAlert aria-hidden="true" className="mt-0.5 shrink-0" size={14} />
                      <span className="flex-1">{error}</span>
                    </div>
                  ) : null}

                  {/* 读取失败时不要显示"此目录为空"：没读到 ≠ 没有内容，那句话会把人带偏。 */}
                  {visibleObjects.length === 0 && !loading && error === null ? (
                    <div className="m-3 flex flex-col items-center justify-center gap-1.5 rounded-md border border-dashed border-neutral-200 py-12 text-center dark:border-neutral-700">
                      <FolderPlus
                        aria-hidden="true"
                        className="text-neutral-300 dark:text-neutral-600"
                        size={26}
                      />
                      <p className="text-xs text-neutral-500">
                        {pageItems.length === 0
                          ? t('empty')
                          : normalizedQuery
                            ? t('bucketsBrowserSearchNoMatch')
                            : t('bucketsBrowserFilteredEmpty')}
                      </p>
                      <p className="text-[11px] text-neutral-400">
                        {pageItems.length === 0
                          ? t('emptyFolderHint')
                          : // 搜索只覆盖已加载的分页；还有下一页时不能说"没有匹配"，
                            // 否则用户会以为目录里真的没有这个对象。
                            hasMore
                            ? t('bucketsBrowserSearchPartial')
                            : t('bucketsBrowserCategoryHint')}
                      </p>
                    </div>
                  ) : (
                    <table className="w-full table-fixed border-separate border-spacing-0 text-[13px]">
                      <thead>
                        <tr className="text-left text-[11px] font-medium uppercase tracking-wide text-neutral-400 dark:text-neutral-500">
                          <th className="sticky top-0 z-10 w-10 border-b border-neutral-200 bg-white/95 px-3 py-2 backdrop-blur dark:border-neutral-800 dark:bg-neutral-900/95">
                            {/* 表头复选框：选中范围是"当前可见"（已加载 + 已筛选）。 */}
                            <input
                              type="checkbox"
                              className="h-3.5 w-3.5 cursor-pointer rounded border-neutral-300 text-blue-600 focus-visible:ring-2 focus-visible:ring-blue-500/60 dark:border-neutral-600 dark:bg-neutral-800"
                              aria-label={t('bucketsBrowserSelectAll')}
                              checked={
                                visibleObjects.length > 0
                                && visibleObjects.every((object) => selectedKeys.has(object.key))
                              }
                              ref={(node) => {
                                if (node) {
                                  node.indeterminate =
                                    selectedKeys.size > 0
                                    && !visibleObjects.every((object) => selectedKeys.has(object.key));
                                }
                              }}
                              onChange={(event) => {
                                if (event.target.checked) {
                                  setSelectedKeys(new Set(visibleObjects.map((object) => object.key)));
                                } else {
                                  clearSelection();
                                }
                              }}
                            />
                          </th>
                          <th
                            aria-sort={ariaSortFor('name')}
                            className="sticky top-0 z-10 border-b border-neutral-200 bg-white/95 px-3 py-2 backdrop-blur dark:border-neutral-800 dark:bg-neutral-900/95"
                          >
                            <SortHeaderButton
                              active={sort.key === 'name'}
                              direction={sort.direction}
                              label={t('nameHeader')}
                              onClick={() => setSort((current) => nextBucketObjectSort(current, 'name'))}
                              sortByLabel={t('bucketsBrowserSortBy', { field: t('nameHeader') })}
                            />
                          </th>
                          <th
                            aria-sort={ariaSortFor('size')}
                            className="sticky top-0 z-10 hidden w-24 border-b border-neutral-200 bg-white/95 px-3 py-2 backdrop-blur dark:border-neutral-800 dark:bg-neutral-900/95 sm:table-cell"
                          >
                            <SortHeaderButton
                              active={sort.key === 'size'}
                              direction={sort.direction}
                              label={t('sizeHeader')}
                              onClick={() => setSort((current) => nextBucketObjectSort(current, 'size'))}
                              sortByLabel={t('bucketsBrowserSortBy', { field: t('sizeHeader') })}
                            />
                          </th>
                          <th
                            aria-sort={ariaSortFor('modified')}
                            className="sticky top-0 z-10 hidden w-40 border-b border-neutral-200 bg-white/95 px-3 py-2 backdrop-blur dark:border-neutral-800 dark:bg-neutral-900/95 md:table-cell"
                          >
                            <SortHeaderButton
                              active={sort.key === 'modified'}
                              direction={sort.direction}
                              label={t('modifiedHeader')}
                              onClick={() =>
                                setSort((current) => nextBucketObjectSort(current, 'modified'))
                              }
                              sortByLabel={t('bucketsBrowserSortBy', { field: t('modifiedHeader') })}
                            />
                          </th>
                          <th className="sticky top-0 z-10 w-[8.5rem] border-b border-neutral-200 bg-white/95 px-3 py-2 text-right backdrop-blur dark:border-neutral-800 dark:bg-neutral-900/95">
                            {t('actHeader')}
                          </th>
                        </tr>
                      </thead>
                      <tbody>
                        {visibleObjects.map((object) => {
                          const kind = resolveBucketObjectCategory(object);
                          const name = relativeObjectPath(object.key, currentPrefix);
                          const tooLargeToTransfer = !object.isFolder
                            && object.sizeBytes > MAX_OBJECT_CONTENT_BYTES;
                          const isPreviewing = previewTarget?.key === object.key;
                          const isSelected = selectedKeys.has(object.key);
                          return (
                            <tr
                              key={object.key}
                              // 行本身是按钮：整行可点，也必须能被键盘 Tab 到并用
                              // Enter/Space 打开（`aria-selected` 在普通 table 的 row 上
                              // 无效，所以用 `aria-current` 表达「正在预览这一行」）。
                              aria-current={isPreviewing ? 'true' : undefined}
                              data-selected={isSelected ? 'true' : undefined}
                              tabIndex={0}
                              className={`group h-12 cursor-pointer border-b border-neutral-100 outline-none transition-colors last:border-0 focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-blue-500/60 dark:border-neutral-800/70 ${
                                isSelected
                                  ? 'bg-blue-50 dark:bg-blue-950/40'
                                  : isPreviewing
                                    ? 'bg-blue-50/80 dark:bg-blue-950/30'
                                    : 'hover:bg-neutral-50 dark:hover:bg-neutral-800/50'
                              }`}
                              onClick={() => openPreview(object)}
                              onKeyDown={(event) => {
                                if (event.key !== 'Enter' && event.key !== ' ') {
                                  return;
                                }
                                // 空格默认会滚动页面；行级快捷键必须自己吃掉它。
                                event.preventDefault();
                                openPreview(object);
                              }}
                            >
                              <td className="px-3 py-1.5">
                                {/*
                                  复选框只在悬停、选中或自身获得焦点时出现：常驻会一直占着
                                  视觉重量，而键盘用户靠 focus-within 同样够得到。
                                */}
                                <input
                                  type="checkbox"
                                  className={`h-3.5 w-3.5 cursor-pointer rounded border-neutral-300 text-blue-600 focus-visible:ring-2 focus-visible:ring-blue-500/60 dark:border-neutral-600 dark:bg-neutral-800 ${
                                    isSelected
                                      ? 'opacity-100'
                                      : 'opacity-0 group-hover:opacity-100 group-focus-within:opacity-100 focus-visible:opacity-100'
                                  }`}
                                  aria-label={t('bucketsBrowserSelectRow', { name })}
                                  checked={isSelected}
                                  onChange={() => toggleSelection(object.key)}
                                  onClick={(event) => event.stopPropagation()}
                                />
                              </td>
                              <td className="px-3 py-1.5">
                                <div className="flex min-w-0 items-center gap-2.5">
                                  <FileKindIcon
                                    kind={resolveFilePreviewKind({
                                      name: object.key,
                                      contentType: object.contentType,
                                      isFolder: object.isFolder,
                                    })}
                                    size="sm"
                                  />
                                  <span
                                    className={`min-w-0 flex-1 truncate ${
                                      object.isFolder
                                        ? 'font-medium text-neutral-800 dark:text-neutral-100'
                                        : 'text-neutral-700 dark:text-neutral-200'
                                    }`}
                                    title={name}
                                  >
                                    {name}
                                  </span>
                                </div>
                              </td>
                              <td className="hidden px-3 py-1.5 tabular-nums text-neutral-500 dark:text-neutral-400 sm:table-cell">
                                {object.isFolder ? '—' : formatByteSize(object.sizeBytes)}
                              </td>
                              <td className="hidden truncate px-3 py-1.5 text-neutral-400 md:table-cell">
                                {object.lastModifiedIso ? (
                                  <time dateTime={modifiedTimeIso(object.lastModifiedIso)}>
                                    {formatModifiedTime(object.lastModifiedIso, language)}
                                  </time>
                                ) : (
                                  '—'
                                )}
                              </td>
                              <td className="px-3 py-1.5">
                                <div className="flex items-center justify-end gap-0.5 opacity-100 lg:opacity-0 lg:group-hover:opacity-100 lg:group-focus-within:opacity-100">
                                  {!object.isFolder ? (
                                    <button
                                      type="button"
                                      className="inline-flex h-7 w-7 items-center justify-center rounded-md text-neutral-500 transition-colors hover:bg-neutral-200/70 hover:text-neutral-900 dark:text-neutral-400 dark:hover:bg-neutral-700 dark:hover:text-white"
                                      title={t('bucketsPreviewFile')}
                                      aria-label={t('bucketsPreviewFile')}
                                      onClick={(event) => {
                                        event.stopPropagation();
                                        setPreviewTarget(object);
                                      }}
                                    >
                                      <Eye aria-hidden="true" size={14} />
                                    </button>
                                  ) : null}
                                  {!object.isFolder ? (
                                    <button
                                      type="button"
                                      className="inline-flex h-7 w-7 items-center justify-center rounded-md text-neutral-500 transition-colors hover:bg-neutral-200/70 hover:text-neutral-900 disabled:opacity-40 dark:text-neutral-400 dark:hover:bg-neutral-700 dark:hover:text-white"
                                      // 超过浏览器内读写上限的对象：按钮保留但禁用，并说明原因 ——
                                      // 直接藏起来会让用户以为"这个文件不能下载"，而其实是"这里下不了、要去厂商控制台"。
                                      title={
                                        tooLargeToTransfer
                                          ? t('bucketsBrowserTransferLimitHint', {
                                              limit: formatByteSize(MAX_OBJECT_CONTENT_BYTES),
                                            })
                                          : t('download')
                                      }
                                      aria-label={t('download')}
                                      disabled={
                                        loading || downloadingKey !== null || tooLargeToTransfer
                                      }
                                      onClick={(event) => {
                                        event.stopPropagation();
                                        void downloadObject(object);
                                      }}
                                    >
                                      {downloadingKey === object.key ? (
                                        <LoaderCircle
                                          aria-hidden="true"
                                          className="animate-spin"
                                          size={14}
                                        />
                                      ) : (
                                        <Download aria-hidden="true" size={14} />
                                      )}
                                    </button>
                                  ) : null}
                                  {!object.isFolder ? (
                                    <button
                                      type="button"
                                      className="inline-flex h-7 w-7 items-center justify-center rounded-md text-neutral-500 transition-colors hover:bg-neutral-200/70 hover:text-neutral-900 disabled:opacity-40 dark:text-neutral-400 dark:hover:bg-neutral-700 dark:hover:text-white"
                                      title={t('rename')}
                                      aria-label={t('rename')}
                                      disabled={loading}
                                      onClick={(event) => {
                                        event.stopPropagation();
                                        // 重命名只换当前目录里的名字：目标 key 由当前前缀拼出，
                                        // 而不是用完整 key 覆盖，否则嵌套目录会被拍平。
                                        openPrompt({
                                          kind: 'rename',
                                          objectKey: `${currentPrefix}${name}`,
                                          initialValue: name,
                                        });
                                      }}
                                    >
                                      <PencilLine aria-hidden="true" size={14} />
                                    </button>
                                  ) : null}
                                  <button
                                    type="button"
                                    className="inline-flex h-7 w-7 items-center justify-center rounded-md text-red-500 transition-colors hover:bg-red-100 hover:text-red-700 disabled:opacity-40 dark:text-red-400 dark:hover:bg-red-950/40 dark:hover:text-red-300"
                                    title={
                                      isDirectoryPlaceholder(object.key)
                                        ? t('bucketsBrowserDeleteFolderHint')
                                        : t('del')
                                    }
                                    aria-label={t('del')}
                                    disabled={loading}
                                    onClick={(event) => {
                                      event.stopPropagation();
                                      setDeleteTarget(object);
                                    }}
                                  >
                                    <Trash2 aria-hidden="true" size={14} />
                                  </button>
                                </div>
                              </td>
                            </tr>
                          );
                        })}
                      </tbody>
                    </table>
                  )}
                </div>

                {/*
                  分页条固定在行区域下方（不属于滚动内容）。

                  为什么是分页而不是"滚到底自动加载"：对象列表接口是游标分页，没有 offset、
                  也没有总数，无限滚动只会得到一条越滚越长的列表——滚动位置不稳定，回到
                  某一页只能靠记忆，而且"到底了没有"永远看不出来。分页让每一页的位置固定、
                  翻页可预期；游标只支持向前，所以"上一页"用已经走过的页（内存里），
                  没有走过的地方不假装能跳。
                */}
                <div
                  data-bucket-page={pageIndex + 1}
                  data-bucket-page-size={pageSize}
                  className="flex shrink-0 flex-wrap items-center gap-x-3 gap-y-2 border-t border-neutral-200 px-4 py-2 text-[11px] text-neutral-500 dark:border-neutral-800 dark:text-neutral-400"
                >
                  <span className="shrink-0 whitespace-nowrap tabular-nums">
                    {t('bucketsBrowserPageLabel', { page: pageIndex + 1 })}
                  </span>
                  <span className="shrink-0 text-neutral-300 dark:text-neutral-600">·</span>
                  <span className="shrink-0 whitespace-nowrap tabular-nums">
                    {t('bucketsBrowserPageItems', { count: pageItems.length })}
                  </span>

                  {/*
                    排序只作用于已加载的页：接口是游标分页且没有排序参数（S3/COS 本身只按
                    键名字典序返回），所以"按大小/时间排整个目录"必须先取回整个目录。有
                    下一页时把这一点说出来，并给一个带上限的"加载全部"。
                  */}
                  {hasMore ? (
                    <>
                      <span className="shrink-0 text-neutral-300 dark:text-neutral-600">·</span>
                      <span className="shrink-0 whitespace-nowrap">
                        {t('bucketsBrowserSortPartial')}
                      </span>
                      <button
                        type="button"
                        className="shrink-0 whitespace-nowrap font-medium text-blue-600 hover:underline disabled:opacity-50 dark:text-blue-400"
                        disabled={loadingAll || loading}
                        onClick={() => void loadAllPages()}
                      >
                        {loadingAll ? t('bucketsBrowserLoadAllRunning') : t('bucketsBrowserLoadAll')}
                      </button>
                    </>
                  ) : null}

                  <span className="flex-1" />

                  <label className="flex shrink-0 items-center gap-1.5 whitespace-nowrap">
                    <span className="whitespace-nowrap">{t('bucketsBrowserPageSize')}</span>
                    <select
                      value={pageSize}
                      disabled={loading}
                      aria-label={t('bucketsBrowserPageSize')}
                      className={`${INPUT_CLASS} h-7 w-auto py-0 text-[11px]`}
                      onChange={(event) => {
                        const next = Number(event.target.value);
                        setPageSize(next);
                        // 每页条数变了，游标链就作废：回到第一页重新走（并显式带上新值）。
                        void loadObjects(currentPrefix, 0, {
                          keepView: true,
                          pageSize: next,
                          reset: true,
                        });
                      }}
                    >
                      {OBJECT_PAGE_SIZE_OPTIONS.map((size) => (
                        <option key={size} value={size}>
                          {size}
                        </option>
                      ))}
                    </select>
                  </label>

                  <div className="flex shrink-0 items-center gap-1">
                    <button
                      type="button"
                      className={GHOST_BUTTON_CLASS}
                      disabled={pageIndex === 0 || loading}
                      aria-label={t('bucketsBrowserPagePrevious')}
                      onClick={() => goToPage(pageIndex - 1)}
                    >
                      <ChevronLeft aria-hidden="true" size={14} />
                      <span className="hidden sm:inline">{t('bucketsBrowserPagePrevious')}</span>
                    </button>
                    <button
                      type="button"
                      className={GHOST_BUTTON_CLASS}
                      disabled={!hasMore || loading}
                      aria-label={t('bucketsBrowserPageNext')}
                      onClick={() => goToPage(pageIndex + 1)}
                    >
                      <span className="hidden sm:inline">{t('bucketsBrowserPageNext')}</span>
                      {loading && hasMore ? (
                        <LoaderCircle aria-hidden="true" className="animate-spin" size={14} />
                      ) : (
                        <ChevronRight aria-hidden="true" size={14} />
                      )}
                    </button>
                  </div>
                </div>
              </section>
            </div>
          </ModalBody>
        </ModalContent>
      </Modal>

      {/*
        预览是嵌套对话框：与文件管理器同尺寸（90% 视口），关闭后回到原来的目录位置，
        所以用户不会因为看一个文件而丢失浏览上下文。
      */}
      {previewTarget ? (
        <BucketObjectPreviewDialog
          bucket={bucket}
          labels={labels}
          navigation={previewNavigation}
          object={previewTarget}
          onClose={() => {
            setPreviewTarget(null);
            // 编辑并保存过内容后，列表里的体积与修改时间已经过期：关闭预览时重取一次，
            // 而不是让用户对着一行旧数据继续操作。
            if (savedInPreviewRef.current) {
              savedInPreviewRef.current = false;
              void loadObjects(currentPrefix);
            }
          }}
          onSaved={() => {
            savedInPreviewRef.current = true;
          }}
          provider={provider}
          region={region}
          service={service}
        />
      ) : null}

      {/*
        新建文件夹 / 重命名用同一套内嵌对话框，而不是自绘遮罩：文件管理器本身是
        Radix 模态，页面树上的自绘遮罩会被它标成 aria-hidden，屏幕阅读器读不到。
      */}
      <Modal
        open={prompt !== null}
        onOpenChange={(next) => {
          if (!next && !promptBusy) {
            setPrompt(null);
          }
        }}
      >
        <ModalContent size="sm">
          <ModalHeader>
            <ModalTitle>
              {prompt?.kind === 'newFolder' ? t('newFolderDialogTitle') : t('renameDialogTitle')}
            </ModalTitle>
          </ModalHeader>
          <ModalBody>
            <input
              autoFocus
              className={INPUT_CLASS}
              value={promptValue}
              placeholder={
                prompt?.kind === 'newFolder' ? t('folderNamePlaceholder') : t('newNamePlaceholder')
              }
              aria-label={
                prompt?.kind === 'newFolder' ? t('newFolderDialogTitle') : t('renameDialogTitle')
              }
              onChange={(event) => setPromptValue(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === 'Enter' && !promptBusy) {
                  void submitPrompt();
                }
                if (event.key === 'Escape' && !promptBusy) {
                  setPrompt(null);
                }
              }}
            />
          </ModalBody>
          <ModalFooter>
            <button
              type="button"
              className={SECONDARY_BUTTON_CLASS}
              disabled={promptBusy}
              onClick={() => setPrompt(null)}
            >
              {t('cancel')}
            </button>
            <button
              type="button"
              className={PRIMARY_BUTTON_CLASS}
              disabled={promptBusy || !promptValue.trim()}
              onClick={() => void submitPrompt()}
            >
              {promptBusy ? t('saving') : t('confirm')}
            </button>
          </ModalFooter>
        </ModalContent>
      </Modal>

      <ConfirmDialog
        confirmLabel={t('del')}
        /*
         * 对象名单独成块，不夹在句子里。
         *
         * 对象名常常是一长串没有断点的字符（哈希文件名、带扩展名的长 key）。夹进
         * 「删除"{key}"？」这种句子里时，浏览器为了塞下这个不可断的长词，会把中文句子
         * 断在任意两个字之间——截图里那种"删"独占一行就是这么来的。把名字放进自己的
         * 块级元素并允许任意断行（`break-all`），句子与数据各占各的行，怎么窄都不会难看。
         */
        description={
          <>
            <span>
              {deleteTarget && isDirectoryPlaceholder(deleteTarget.key)
                ? t('bucketsBrowserDeleteFolderConfirmLabel')
                : t('deleteObjectConfirmLabel')}
            </span>
            <span className="mt-1 block break-all font-mono text-xs text-neutral-600 dark:text-neutral-300">
              {deleteTarget?.key ?? ''}
            </span>
          </>
        }
        onConfirm={() => {
          if (deleteTarget) {
            void deleteObject(deleteTarget.key);
          }
        }}
        onOpenChange={(next) => {
          if (!next) {
            setDeleteTarget(null);
          }
        }}
        open={deleteTarget !== null}
        title={t('del')}
        tone="danger"
      />

      {/* 批量删除确认：数量与不可撤销都写在文案里。 */}
      <ConfirmDialog
        cancelLabel={t('cancel')}
        confirmLabel={t('bucketsBrowserDeleteSelected')}
        description={t('bucketsBrowserDeleteSelectedConfirm', { count: selectedKeys.size })}
        onConfirm={() => void deleteSelection()}
        onOpenChange={(next) => {
          if (!next) {
            setSelectionToDelete(false);
          }
        }}
        open={selectionToDelete}
        title={t('bucketsBrowserDeleteSelected')}
        tone="danger"
      />

      {/*
        覆盖确认：复制与 PUT 都是整对象替换，目标对象的内容不会进回收站。
        没有这一步时，重命名 a.txt→b.txt 会先覆盖 b.txt 再删掉 a.txt，不可撤销。
      */}
      <ConfirmDialog
        cancelLabel={t('cancel')}
        confirmLabel={t('bucketsBrowserOverwriteConfirm')}
        // 同上：待覆盖的对象名各自成块，长哈希文件名不会把句子挤断。
        description={
          <>
            <span>{t('bucketsBrowserOverwriteHintLabel')}</span>
            <span className="mt-1 block break-all font-mono text-xs text-neutral-600 dark:text-neutral-300">
              {(overwrite?.kind === 'rename'
                ? [overwrite.destination]
                : (overwrite?.files ?? []).map((file) => `${currentPrefix}${file.name}`)
              ).join('\n')}
            </span>
          </>
        }
        onConfirm={() => {
          const pending = overwrite;
          setOverwrite(null);
          if (!pending) {
            return;
          }
          if (pending.kind === 'rename') {
            void performRename(pending.objectKey, pending.destination);
            return;
          }
          void uploadFiles(pending.files, { overwriteConfirmed: true });
        }}
        onOpenChange={(next) => {
          if (!next) {
            setOverwrite(null);
          }
        }}
        open={overwrite !== null}
        title={t('bucketsBrowserOverwriteTitle')}
        tone="danger"
      />
    </>
  );
}

function readFileAsBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onerror = () => reject(reader.error ?? new Error('File read failed.'));
    reader.onload = () => {
      const result = typeof reader.result === 'string' ? reader.result : '';
      const comma = result.indexOf(',');
      resolve(comma >= 0 ? result.slice(comma + 1) : result);
    };
    reader.readAsDataURL(file);
  });
}
