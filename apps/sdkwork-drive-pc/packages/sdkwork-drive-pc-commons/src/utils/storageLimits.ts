/**
 * 存储平面共享的数值上限。
 *
 * 放在 commons 是为了让「谁都能读、只有一处定义」：管理端的对象浏览器、存储桶文件弹窗
 * 与预览层都受同一个后端上限约束（`storageProviders.objects.content.*` 单次读写 8 MiB）。
 * 以前这个数字在三个包里各写了一遍，任何一处改动都会造成"这里能传、那里传不上去"的
 * 割裂体验。
 */

/**
 * 单次对象内容读写上限（8 MiB），与后端内容接口一致。
 *
 * 超过它的对象：浏览器内既不读也不写，界面必须明确说明并指向厂商控制台，而不是让请求
 * 以 413 结束。
 */
export const MAX_OBJECT_CONTENT_BYTES = 8 * 1024 * 1024;

/**
 * 一个对象内容写入请求在网线上的实际体积（字节）。
 *
 * 内容接口的请求体是 JSON，对象字节以 **base64** 承载，所以请求体不是"文件大小"，而是
 * `ceil(字节数 / 3) * 4` 再加 JSON 信封。这个换算是 413 的真正来源：8 MiB 的对象在网线上
 * 是约 11.2 MB，任何低于它的请求体上限（例如网关默认的 10 MiB）都会让"客户端允许、
 * 服务端拒绝"（用户看到的 `Payload too large`）。
 *
 * 用途：客户端自检与错误文案都要按这个体积说话，而不是按文件大小。
 */
export function objectContentRequestBytes(decodedBytes: number): number {
  if (!Number.isFinite(decodedBytes) || decodedBytes <= 0) {
    // 空对象（目录占位）仍有信封开销，给一个保守的下界。
    return 256;
  }
  const base64Bytes = Math.ceil(decodedBytes / 3) * 4;
  // 信封（字段名、contentType、引号等）实测在几百字节量级；留 1 KiB 余量。
  return base64Bytes + 1024;
}

/**
 * 全部落到同一个上限上的三个数字，供界面解释"为什么这个文件传不上去"。
 *
 * `requestBytes` 是网线体积，`decodedBytes` 是对象本身的字节数；两者必须一起展示，
 * 否则用户会以为"8 MiB 的文件，请求体也该是 8 MiB"。
 */
export function describeObjectContentLimit(decodedBytes = MAX_OBJECT_CONTENT_BYTES): {
  decodedBytes: number;
  requestBytes: number;
} {
  return { decodedBytes, requestBytes: objectContentRequestBytes(decodedBytes) };
}
