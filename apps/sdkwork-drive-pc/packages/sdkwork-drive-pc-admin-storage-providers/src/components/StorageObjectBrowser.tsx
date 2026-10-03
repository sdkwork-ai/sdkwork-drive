import React, { useCallback, useEffect, useRef, useState } from 'react';
import {
  ArrowLeft,
  CircleAlert,
  FolderOpen,
  FolderPlus,
  LoaderCircle,
  RefreshCw,
  Upload,
} from 'lucide-react';
import type { StorageProviderAdminService } from '../services/storageProviderAdminService';
import type { StorageProviderObjectView, StorageProviderView } from '../types/storageProviderAdminTypes';
import { formatDriveBytes, MAX_OBJECT_CONTENT_BYTES } from 'sdkwork-drive-pc-commons';
import { formatDriveDateTime } from '../utils/formatDriveTimestamp';
import { formatMutationError } from '../utils/mutationError';
import { fileNameOf, parentPrefixOf } from '../utils/objectKeyUtils';
import { useTranslation } from '../hooks/useTranslation';
import { ConfirmDialog } from './ConfirmDialog';
import {
  GHOST_BUTTON_CLASS,
  INPUT_CLASS,
  PRIMARY_BUTTON_CLASS,
  SECONDARY_BUTTON_CLASS,
} from '../utils/uiPrimitives';

/**
 * 上传大小上限：与服务端对象内容写入上限一致，取自 commons 的单一定义
 * （`MAX_OBJECT_CONTENT_BYTES`）。桶浏览器读的是同一个常量，两边不会再漂移。
 */
const MAX_UPLOAD_BYTES = MAX_OBJECT_CONTENT_BYTES;

interface StorageObjectBrowserProps {
  provider: StorageProviderView;
  service: StorageProviderAdminService;
  /**
   * 打开时定位的前缀（默认为存储桶根）。
   *
   * 绑定管理把它设为该绑定的 `storageRootPrefix`，弹窗一打开就是该空间类型的子树，
   * 而不是整个存储桶；面包屑仍然从存储桶根算起，所以「根目录」始终是退出的出口。
   * 挂载后再改变该值会重新定位，调用方也可以换 `key` 强制重挂载。
   */
  initialPrefix?: string;
}

type PromptKind = 'newFolder' | 'rename';

interface ObjectPrompt {
  kind: PromptKind;
  objectKey?: string;
  initialValue: string;
}

