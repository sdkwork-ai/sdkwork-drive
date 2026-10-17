export interface CreateStorageProviderAccountRequest {
  displayName: string;
  /** Service provider vendor code; the storage console derives it from the selected provider kind. */
  vendorCode: string;
  /** Business-agnostic account code, unique per tenant/organization. */
  accountCode: string;
  /** The identity shape of the credential, not the account's relationship to the tenant. The storage console stores an access-key pair and therefore always records `long_term_key`; a create request naming any other shape is rejected rather than silently overwritten. `temporary_credential` and the federated / managed families are reserved for accounts that authenticate through a broker or an identity provider. */
  accountType?: 'long_term_key' | 'temporary_credential' | 'service_account' | 'service_linked_role' | 'federated_identity' | 'managed_identity' | 'api_key';
  environment?: 'development' | 'sandbox' | 'production';
  externalAccountId?: string;
  regionCode?: string;
  /** Account scope. platform = a global default published by platform operators and readable from every tenant; tenant = the application tenant's own default; user = a personal account owned by one end user. platform requires a platform operator; user defaults its owner to the caller. */
  scopeType?: 'platform' | 'tenant' | 'user';
  /** Set only for user-scoped accounts: the owning user id. Naming somebody else is rejected. */
  ownerUserId?: string;
  /** Whether this account is its scope's default for the vendor + environment, i.e. what a consumer gets when it does not name an account. Promoting a new default demotes the previous one for the same vendor + environment. */
  isDefault?: boolean;
  accessKeyId: string;
  /** Sealed into the write-only credential row; never echoed back. */
  secretAccessKey: string;
  sessionToken?: string;
}
