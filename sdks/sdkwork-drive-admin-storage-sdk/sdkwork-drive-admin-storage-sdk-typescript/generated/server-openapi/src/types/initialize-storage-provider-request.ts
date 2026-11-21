export interface InitializeStorageProviderRequest {
  /** After the readiness probe passes, re-point every active logical bucket and the tenant fallback at this provider. Existing objects keep their own locators; only new writes move. */
  adoptAsDefault?: boolean;
}
