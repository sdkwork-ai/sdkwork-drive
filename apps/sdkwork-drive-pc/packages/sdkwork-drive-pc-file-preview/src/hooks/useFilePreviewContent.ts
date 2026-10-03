import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import {
  DEFAULT_INLINE_PREVIEW_LIMIT_BYTES,
  type FilePreviewContent,
  type FilePreviewResource,
} from '../ports/filePreviewResource';
import {
  isBinaryKind,
  isPreviewableKind,
  isTextLikeKind,
  prefersDirectUrl,
  type FilePreviewKind,
} from '../kinds/filePreviewKind';
import { decodeUtf8Text, looksLikeBinary } from '../utils/bytes';

export interface UseFilePreviewContentOptions {
  /** 是否立即加载；关闭时内容停在 `loading`，由调用方决定何时开始。 */
  enabled?: boolean;
  kind: FilePreviewKind;
  resource: FilePreviewResource;
}

export interface FilePreviewContentHandle {
  content: FilePreviewContent;
  /** 重新加载（错误重试、保存后刷新）。 */
  reload: () => void;
}

function isAbortError(error: unknown): boolean {
  return (
    typeof error === 'object'
    && error !== null
    && 'name' in error
    && (error as { name?: unknown }).name === 'AbortError'
  );
}

function errorMessage(error: unknown): string {
  if (error instanceof Error && error.message.trim() !== '') {
    return error.message;
  }
  if (typeof error === 'string' && error.trim() !== '') {
    return error;
  }
  // 空串表示「无法归因」：Surface 会用宿主语言渲染通用失败文案，而不是露出英文开发者串。
  return '';
}

/**
 * 按种类与宿主能力选择读取通道，产出预览正文。
 *
 * 选择顺序是有意为之：
 * 1. 媒体/PDF 优先直链（`resolveUrl`）——浏览器原生播放器需要可寻址 URL，整文件
 *    base64 会让内存翻倍；
 * 2. 文本类先试 `readText`，没有再退回字节 + UTF-8 解码；
 * 3. 其余（Office、压缩包、图片兜底）读字节。
 *
 * 内联传输前先按 `inlineLimitBytes` 判定：超过上限的文件直接进 `too-large`，而不是
 * 先下载几十兆再失败——这正是管理端 8 MiB 内容接口下最常见的真实情况。
 */
export function useFilePreviewContent({
  enabled = true,
  kind,
  resource,
}: UseFilePreviewContentOptions): FilePreviewContentHandle {
  const [content, setContent] = useState<FilePreviewContent>({ status: 'loading' });
  const [reloadToken, setReloadToken] = useState(0);
  const resourceRef = useRef(resource);
  resourceRef.current = resource;

  const identity = useMemo(
    () => `${resource.name}\u0000${resource.contentType ?? ''}\u0000${resource.sizeBytes ?? ''}\u0000${kind}`,
    [kind, resource.contentType, resource.name, resource.sizeBytes],
  );

  const reload = useCallback(() => setReloadToken((token) => token + 1), []);

  useEffect(() => {
    if (!enabled) {
      return undefined;
    }
    const current = resourceRef.current;
    const controller = new AbortController();
    let active = true;

    const emit = (next: FilePreviewContent) => {
      if (active) {
        setContent(next);
      }
    };

    const limitBytes = current.inlineLimitBytes ?? DEFAULT_INLINE_PREVIEW_LIMIT_BYTES;
    const needsInlineTransfer =
      !prefersDirectUrl(kind) || typeof current.resolveUrl !== 'function';
    if (
      needsInlineTransfer
      && current.sizeBytes !== undefined
      && current.sizeBytes > limitBytes
      && (typeof current.readBytes === 'function' || typeof current.readText === 'function')
    ) {
      emit({ status: 'too-large', sizeBytes: current.sizeBytes, limitBytes });
      return () => {
        active = false;
      };
    }

    if (!isPreviewableKind(kind)) {
      emit({ status: 'unavailable', reason: 'not-previewable' });
      return () => {
        active = false;
      };
    }

    const load = async (): Promise<FilePreviewContent> => {
      const options = { signal: controller.signal };

      if (prefersDirectUrl(kind) && typeof current.resolveUrl === 'function') {
        const url = await current.resolveUrl(options);
        return url ? { status: 'url', url } : { status: 'unavailable', reason: 'no-reader' };
      }

      if (isTextLikeKind(kind)) {
        if (typeof current.readText === 'function') {
          const text = await current.readText(options);
          return text === '' ? { status: 'empty' } : { status: 'text', text };
        }
        if (typeof current.readBytes === 'function') {
          const bytes = await current.readBytes(options);
          if (bytes.byteLength === 0) {
            return { status: 'empty' };
          }
          // 扩展名说是文本，内容却像二进制：不把它渲染成乱码。
          if (looksLikeBinary(bytes)) {
            return { status: 'unavailable', reason: 'no-reader' };
          }
          return { status: 'text', text: decodeUtf8Text(bytes) };
        }
        return { status: 'unavailable', reason: 'no-reader' };
      }

      if (typeof current.readBytes === 'function') {
        const bytes = await current.readBytes(options);
        if (bytes.byteLength === 0) {
          return { status: 'empty' };
        }
        return { status: 'bytes', bytes };
      }

      if (typeof current.readText === 'function' && isBinaryKind(kind)) {
        return { status: 'unavailable', reason: 'no-reader' };
      }

      return { status: 'unavailable', reason: 'no-reader' };
    };

    setContent({ status: 'loading' });
    load()
      .then(emit)
      .catch((error: unknown) => {
        if (isAbortError(error)) {
          return;
        }
        emit({ status: 'error', message: errorMessage(error) });
      });

    return () => {
      active = false;
      controller.abort();
    };
  }, [enabled, identity, kind, reloadToken]);

  return { content, reload };
}
