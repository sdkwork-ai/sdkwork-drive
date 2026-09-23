import type { StorageProviderAccountDefaultView } from '../types/storageProviderAdminTypes';

/**
 * 初始化结果的折叠计数，直接喂给 `noticeAccountsInitialized` 的提示参数。
 *
 * `total` 是本次被检查的服务商行数，其余三个是"本次真正新建了什么"。
 */
export interface ProviderAccountDefaultsSummary {
  total: number;
  providers: number;
  accounts: number;
  credentials: number;
}

/**
 * 把服务端逐行的初始化结果折叠成提示计数。
 *
 * 刻意提成纯函数而不是写在页面组件里：最需要被测到的是**全零分支**——重复点击
 * 初始化时三个计数必须都归零，那是"没有覆盖运维已填真实密钥"这件事唯一的可见
 * 证据。挂在组件里的三行 `filter().length` 没有任何测试会走到，把渲染测试当门槛
 * 也抓不到它算错。
 */
export function summarizeProviderAccountDefaults(
  rows: readonly StorageProviderAccountDefaultView[],
): ProviderAccountDefaultsSummary {
  return {
    total: rows.length,
    providers: rows.filter((row) => row.providerCreated).length,
    accounts: rows.filter((row) => row.accountCreated).length,
    credentials: rows.filter((row) => row.credentialSeeded).length,
  };
}
