import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { Download, ChevronLeft, ChevronRight, Eye, PencilLine, X } from 'lucide-react';
import { FileKindIcon } from './FileKindIcon';
import { PreviewErrorBoundary } from './PreviewErrorBoundary';
import {
  PreviewEmptyPanel,
  PreviewErrorPanel,
  PreviewLoadingPanel,
  PreviewTooLargePanel,
} from './PreviewStatePanels';
import { UnsupportedFilePreview } from './previews/UnsupportedFilePreview';
import { ImageFilePreview } from './previews/ImageFilePreview';
import { PdfFilePreview } from './previews/PdfFilePreview';
import { TextFilePreview } from './previews/TextFilePreview';
import { VideoFilePreview } from './previews/VideoFilePreview';
import { AudioFilePreview } from './previews/AudioFilePreview';
import { WordDocumentPreview } from './previews/WordDocumentPreview';
import { SpreadsheetPreview } from './previews/SpreadsheetPreview';
import { SlidesPreview } from './previews/SlidesPreview';
import { ArchivePreview } from './previews/ArchivePreview';
import { useFilePreviewContent } from '../hooks/useFilePreviewContent';
import { useHostColorMode, type HostColorMode } from '../hooks/useHostColorMode';
import {
  isDelimitedTextFile,
  isEditableKind,
  resolveFilePreviewKind,
  type FilePreviewKind,
} from '../kinds/filePreviewKind';
import { isZipArchive, readZipArchive } from '../ooxml/zipArchive';
import { useFilePreviewLabels, type FilePreviewLabelsInput } from '../i18n/useFilePreviewLabels';
import {
  formatFilePreviewLabel,
  type FilePreviewLabels,
} from '../i18n/filePreviewLabels';
import type { FilePreviewContent, FilePreviewResource } from '../ports/filePreviewResource';
import { formatByteSize } from '../utils/bytes';
import {
  PREVIEW_BADGE_CLASS,
  PREVIEW_GHOST_BUTTON_CLASS,
  PREVIEW_ICON_BUTTON_CLASS,
  PREVIEW_PRIMARY_BUTTON_CLASS,
  PREVIEW_SURFACE_CLASS,
  PREVIEW_TOOLBAR_CLASS,
} from './previewStyles';

/** Monaco 相关组件单独成 chunk：只有真正打开代码/文本文件时才加载编辑器内核。 */
const CodeFilePreview = React.lazy(() =>
  import('./previews/CodeFilePreview').then((module) => ({ default: module.CodeFilePreview })),
);
const CodeFileEditor = React.lazy(() =>
  import('./previews/CodeFileEditor').then((module) => ({ default: module.CodeFileEditor })),
);

/**
 * 相邻文件导航。
 *
 * 宿主是唯一知道"这个文件在列表里的位置"的一方（它握有目录内容与当前筛选），
 * 所以预览层只接收位置与两个回调，不猜测顺序、也不自己去取列表。
 */
export interface FilePreviewNavigation {
  /** 当前文件在序列中的下标（0 起）。 */
  index: number;
  /** 序列总数。 */
  total: number;
  onNext?: () => void;
  onPrevious?: () => void;
}

export interface FilePreviewSurfaceProps {
  className?: string;
  colorMode?: HostColorMode;
  /** 覆盖种类判定（宿主已知文件真实类型时使用）。 */
  kind?: FilePreviewKind;
  labels?: FilePreviewLabelsInput;
  /** 上一个 / 下一个文件；缺省时不渲染切换入口，也不接管左右方向键。 */
  navigation?: FilePreviewNavigation;
  /** 关闭入口；缺省不渲染关闭按钮（宿主可能用外层对话框自己的关闭按钮）。 */
  onClose?: () => void;
  /** 编辑器的脏状态上报；宿主用它拦截「带着未保存修改关闭」。 */
  onDirtyChange?: (dirty: boolean) => void;
  /** 保存成功后的通知；宿主据此刷新列表里的体积与修改时间。 */
  onSaved?: () => void;
  resource: FilePreviewResource;
}

type ViewMode = 'preview' | 'edit';
type SaveState = 'idle' | 'saving' | 'saved' | 'error';

/**
 * 每种 OOXML 包必须存在的标记部件。
 *
 * 只看「是不是 ZIP」还不够：`.ods`/`.odt`/`.odp`、Pages/Numbers/Keynote 同样是 ZIP，
 * 内部结构却完全不同。少了这层判断，表格预览会把 `.ods` 的压缩字节当 UTF-8 解码成
 * 一张乱码表，Word 预览会对非空文件说「文件内容为空」——两种都是在骗用户。标记部件
 * 缺失时老实落到兜底面板，并把下载入口摆出来。
 */
