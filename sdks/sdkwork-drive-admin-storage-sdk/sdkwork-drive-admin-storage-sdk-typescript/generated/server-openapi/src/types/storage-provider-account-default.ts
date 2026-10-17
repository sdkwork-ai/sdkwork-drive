import type { StorageProviderVendorCapabilityDefaults } from './storage-provider-vendor-capability-defaults';
import type { StorageProviderVendorCredentialFields } from './storage-provider-vendor-credential-fields';

export interface StorageProviderAccountDefault {
  /** Built-in storage provider kind this row bootstraps. */
  providerKind: string;
  /** Id of the provider configuration this kind bootstraps. */
  providerId: string;
  /** True when this call created the provider configuration. */
  providerCreated: boolean;
  /** Account-center vendor the provider is bound to; absent for a credential-free kind (local_filesystem). */
  vendorCode?: string;
  /** Account-center account the provider is bound to; absent for a credential-free kind. */
  providerAccountId?: string;
  /** Stable account code of the bootstrapped account; absent for a credential-free kind. */
  accountCode?: string;
  /** True when this call created the account; false when an account already existed and was left untouched. */
  accountCreated: boolean;
  /** True when this call wrote the placeholder access-key pair. False means an account already existed, so its credential was deliberately not overwritten. */
  credentialSeeded: boolean;
  credentialFields?: StorageProviderVendorCredentialFields;
  vendorCapabilities?: StorageProviderVendorCapabilityDefaults;
}
