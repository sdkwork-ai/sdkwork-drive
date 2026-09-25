export { STORAGE_PROVIDER_ADMIN_CAPABILITY } from './capability';
export type { StorageProviderAdminService } from './services/storageProviderAdminService';
export { createStorageProviderAdminService } from './services/storageProviderAdminService';
export { StorageObjectBrowser } from './components/StorageObjectBrowser';
export { StorageOverviewAdminPage } from './pages/StorageOverviewAdminPage';
export { StorageProvidersAdminPage } from './pages/StorageProvidersAdminPage';
export { StorageBindingsAdminPage } from './pages/StorageBindingsAdminPage';
export { StorageProviderKindsAdminPage } from './pages/StorageProviderKindsAdminPage';
export { StorageBucketsAdminPage } from './pages/StorageBucketsAdminPage';
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
export type {
  AccountScopeFilter,
  CredentialSource,
} from './components/StorageProviderCredentialFields';
export {
  getProviderKindMeta,
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
  StorageProviderBucketView,
  StorageProviderBucketListItemView,
  StorageProviderCapabilitiesView,
  StorageProviderObjectView,
  StorageProviderObjectContentView,
  StorageProviderObjectMutationResult,
  WriteStorageProviderObjectContentInput,
  CopyStorageProviderObjectInput,
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
  CreateStorageProviderAccountInput,
  ListStorageProviderAccountsInput,
  StorageProviderAccountDefaultView,
  StorageProviderAccountScope,
  StorageProviderAccountView,
  StorageProviderKind,
} from './types/storageProviderAdminTypes';
