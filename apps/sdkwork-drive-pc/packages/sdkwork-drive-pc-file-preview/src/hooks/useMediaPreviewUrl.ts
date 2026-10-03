import { useEffect, useMemo, useState } from 'react';

export interface MediaPreviewSource {
  /** 宿主直链；提供时优先使用，不构造 blob。 */
  url?: string;
  bytes?: Uint8Array;
  contentType?: string;
}

/**
 * 把「直链或字节」统一成一个可寻址 URL，并在卸载/替换时释放。
 *
 * 媒体与 PDF 的原生元素只认 URL，而管理端只能拿到 base64 字节，所以转换发生在
 * 这里而不是每个预览组件里各写一次；blob URL 的释放也一并在同一处收口，避免长
 * 会话里预览几十个文件后把内存占满。
 */
export function useMediaPreviewUrl(source: MediaPreviewSource): string | undefined {
  const [blobUrl, setBlobUrl] = useState<string | undefined>(undefined);

  const identity = useMemo(() => {
    if (source.url) {
      return `url:${source.url}`;
    }
    if (source.bytes) {
      // 字节内容可能很大，身份只取长度 + 首尾字节，够区分不同预览目标即可。
      const head = source.bytes[0] ?? 0;
      const tail = source.bytes[source.bytes.length - 1] ?? 0;
      return `bytes:${source.bytes.byteLength}:${head}:${tail}:${source.contentType ?? ''}`;
    }
    return 'empty';
  }, [source.bytes, source.contentType, source.url]);

  useEffect(() => {
    if (source.url || !source.bytes) {
      setBlobUrl(undefined);
      return undefined;
    }
    const buffer = new ArrayBuffer(source.bytes.byteLength);
    new Uint8Array(buffer).set(source.bytes);
    const objectUrl = URL.createObjectURL(
      new Blob([buffer], { type: source.contentType ?? 'application/octet-stream' }),
    );
    setBlobUrl(objectUrl);
    return () => {
      URL.revokeObjectURL(objectUrl);
    };
    // identity 覆盖了 source 的内容特征，比逐字段依赖更稳定（字节数组每次渲染都是新引用）。
  }, [identity, source.bytes, source.contentType, source.url]);

  return source.url ?? blobUrl;
}