const OOXML_MARKER_PART: Partial<Record<FilePreviewKind, string>> = {
  word: 'word/document.xml',
  spreadsheet: 'xl/workbook.xml',
  presentation: 'ppt/presentation.xml',
};

/**
 * 预览分发器：把「一个文件 + 一份内容通道」变成正确的预览组件。
 *
 * 它只做四件事——判定种类、取内容、在预览/编辑之间切换、把加载与失败状态画出来；
 * 每种格式的表现交给各自的独立组件。宿主因此只需要提供一个 `FilePreviewResource`，
 * 就能拿到整套内联预览能力，而新增一种格式只是多一个组件加一条分支。
 */
export function FilePreviewSurface({
  className,
  colorMode,
  kind: kindOverride,
  labels: labelsInput,
  navigation,
  onClose,
  onDirtyChange,
  onSaved,
  resource,
}: FilePreviewSurfaceProps) {
  const labels = useFilePreviewLabels(labelsInput);
  const kind = useMemo(
    () =>
      kindOverride
      ?? resolveFilePreviewKind({
        name: resource.name,
        contentType: resource.contentType,
      }),
    [kindOverride, resource.contentType, resource.name],
  );
  const canEdit = isEditableKind(kind) && typeof resource.saveText === 'function';
  const [viewMode, setViewMode] = useState<ViewMode>('preview');
  const [saveState, setSaveState] = useState<SaveState>('idle');
  const [saveError, setSaveError] = useState<string | undefined>(undefined);
  /**
   * 未保存的草稿由 Surface 持有，而不是编辑器。
   *
   * 编辑器在「预览 / 编辑」切换、保存后重新取内容时都会卸载重挂；草稿若留在编辑器里，
   * 切一次预览就等于悄悄丢弃用户刚写的内容，连"带着未保存修改关闭"的守卫也会一起失效。
   * `null` 表示"没有草稿"，此时显示的是接口返回的内容。
   */
  const [draft, setDraft] = useState<string | null>(null);
  const { content, reload } = useFilePreviewContent({ kind, resource });
  const savedText = content.status === 'text' ? content.text : undefined;
  const dirty = draft !== null && draft !== savedText;
  // 保存是异步的：关掉预览后回来的响应该被丢弃，否则会对已卸载组件写状态。
  const mountedRef = useRef(true);
  useEffect(() => {
    mountedRef.current = true;
    return () => {
      mountedRef.current = false;
    };
  }, []);

  // 换文件时回到预览态并丢掉上一个文件的草稿。
  useEffect(() => {
    setViewMode('preview');
    setSaveState('idle');
    setSaveError(undefined);
    setDraft(null);
    // 同名同体积同类型的不同对象也是另一个文件；身份必须带上这几个字段。
  }, [resource.contentType, resource.name, resource.sizeBytes]);

  // 脏状态向上报告：宿主据此拦截关闭。
  const dirtyChangeRef = useRef(onDirtyChange);
  dirtyChangeRef.current = onDirtyChange;
  useEffect(() => {
    dirtyChangeRef.current?.(dirty);
  }, [dirty]);

  const draftRef = useRef(draft);
  draftRef.current = draft;

  /**
   * 左右方向键切换相邻文件。
   *
   * 三层让路，缺一就会出现"按一下动两处"或"打字时换文件"：
   * 1. 没有 navigation（宿主没给序列）时完全不接管方向键；
   * 2. 焦点在输入类元素里（搜索框、Monaco 的隐藏 textarea）时不接管——方向键属于光标；
   * 3. 事件已被子组件消费（`defaultPrevented`，例如视频播放器把左右键用于快退/快进）时不接管。
   */
  const handleNavigationKeyDown = useCallback(
    (event: React.KeyboardEvent<HTMLElement>) => {
      if (!navigation || event.defaultPrevented) {
        return;
      }
      const target = event.target as HTMLElement | null;
      if (
        target
        && (target.tagName === 'INPUT'
          || target.tagName === 'TEXTAREA'
          || target.tagName === 'SELECT'
          || target.isContentEditable)
      ) {
        return;
      }
      if (event.key === 'ArrowLeft' && navigation.onPrevious) {
        event.preventDefault();
        navigation.onPrevious();
        return;
      }
      if (event.key === 'ArrowRight' && navigation.onNext) {
        event.preventDefault();
        navigation.onNext();
      }
    },
    [navigation],
  );

  /**
   * 下载失败必须说出来。
   *
   * 端口允许宿主返回 Promise（内容接口有 8 MiB 上限、直链会过期）。宿主自己弹错误时
   * 这里不会重复；宿主静默 reject 时，错误落在这条状态上，用户不会以为"点了没反应"。
   */
  const [downloadError, setDownloadError] = useState<string | undefined>(undefined);
  const handleDownload = useCallback(() => {
    const download = resource.download;
    if (!download) {
      return;
    }
    setDownloadError(undefined);
    void Promise.resolve()
      .then(() => download())
      .catch((error: unknown) => {
        if (!mountedRef.current) {
          return;
        }
        setDownloadError(error instanceof Error ? error.message : String(error));
      });
  }, [resource.download]);

  const handleSave = useCallback(
    (next: string) => {
      const save = resource.saveText;
      if (!save) {
        return;
      }
      setSaveState('saving');
      setSaveError(undefined);
      void save(next)
        .then(() => {
          if (!mountedRef.current) {
            return;
          }
          setSaveState('saved');
          // 只有在保存期间没有新输入时才清空草稿：清掉就等于丢掉那几次击键。
          if (draftRef.current === next) {
            setDraft(null);
          }
          reload();
          // 内容变了，列表里的体积/修改时间就旧了；由宿主决定何时重取。
          onSaved?.();
        })
        .catch((error: unknown) => {
          if (!mountedRef.current) {
            return;
          }
          setSaveState('error');
          setSaveError(error instanceof Error ? error.message : String(error));
        });
    },
    [onSaved, reload, resource.saveText],
  );

  return (
    <section
      className={`${PREVIEW_SURFACE_CLASS} ${className ?? ''}`}
      aria-label={labels.preview}
      onKeyDown={handleNavigationKeyDown}
    >
      <header className={PREVIEW_TOOLBAR_CLASS}>
        <FileKindIcon kind={kind} size="md" />
        <div className="min-w-0 flex-1">
          <p
            className="truncate text-sm font-medium text-neutral-900 dark:text-neutral-100"
            title={resource.name}
          >
            {resource.name}
          </p>
          <p className="flex flex-wrap items-center gap-x-2 text-[11px] text-neutral-500 dark:text-neutral-400">
            <span>{labels.kind[kind]}</span>
            {resource.sizeBytes !== undefined ? <span>· {formatByteSize(resource.sizeBytes)}</span> : null}
            {resource.contentType ? <span className="truncate">· {resource.contentType}</span> : null}
          </p>
        </div>

        <div className="flex shrink-0 items-center gap-1.5">
          {downloadError ? (
            <span
              className="max-w-[16rem] truncate text-[11px] text-red-600 dark:text-red-400"
              role="alert"
              title={downloadError}
            >
              {labels.downloadFailed}: {downloadError}
            </span>
          ) : null}
          {/*
            相邻文件切换：序号 + 左右两个按钮，位置固定在最常用的动作旁边。
            到序列两端就禁用而不是绕回——首尾之间来回跳会让用户失去"我在哪"的感觉。
          */}
          {navigation ? (
            <div className="flex shrink-0 items-center gap-0.5 rounded-md border border-neutral-200 px-0.5 dark:border-neutral-700">
              <button
                type="button"
                className={PREVIEW_ICON_BUTTON_CLASS}
                aria-keyshortcuts="ArrowLeft"
                aria-label={labels.previousFile}
                disabled={navigation.onPrevious === undefined}
                title={labels.previousFile}
                onClick={() => navigation.onPrevious?.()}
              >
                <ChevronLeft aria-hidden="true" size={15} />
              </button>
              <span
                className="min-w-[3.5rem] shrink-0 text-center text-[11px] tabular-nums text-neutral-500 dark:text-neutral-400"
                data-preview-position
              >
                {formatFilePreviewLabel(labels.position, {
                  index: navigation.index + 1,
                  total: navigation.total,
                })}
              </span>
              <button
                type="button"
                className={PREVIEW_ICON_BUTTON_CLASS}
                aria-keyshortcuts="ArrowRight"
                aria-label={labels.nextFile}
                disabled={navigation.onNext === undefined}
                title={labels.nextFile}
                onClick={() => navigation.onNext?.()}
              >
                <ChevronRight aria-hidden="true" size={15} />
              </button>
            </div>
          ) : null}
          {canEdit ? (
            <button
              type="button"
              className={viewMode === 'edit' ? PREVIEW_PRIMARY_BUTTON_CLASS : PREVIEW_GHOST_BUTTON_CLASS}
              aria-pressed={viewMode === 'edit'}
              title={labels.editHint}
              onClick={() => setViewMode((current) => (current === 'edit' ? 'preview' : 'edit'))}
            >
              {viewMode === 'edit' ? (
                <Eye aria-hidden="true" size={14} />
              ) : (
                <PencilLine aria-hidden="true" size={14} />
              )}
              {viewMode === 'edit' ? labels.preview : labels.edit}
            </button>
          ) : null}
          {resource.download ? (
            <button
              type="button"
              className={PREVIEW_GHOST_BUTTON_CLASS}
              onClick={handleDownload}
              title={labels.download}
            >
              <Download aria-hidden="true" size={14} />
              <span className="hidden sm:inline">{labels.download}</span>
            </button>
          ) : null}
          {onClose ? (
            <button
              type="button"
              className={PREVIEW_GHOST_BUTTON_CLASS}
              onClick={onClose}
              aria-label={labels.close}
              title={labels.close}
            >
              <X aria-hidden="true" size={15} />
            </button>
          ) : null}
        </div>
      </header>

      <div className="flex min-h-0 flex-1 flex-col">
        <PreviewBody
          canEdit={canEdit}
          colorMode={colorMode}
          content={content}
          draft={draft}
          kind={kind}
          labels={labels}
          name={resource.name}
          onDraftChange={setDraft}
          onRetry={reload}
          onSave={handleSave}
          resource={resource}
          saveError={saveError}
          saveState={saveState}
          viewMode={viewMode}
        />
      </div>
    </section>
  );
}

