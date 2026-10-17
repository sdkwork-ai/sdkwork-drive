export interface StorageProviderAccount {
  id: string;
  /** Account scope. platform = a global default published by platform operators and readable from every tenant; tenant = the application tenant's own default; user = a personal account owned by one end user. */
  scopeType: 'platform' | 'tenant' | 'user';
  /** Set only for user-scoped accounts: the owning user id. */
  ownerUserId?: string;
  /** Service provider vendor code, for example aliyun, tencent, aws, minio. */
  vendorCode: string;
  /** Business-agnostic account code, unique per tenant/organization. */
  accountCode: string;
  displayName: string;
  /** The identity shape of the credential, not the account's relationship to the tenant. The storage console stores an access-key pair and therefore always records `long_term_key`; a create request naming any other shape is rejected rather than silently overwritten. `temporary_credential` and the federated / managed families are reserved for accounts that authenticate through a broker or an identity provider. */
  accountType: 'long_term_key' | 'temporary_credential' | 'service_account' | 'service_linked_role' | 'federated_identity' | 'managed_identity' | 'api_key';
  environment: 'development' | 'sandbox' | 'production';
  /** Upstream account identifier, for example the Aliyun UID. */
  externalAccountId?: string;
  /** What the account can serve; object_storage marks storage consumers. */
  capabilityCodes?: string[];
  regionCode?: string;
  status: 'active' | 'disabled' | 'deleted';
  /** Whether an active credential row exists for this account. */
  credentialConfigured: boolean;
  /** How many non-deleted credential rows the account owns. */
  credentialCount: number;
  version: number;
  /** Whether this account is its scope's default for the vendor + environment, i.e. what a consumer gets when it does not name an account. */
  isDefault: boolean;
}