export function StorageObjectBrowser({ provider, service, initialPrefix = '' }: StorageObjectBrowserProps) {
  const { t, language } = useTranslation();
  const [objects, setObjects] = useState<StorageProviderObjectView[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [currentPrefix, setCurrentPrefix] = useState(initialPrefix);
  const [pageToken, setPageToken] = useState<string | null>(null);
  const [hasMore, setHasMore] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<string | null>(null);
  const [prompt, setPrompt] = useState<ObjectPrompt | null>(null);
  const [promptValue, setPromptValue] = useState('');
  const [promptBusy, setPromptBusy] = useState(false);
  const [uploading, setUploading] = useState(false);
  const fileInputRef = useRef<HTMLInputElement | null>(null);
  /** 请求序号：loadMore 与导航并发时丢弃过期响应（防止旧前缀分页混入新列表）。 */
  const loadSeqRef = useRef(0);
  /** 已拉取过的目标（provider + 前缀），用于挂载后只自动加载一次。 */
  const loadedTargetRef = useRef<string | null>(null);

  const formatSize = formatDriveBytes;

  const loadObjects = useCallback(async (prefix: string, token?: string) => {
    const seq = ++loadSeqRef.current;
    setLoading(true);
    setError(null);
    try {
      const result = await service.listObjects(provider.id, {
        prefix,
        pageToken: token,
      });
      if (seq !== loadSeqRef.current) {
        return;
      }
      if (token) {
        setObjects((prev) => [...prev, ...result.items]);
      } else {
        setObjects(result.items);
      }
      setPageToken(result.nextPageToken || null);
      setHasMore(result.hasMore);
      setCurrentPrefix(prefix);
    } catch (err) {
      if (seq !== loadSeqRef.current) {
        return;
      }
      setError(formatMutationError(err, t('errorLoadObjects')));
    } finally {
      if (seq === loadSeqRef.current) {
        setLoading(false);
      }
    }
  }, [provider.id, service, t]);

  /**
   * 挂载即显示 `initialPrefix` 的内容。
   *
   * 守卫按「已拉取的目标」而不是回调标识去重：`t` 在宿主语言 Provider 每次重渲染时
   * 都会换新标识，`loadObjects` 随之换新，按标识做依赖会重复请求。同一 provider 同一
   * 前缀只自动加载一次，之后交给刷新按钮或换 `key` 重挂载。
   */
  useEffect(() => {
    const target = `${provider.id}\u0000${initialPrefix}`;
    if (loadedTargetRef.current === target) {
      return;
    }
    loadedTargetRef.current = target;
    void loadObjects(initialPrefix);
  }, [initialPrefix, loadObjects, provider.id]);

  const navigateToFolder = (prefix: string) => {
    loadObjects(prefix);
  };

  const navigateUp = () => {
    loadObjects(parentPrefixOf(currentPrefix));
  };

  const openPrompt = (next: ObjectPrompt) => {
    setPromptValue(next.initialValue);
    setPrompt(next);
  };

  const submitPrompt = async () => {
    if (!prompt) return;
    const value = promptValue.trim();
    if (!value) return;
    setPromptBusy(true);
    setError(null);
    try {
      if (prompt.kind === 'newFolder') {
        await service.writeObjectContent(provider.id, `${currentPrefix}${value}/`, {
          content: '',
        });
      } else if (prompt.objectKey) {
        const destination = `${parentPrefixOf(prompt.objectKey)}${value}`;
        if (destination === prompt.objectKey) {
          setPrompt(null);
          return;
        }
        await service.renameObject(provider.id, prompt.objectKey, destination);
      }
      setPrompt(null);
      await loadObjects(currentPrefix);
    } catch (err) {
      setError(formatMutationError(err, prompt.kind === 'newFolder' ? t('newFolderError') : t('renameError')));
    } finally {
      setPromptBusy(false);
    }
  };

  const uploadFile = async (file: File) => {
    if (file.size > MAX_UPLOAD_BYTES) {
      setError(t('fileTooLarge'));
      if (fileInputRef.current) {
        fileInputRef.current.value = '';
      }
      return;
    }
    setUploading(true);
    setError(null);
    try {
      const base64 = await readFileAsBase64(file);
      await service.writeObjectContent(provider.id, `${currentPrefix}${file.name}`, {
        content: base64,
        encoding: 'base64',
        ...(file.type ? { contentType: file.type } : {}),
      });
      await loadObjects(currentPrefix);
    } catch (err) {
      setError(formatMutationError(err, t('uploadError')));
    } finally {
      setUploading(false);
      if (fileInputRef.current) {
        fileInputRef.current.value = '';
      }
    }
  };

  const downloadObject = async (object: StorageProviderObjectView) => {
    setError(null);
    try {
      const content = await service.readObjectContent(provider.id, object.key);
      const bytes = base64ToBytes(content.content);
      const arrayBuffer = new ArrayBuffer(bytes.byteLength);
      new Uint8Array(arrayBuffer).set(bytes);
      const blob = new Blob(
        [arrayBuffer],
        { type: content.contentType ?? 'application/octet-stream' },
      );
      const url = URL.createObjectURL(blob);
      const anchor = document.createElement('a');
      anchor.href = url;
      anchor.download = fileNameOf(object.key);
      document.body.appendChild(anchor);
      anchor.click();
      anchor.remove();
      // 延迟释放：部分浏览器在同步 revoke 后会中断下载。
      window.setTimeout(() => URL.revokeObjectURL(url), 60_000);
    } catch (err) {
      setError(formatMutationError(err, t('downloadError')));
    }
  };

  const deleteObject = useCallback(async (key: string) => {
    setDeleteTarget(null);
    setLoading(true);
    setError(null);
    try {
      await service.deleteObject(provider.id, key);
      await loadObjects(currentPrefix);
    } catch (err) {
      setError(formatMutationError(err, t('errorDeleteObject')));
    } finally {
      setLoading(false);
    }
  }, [provider.id, currentPrefix, service, loadObjects, t]);

  const breadcrumbSegments = currentPrefix.split('/').filter(Boolean);

  return (
    <>
    <section className="rounded-lg border border-neutral-200 bg-white shadow-sm dark:border-neutral-700 dark:bg-neutral-900">
      <div className="flex flex-wrap items-center gap-2 border-b border-neutral-100 px-5 py-3 dark:border-neutral-800">
        <FolderOpen aria-hidden="true" className="shrink-0 text-neutral-400" size={15} />
        <h3 className="shrink-0 text-sm font-semibold">{t('files')}</h3>

        <span className="mx-1 hidden h-5 w-px shrink-0 bg-neutral-200 dark:bg-neutral-700 sm:block" />

        <nav aria-label={t('files')} className="flex min-w-0 flex-1 flex-wrap items-center gap-0.5 text-xs">
          <button
            type="button"
            onClick={() => loadObjects('')}
            disabled={loading || uploading}
            className="rounded px-1.5 py-1 font-medium text-neutral-600 transition-colors hover:bg-neutral-100 hover:text-neutral-900 disabled:cursor-not-allowed disabled:opacity-50 dark:text-neutral-300 dark:hover:bg-neutral-800 dark:hover:text-white"
          >
            {t('root')}
          </button>
          {breadcrumbSegments.map((segment, index) => (
            <React.Fragment key={`${segment}-${index}`}>
              <span aria-hidden="true" className="text-neutral-400">/</span>
              <button
                type="button"
                className="max-w-[12rem] truncate rounded px-1.5 py-1 font-mono text-blue-600 transition-colors hover:bg-blue-50 disabled:cursor-not-allowed disabled:opacity-50 dark:text-blue-400 dark:hover:bg-blue-950/30"
                disabled={loading}
                onClick={() => loadObjects(breadcrumbSegments.slice(0, index + 1).join('/') + '/')}
              >
                {segment}
              </button>
            </React.Fragment>
          ))}
        </nav>

        <div className="flex shrink-0 flex-wrap items-center gap-2">
          {currentPrefix ? (
            <button type="button" onClick={navigateUp} disabled={loading || uploading} className={GHOST_BUTTON_CLASS}>
              <ArrowLeft aria-hidden="true" size={14} />
              {t('up')}
            </button>
          ) : null}
          <button
            type="button"
            onClick={() => openPrompt({ kind: 'newFolder', initialValue: '' })}
            disabled={loading || uploading}
            className={SECONDARY_BUTTON_CLASS}
          >
            <FolderPlus aria-hidden="true" size={14} />
            {t('newFolder')}
          </button>
          <label className={`${PRIMARY_BUTTON_CLASS} cursor-pointer`} aria-disabled={uploading}>
            {uploading ? (
              <LoaderCircle aria-hidden="true" className="animate-spin" size={14} />
            ) : (
              <Upload aria-hidden="true" size={14} />
            )}
            {uploading ? t('uploading') : t('upload')}
            <input
              ref={fileInputRef}
              className="hidden"
              type="file"
              disabled={uploading}
              onChange={(event) => {
                const file = event.target.files?.[0];
                if (file) void uploadFile(file);
              }}
            />
          </label>
          <button
            type="button"
            onClick={() => loadObjects(currentPrefix)}
            disabled={loading || uploading}
            className={GHOST_BUTTON_CLASS}
            aria-label={t('refresh')}
            title={t('refresh')}
          >
            <RefreshCw aria-hidden="true" className={loading ? 'animate-spin' : undefined} size={14} />
          </button>
        </div>
      </div>

      <div className="px-5 py-4">
        {error && (
          <div className="mb-3 flex items-start gap-2 rounded-md border border-red-200 bg-red-50 px-3 py-2 text-xs text-red-700 dark:border-red-900 dark:bg-red-950/20 dark:text-red-300">
            <CircleAlert aria-hidden="true" className="mt-0.5 shrink-0" size={14} />
            <span className="flex-1">{error}</span>
          </div>
        )}

        {objects.length === 0 && !loading ? (
          <div className="flex flex-col items-center justify-center gap-1.5 rounded-md border border-dashed border-neutral-200 py-10 text-center dark:border-neutral-700">
            <FolderOpen aria-hidden="true" className="text-neutral-300 dark:text-neutral-600" size={26} />
            <p className="text-xs text-neutral-500">{t('empty')}</p>
            <p className="text-[11px] text-neutral-400">{t('emptyFolderHint')}</p>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-xs">
              <thead>
                <tr className="border-b border-neutral-200 text-left text-neutral-500 dark:border-neutral-700">
                  <th className="py-2 pr-4 font-medium">{t('nameHeader')}</th>
                  <th className="py-2 pr-4 font-medium">{t('sizeHeader')}</th>
                  <th className="py-2 pr-4 font-medium">{t('modifiedHeader')}</th>
                  <th className="py-2 text-right font-medium">{t('actHeader')}</th>
                </tr>
              </thead>
              <tbody>
                {objects.map((obj) => (
                  <tr key={obj.key} className="border-b border-neutral-100 transition-colors last:border-0 hover:bg-neutral-50 dark:border-neutral-800 dark:hover:bg-neutral-800/50">
                    <td className="py-2 pr-4 font-mono">
                      {obj.isFolder ? (
                        <button
                          type="button"
                          onClick={() => navigateToFolder(obj.key)}
                          className="inline-flex max-w-full items-center gap-1.5 text-blue-600 hover:underline dark:text-blue-400"
                        >
                          <FolderOpen aria-hidden="true" className="shrink-0" size={13} />
                          <span className="truncate">{fileNameOf(obj.key)}/</span>
                        </button>
                      ) : (
                        <span className="truncate">{fileNameOf(obj.key)}</span>
                      )}
                    </td>
                    <td className="py-2 pr-4 tabular-nums text-neutral-600 dark:text-neutral-300">
                      {obj.isFolder ? '—' : formatSize(obj.sizeBytes)}
                    </td>
                    <td className="py-2 pr-4 text-neutral-400">
                      {obj.lastModifiedIso ? formatDriveDateTime(obj.lastModifiedIso, language) : '—'}
                    </td>
                    <td className="py-2">
                      <div className="flex items-center justify-end gap-3">
                        {!obj.isFolder && obj.sizeBytes > MAX_UPLOAD_BYTES && (
                          <span className="text-[11px] text-neutral-400" title={t('fileTooLarge')}>
                            {t('fileTooLarge')}
                          </span>
                        )}
                        {!obj.isFolder && obj.sizeBytes <= MAX_UPLOAD_BYTES && (
                          <button
                            type="button"
                            onClick={() => void downloadObject(obj)}
                            disabled={loading}
                            className="text-blue-600 hover:underline disabled:opacity-50 dark:text-blue-400"
                          >
                            {t('download')}
                          </button>
                        )}
                        {!obj.isFolder && (
                          <button
                            type="button"
                            onClick={() => openPrompt({ kind: 'rename', objectKey: obj.key, initialValue: fileNameOf(obj.key) })}
                            disabled={loading}
                            className="text-neutral-600 hover:underline disabled:opacity-50 dark:text-neutral-300"
                          >
                            {t('rename')}
                          </button>
                        )}
                        {!obj.isFolder && (
                          <button
                            type="button"
                            onClick={() => setDeleteTarget(obj.key)}
                            disabled={loading}
                            className="text-red-600 hover:underline disabled:opacity-50 dark:text-red-400"
                          >
                            {t('del')}
                          </button>
                        )}
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}

        {hasMore && (
          <button
            type="button"
            onClick={() => loadObjects(currentPrefix, pageToken ?? undefined)}
            disabled={loading}
            className="mt-3 text-xs font-medium text-blue-600 hover:underline disabled:opacity-50 dark:text-blue-400"
          >
            {t('loadMore')}
          </button>
        )}
      </div>
    </section>
    {prompt ? (
      <div
        className="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
        role="presentation"
        onPointerDown={(event) => {
          if (event.target === event.currentTarget && !promptBusy) {
            setPrompt(null);
          }
        }}
      >
        <div
          aria-modal="true"
          className="w-full max-w-sm rounded-lg border border-neutral-200 bg-white p-4 shadow-xl dark:border-neutral-800 dark:bg-[#1b1b1b]"
          role="dialog"
          aria-labelledby="storage-object-prompt-title"
        >
          <h3 className="mb-3 text-sm font-medium text-neutral-800 dark:text-neutral-100" id="storage-object-prompt-title">
            {prompt.kind === 'newFolder' ? t('newFolderDialogTitle') : t('renameDialogTitle')}
          </h3>
          <input
            autoFocus
            className={INPUT_CLASS}
            value={promptValue}
            placeholder={prompt.kind === 'newFolder' ? t('folderNamePlaceholder') : t('newNamePlaceholder')}
            onChange={(event) => setPromptValue(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === 'Enter' && !promptBusy) void submitPrompt();
              if (event.key === 'Escape' && !promptBusy) setPrompt(null);
            }}
          />
          <div className="mt-4 flex justify-end gap-2">
            <button type="button" className={SECONDARY_BUTTON_CLASS} disabled={promptBusy} onClick={() => setPrompt(null)}>
              {t('cancel')}
            </button>
            <button type="button" className={PRIMARY_BUTTON_CLASS} disabled={promptBusy || !promptValue.trim()} onClick={() => void submitPrompt()}>
              {promptBusy ? t('saving') : t('confirm')}
            </button>
          </div>
        </div>
      </div>
    ) : null}
    <ConfirmDialog
      busy={loading}
      confirmLabel={t('del')}
      message={t('deleteObjectConfirm', { key: deleteTarget ?? '' })}
      onCancel={() => setDeleteTarget(null)}
      onConfirm={() => { if (deleteTarget) void deleteObject(deleteTarget); }}
      open={deleteTarget !== null}
      title={t('del')}
      variant="danger"
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

function base64ToBytes(base64: string): Uint8Array {
  const binary = window.atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let index = 0; index < binary.length; index += 1) {
    bytes[index] = binary.charCodeAt(index);
  }
  return bytes;
}