interface PreviewBodyProps {
  canEdit: boolean;
  colorMode?: HostColorMode;
  content: FilePreviewContent;
  /** `null` 表示没有草稿，显示接口返回的内容。 */
  draft: string | null;
  kind: FilePreviewKind;
  labels: FilePreviewLabels;
  name: string;
  onDraftChange: (draft: string) => void;
  onRetry: () => void;
  onSave: (next: string) => void;
  resource: FilePreviewResource;
  saveError?: string;
  saveState: SaveState;
  viewMode: ViewMode;
}

/** 状态分支与格式分支都在这里：上层只负责工具条与保存生命周期。 */
function PreviewBody({
  canEdit,
  colorMode,
  content,
  draft,
  kind,
  labels,
  name,
  onDraftChange,
  onRetry,
  onSave,
  resource,
  saveError,
  saveState,
  viewMode,
}: PreviewBodyProps) {
  const hostColorMode = useHostColorMode(colorMode);
  const mediaSource = useMemo(
    () => ({
      bytes: content.status === 'bytes' ? content.bytes : undefined,
      contentType: resource.contentType,
      url: content.status === 'url' ? content.url : undefined,
    }),
    [content, resource.contentType],
  );
  /** 兜底面板在多条分支里复用，抽成常量避免四处重复同一段 JSX。 */
  const unsupportedPanel = (
    <UnsupportedFilePreview
      contentType={resource.contentType}
      download={resource.download}
      labels={labels}
      name={name}
      sizeBytes={resource.sizeBytes}
    />
  );

  if (content.status === 'loading') {
    return <PreviewLoadingPanel labels={labels} />;
  }
  if (content.status === 'error') {
    // 宿主通道错误（网络/鉴权）保留原文：那是运维诊断需要的信息；内部兜底错误则用
    // 本地化文案，避免把 'Unknown preview error' 这种开发者字符串摆到界面上。
    return (
      <PreviewErrorPanel
        labels={labels}
        message={content.message.trim() === '' ? labels.loadFailed : content.message}
        onRetry={onRetry}
      />
    );
  }
  if (content.status === 'too-large') {
    return (
      <PreviewTooLargePanel
        download={resource.download}
        labels={labels}
        limitBytes={content.limitBytes}
        sizeBytes={content.sizeBytes}
      />
    );
  }
  if (content.status === 'empty') {
    return <PreviewEmptyPanel labels={labels} name={name} />;
  }
  if (content.status === 'unavailable') {
    return (
      <UnsupportedFilePreview
        contentType={resource.contentType}
        download={resource.download}
        labels={labels}
        name={name}
        sizeBytes={resource.sizeBytes}
      />
    );
  }

  /**
   * Office 与压缩包都要求内容是 OOXML/ZIP 容器，而且 Office 还要求是**对应那一种**包。
   *
   * 三道判断，缺一不可：
   * 1. 不是 ZIP（`.doc`/`.xls`/`.ppt` 这类旧版二进制、rar/7z/tar）→ 兜底面板；
   * 2. 是 ZIP 但没有该格式的标记部件（`.ods`/`.odt`/`.odp`、Pages/Numbers/Keynote、
   *    以及结构损坏的 OOXML）→ 兜底面板，而不是让解析器把压缩字节当文本解出一张乱码表，
   *    或者对非空文件报「文件内容为空」；
   * 3. 归档本身读不出来（截断、加密）→ 交给解析器报「损坏」，因为那确实是损坏而不是
   *    「格式不支持」。
   *
   * `.csv`/`.tsv` 是例外：它们按分隔符文本解析，本来就不是容器。
   */
  if (kind === 'word' || kind === 'spreadsheet' || kind === 'presentation' || kind === 'archive') {
    if (content.status !== 'bytes') {
      return unsupportedPanel;
    }
    const delimitedText = kind === 'spreadsheet' && isDelimitedTextFile(name);
    if (!delimitedText && !isZipArchive(content.bytes)) {
      return unsupportedPanel;
    }
    const markerPart = OOXML_MARKER_PART[kind];
    if (markerPart) {
      try {
        if (!readZipArchive(content.bytes).byPath.has(markerPart)) {
          return unsupportedPanel;
        }
      } catch {
        // 中央目录都读不出来：这更像「文件坏了」，让解析器给出损坏面板。
      }
    }
    switch (kind) {
      case 'word':
        return <WordDocumentPreview bytes={content.bytes} labels={labels} name={name} />;
      case 'spreadsheet':
        return (
          <SpreadsheetPreview
            bytes={content.bytes}
            contentType={resource.contentType}
            labels={labels}
            name={name}
          />
        );
      case 'presentation':
        return <SlidesPreview bytes={content.bytes} labels={labels} name={name} />;
      default:
        return <ArchivePreview bytes={content.bytes} labels={labels} name={name} />;
    }
  }

  switch (kind) {
    case 'image':
      return <ImageFilePreview labels={labels} name={name} source={mediaSource} />;
    case 'video':
      return <VideoFilePreview labels={labels} name={name} source={mediaSource} />;
    case 'audio':
      return <AudioFilePreview labels={labels} name={name} source={mediaSource} />;
    case 'pdf':
      return <PdfFilePreview labels={labels} name={name} source={mediaSource} />;
    case 'text':
      return content.status === 'text' ? (
        viewMode === 'edit' && canEdit ? (
          <PreviewErrorBoundary
            fallback={(retry) => (
              <PreviewErrorPanel labels={labels} message={labels.parserUnavailableHint} onRetry={retry} />
            )}
          >
            <React.Suspense fallback={<PreviewLoadingPanel labels={labels} />}>
              <CodeFileEditor
                colorMode={hostColorMode}
                labels={labels}
                name={name}
                onChange={onDraftChange}
                onSave={onSave}
                saveError={saveError}
                saveState={saveState}
                value={draft ?? content.text}
              />
            </React.Suspense>
          </PreviewErrorBoundary>
        ) : (
          // 切到预览时显示草稿而不是已保存版本：用户看到的必须还是自己写的内容，
          // 只是暂时不可编辑（脏标记与保存入口仍在工具条上）。
          <TextFilePreview labels={labels} name={name} text={draft ?? content.text} />
        )
      ) : unsupportedPanel;
    case 'markdown':
    case 'code':
      return content.status === 'text' ? (
        <PreviewErrorBoundary
          fallback={(retry) => (
            <PreviewErrorPanel labels={labels} message={labels.parserUnavailableHint} onRetry={retry} />
          )}
        >
          <React.Suspense fallback={<PreviewLoadingPanel labels={labels} />}>
            {viewMode === 'edit' && canEdit ? (
              <CodeFileEditor
                colorMode={hostColorMode}
                labels={labels}
                name={name}
                onChange={onDraftChange}
                onSave={onSave}
                saveError={saveError}
                saveState={saveState}
                value={draft ?? content.text}
              />
            ) : (
              <CodeFilePreview
                colorMode={hostColorMode}
                labels={labels}
                name={name}
                text={draft ?? content.text}
              />
            )}
          </React.Suspense>
        </PreviewErrorBoundary>
      ) : (
        unsupportedPanel
      );
    default:
      return unsupportedPanel;
  }
}
