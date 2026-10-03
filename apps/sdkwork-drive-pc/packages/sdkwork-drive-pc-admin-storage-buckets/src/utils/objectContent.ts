import { base64ToBytes, bytesToBase64, decodeUtf8Text } from 'sdkwork-drive-pc-file-preview';
import { MAX_OBJECT_CONTENT_BYTES, objectContentRequestBytes } from 'sdkwork-drive-pc-commons';
import type {
  StorageProviderAdminService,
  StorageProviderObjectView,
} from 'sdkwork-drive-pc-admin-storage-providers';

/**
 * 单次对象内容读写的字节上限（8 MiB），与后端 `storageProviders.objects.content.*` 一致。
 *
 * 常量本体定义在 commons：存储对象浏览器与预览层读的是同一个值，避免"这里能传、那里
 * 传不上去"。这里重新导出，保持本包既有导入路径不变。
 */
export { MAX_OBJECT_CONTENT_BYTES };

/**
 * 同一个上限对应的"网线体积"（base64 + JSON 信封，约 11.2 MB）。
 *
 * 请求体上限比业务上限更早生效，所以错误文案必须同时给出这两个数字，否则用户会把
 * `Payload too large` 理解成"对象超过 8 MiB"。
 */
export { objectContentRequestBytes };

export interface ObjectContentScope {
  bucket: string;
  /**
   * 桶所在地域（桶清单那一行的 `region`）。
   *
   * 账号级桶清单跨地域，读文件必须按这个桶自己的地域找端点；缺省时服务端仍用配置
   * 里写的地域，跨地域的桶就会打不开。
   */
  region?: string;
  providerId: string;
  service: StorageProviderAdminService;
}

/** 读原始字节：内容接口返回 base64，这里是唯一解码处。 */
export async function readObjectBytes(
  scope: ObjectContentScope,
  objectKey: string,
  signal?: AbortSignal,
): Promise<Uint8Array> {
  const content = await scope.service.readObjectContent(scope.providerId, objectKey, {
    bucket: scope.bucket,
    region: scope.region,
    signal,
  });
  return base64ToBytes(content.content);
}

export async function readObjectText(
  scope: ObjectContentScope,
  objectKey: string,
  signal?: AbortSignal,
): Promise<string> {
  const content = await scope.service.readObjectContent(scope.providerId, objectKey, {
    bucket: scope.bucket,
    region: scope.region,
    signal,
  });
  return decodeUtf8Text(base64ToBytes(content.content));
}

/** 写回文本：编辑器保存走这里，编码固定 utf8（内容接口的默认编码）。 */
export async function writeObjectText(
  scope: ObjectContentScope,
  objectKey: string,
  text: string,
  contentType?: string,
): Promise<void> {
  await scope.service.writeObjectContent(
    scope.providerId,
    objectKey,
    {
      content: text,
      encoding: 'utf8',
      ...(contentType ? { contentType } : {}),
    },
    { bucket: scope.bucket, region: scope.region },
  );
}

export async function writeObjectBytes(
  scope: ObjectContentScope,
  objectKey: string,
  bytes: Uint8Array,
  contentType?: string,
): Promise<void> {
  await scope.service.writeObjectContent(
    scope.providerId,
    objectKey,
    {
      content: bytesToBase64(bytes),
      encoding: 'base64',
      ...(contentType ? { contentType } : {}),
    },
    { bucket: scope.bucket, region: scope.region },
  );
}

/**
 * 浏览器内下载。
 *
 * 延迟 60 秒再释放 object URL：部分浏览器在同步 revoke 之后会中断尚未开始的下载，
 * 这是「下载了但文件是 0 字节」这类问题的根因。
 */
export async function downloadObjectBytes(
  scope: ObjectContentScope,
  object: StorageProviderObjectView,
): Promise<void> {
  const content = await scope.service.readObjectContent(scope.providerId, object.key, {
    bucket: scope.bucket,
    region: scope.region,
  });
  const bytes = base64ToBytes(content.content);
  const buffer = new ArrayBuffer(bytes.byteLength);
  new Uint8Array(buffer).set(bytes);
  const blob = new Blob([buffer], {
    type: content.contentType ?? object.contentType ?? 'application/octet-stream',
  });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement('a');
  anchor.href = url;
  anchor.download = object.key.split('/').filter(Boolean).at(-1) ?? object.key;
  document.body.appendChild(anchor);
  anchor.click();
  anchor.remove();
  window.setTimeout(() => URL.revokeObjectURL(url), 60_000);
}
