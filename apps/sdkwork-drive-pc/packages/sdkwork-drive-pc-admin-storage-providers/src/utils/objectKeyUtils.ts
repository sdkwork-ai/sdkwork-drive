/**
 * 对象 key 的展示派生值。
 *
 * 存储桶浏览（`StorageObjectBrowser`）与桶文件管理弹窗（
 * `sdkwork-drive-pc-admin-storage-buckets`）渲染的是同一套 key，所以「取末段文件名」
 * 与「取父前缀」只在这里实现一次，而不是每个浏览器各写一份。
 */

/** key 的末段；`a/b/c.txt` → `c.txt`（文件夹前缀 `a/b/` → `b`）。 */
export function fileNameOf(objectKey: string): string {
  const segments = objectKey.split('/').filter(Boolean);
  return segments.at(-1) ?? objectKey;
}

/** key 所在的前缀；`a/b/c.txt` → `a/b/`，根下的对象 → `''`。 */
export function parentPrefixOf(objectKey: string): string {
  const segments = objectKey.split('/').filter(Boolean);
  segments.pop();
  return segments.length > 0 ? `${segments.join('/')}/` : '';
}

/**
 * 相对当前目录的展示路径。
 *
 * 列表接口按 `delimiter=/` 返回「当前前缀 + 一段名字」，但历史实现里也出现过完整
 * key，所以渲染前统一裁掉当前前缀，避免出现 `docs/docs/a.txt` 这种重复。
 */
export function relativeObjectPath(objectKey: string, currentPrefix: string): string {
  return currentPrefix && objectKey.startsWith(currentPrefix)
    ? objectKey.slice(currentPrefix.length)
    : objectKey;
}
