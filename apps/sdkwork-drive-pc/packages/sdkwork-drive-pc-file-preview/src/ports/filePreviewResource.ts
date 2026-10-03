/**
 * 宿主与预览组件之间的唯一内容边界。
 *
 * 预览组件不认 SDK、不认对象存储、不认 drive 会话，它只知道「怎么拿到这个文件的
 * 内容」。管理端用后端 8 MiB 内容接口实现它，drive 控制台可以用预签名直链实现它，
 * 两边挂的是同一批预览组件。
 */

export interface FilePreviewReadOptions {
  signal?: AbortSignal;
}

export interface FilePreviewResource {
  /** 展示用文件名（含扩展名，可含目录前缀）。 */
  name: string;
  contentType?: string;
  sizeBytes?: number;
  /**
   * 读取原始字节。二进制预览（Office / 压缩包）与图片兜底走它。
   */
  readBytes?: (options?: FilePreviewReadOptions) => Promise<Uint8Array>;
  /**
   * 直接可用的对象地址（预签名或 blob URL）。
   *
   * 媒体与 PDF 优先走它：浏览器原生 `<video>`/`<audio>`/`<iframe>` 需要可寻址的
   * URL，而 base64 往返会把整个文件读进内存。
   */
  resolveUrl?: (options?: FilePreviewReadOptions) => Promise<string>;
  /** 文本读取。缺省时宿主若提供 `readBytes`，预览会自行按 UTF-8 解码。 */
  readText?: (options?: FilePreviewReadOptions) => Promise<string>;
  /** 文本写回（编辑器保存）。缺省即只读。 */
  saveText?: (text: string, options?: FilePreviewReadOptions) => Promise<void>;
  /**
   * 下载入口；缺省时不渲染下载动作。
   *
   * 允许返回 Promise：下载也会失败（对象超过接口上限、直链过期）。宿主抛出的错误由
   * Surface 捕获并显示在工具条上——静默失败会让人以为"点了没反应"。
   */
  download?: () => void | Promise<void>;
  /**
   * 内联传输上限（字节）。超过它的文件不下载到浏览器，而是给出明确提示与下载入口；
   * 缺省 `DEFAULT_INLINE_PREVIEW_LIMIT_BYTES`。只约束 `readBytes`/`readText`，
   * 因为 `resolveUrl` 是宿主自己的通道，其容量由宿主决定。
   */
  inlineLimitBytes?: number;
}

/** 与管理端对象内容接口一致的默认上限。 */
export const DEFAULT_INLINE_PREVIEW_LIMIT_BYTES = 8 * 1024 * 1024;

/** 预览正文的加载状态机（Surface 与各预览组件共用的输入）。 */
export type FilePreviewContent =
  | { status: 'loading' }
  | { status: 'text'; text: string }
  | { status: 'bytes'; bytes: Uint8Array }
  | { status: 'url'; url: string }
  | { status: 'empty' }
  | { status: 'too-large'; sizeBytes: number; limitBytes: number }
  | { status: 'unavailable'; reason: 'no-reader' | 'not-previewable' }
  | { status: 'error'; message: string };

/** 已就绪的内容（渲染分支用不到 loading/error 时收窄类型）。 */
export type ReadyFilePreviewContent = Extract<
  FilePreviewContent,
  { status: 'text' | 'bytes' | 'url' | 'empty' }
>;
