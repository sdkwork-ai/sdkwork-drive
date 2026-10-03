export { STORAGE_PROVIDER_ADMIN_CAPABILITY } from './capability';
export type { StorageProviderAdminService } from './services/storageProviderAdminService';
export { createStorageProviderAdminService } from './services/storageProviderAdminService';
export { StorageObjectBrowser } from './components/StorageObjectBrowser';
export { StorageProviderNavList } from './components/StorageProviderNavList';
// 翻译 hook 绑定的是 `storageProviders.` 这一份字典前缀，桶浏览弹窗读同一份字典，
// 所以从本包导出而不是在第二个包里再包一层前缀。
export { useTranslation } from './hooks/useTranslation';
// 服务商选项集（最大页 + 显式续页）同样是两个页面共用的读取方式。
export { PROVIDER_OPTIONS_PAGE_SIZE, useProviderOptions } from './hooks/useProviderOptions';
export type { ProviderOptionsState } from './hooks/useProviderOptions';
export { StorageOverviewAdminPage } from './pages/StorageOverviewAdminPage';
export { StorageProvidersAdminPage } from './pages/StorageProvidersAdminPage';
export { StorageBindingsAdminPage } from './pages/StorageBindingsAdminPage';
export { StorageProviderKindsAdminPage } from './pages/StorageProviderKindsAdminPage';
// 存储桶浏览已经独立成 `sdkwork-drive-pc-admin-storage-buckets`：桶列表、桶文件管理
// 弹窗与它的 CRUD 都在那个包里，这里只保留服务商配置自身的页面。
// 对象 key 的展示派生值由两个浏览器共用，所以从本包导出而不是各写一份。
export { fileNameOf, parentPrefixOf, relativeObjectPath } from './utils/objectKeyUtils';
export { formatMutationError, isPayloadTooLargeError } from './utils/mutationError';
// 时间显示只有一处实现：服务层只搬 ISO，格式化跟着宿主语言走，两个存储包共用。
export { formatDriveDate, formatDriveDateTime } from './utils/formatDriveTimestamp';
// 存储桶文件管理弹窗与绑定页共用同一套表格/按钮视觉，所以类名也走同一处导出。
export {
  BADGE_BASE_CLASS,
  CARD_BODY_CLASS,
  CARD_CLASS,
  CARD_HEADER_CLASS,
  DANGER_BUTTON_CLASS,
  GHOST_BUTTON_CLASS,
  ICON_BUTTON_CLASS,
  INPUT_CLASS,
  PRIMARY_BUTTON_CLASS,
  SECONDARY_BUTTON_CLASS,
  SELECT_CLASS,
} from './utils/uiPrimitives';
// The provider editor and its credential picker are exported so a host that
// already routes storage through this package does not grow a second, divergent
// form: cloudrouter reached the provider service from its own console while
// keeping a `plain | reference` credential switch of its own, which meant a
// storage provider configured there could never reference a reusable account.
// One editor means one credential model, and the account centre reaches every
// host that binds a provider. A host rendering these must mount
// `LanguageProvider` from `sdkwork-drive-pc-commons`, or every label falls back
// to its raw key.
export { StorageProviderEditor } from './components/StorageProviderEditor';
export { StorageProviderCredentialFields } from './components/StorageProviderCredentialFields';
export { StorageProviderAccountPickerDialog } from './components/StorageProviderAccountPickerDialog';
export { StorageProviderAccountBadges } from './components/StorageProviderAccountBadges';
export type { CredentialSource } from './components/StorageProviderCredentialFields';
export {
  getProviderKindMeta,
  providerDisplayName,
  providerRegionLabel,
  providerVendorCodeForKind,
  resolveProviderKindMeta,
} from './utils/providerKindConfig';
export type {
  ProviderCredentialFieldMeta,
  ProviderKindMeta,
} from './utils/providerKindConfig';
export {
  buildCredentialRef,
  isCredentialRefMasked,
  parseCredentialRef,
} from './utils/credentialRefUtils';
export type { CredentialInputMode } from './utils/credentialRefUtils';
export { summarizeProviderAccountDefaults } from './utils/providerAccountDefaultsSummary';
export type { ProviderAccountDefaultsSummary } from './utils/providerAccountDefaultsSummary';
export type {
  GetStorageOverviewInput,
  CreateStorageProviderInput,
  UpdateStorageProviderInput,
  StorageProviderView,
  StorageProviderKindView,
  StorageProviderBindingView,
  StorageBindingScope,
  ListStorageProviderBindingsInput,
  ListStorageProviderBindingsPageResult,
  StorageProviderBucketView,
  StorageProviderBucketListItemView,
  StorageProviderCapabilitiesView,
  StorageProviderObjectView,
  StorageProviderObjectContentView,
  StorageProviderObjectMutationResult,
  WriteStorageProviderObjectContentInput,
  CopyStorageProviderObjectInput,
  AbortStorageProviderMultipartUploadInput,
  CompleteStorageProviderMultipartUploadInput,
  CreateStorageProviderMultipartUploadInput,
  PresignStorageProviderUploadPartsInput,
  StorageProviderMultipartUploadView,
  StorageProviderUploadPartGrantView,
  StorageProviderUploadPartGrantsView,
  ListStorageProvidersInput,
  ListStorageProvidersPageResult,
  ListStorageProviderObjectsInput,
  ListStorageProviderObjectsResult,
  SetDefaultStorageProviderBindingInput,
  StorageOverviewView,
  StorageOverviewCapacityView,
  StorageOverviewBindingScopeCountsView,
  StorageOverviewBindingsView,
  StorageOverviewCatalogView,
  StorageOverviewProviderUsageView,
  StorageOverviewTrendPointView,
  StorageOverviewProvidersView,
  StorageProviderMutationOptions,
  StorageProviderObjectScopeOptions,
  CreateStorageProviderAccountInput,
  ListStorageProviderAccountsInput,
  ListStorageProviderAccountsPageResult,
  StorageProviderAccountDefaultView,
  StorageProviderAccountScope,
  StorageProviderAccountView,
  StorageProviderKind,
} from './types/storageProviderAdminTypes';
