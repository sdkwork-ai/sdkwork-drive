mod provider_store;
mod runtime;
mod upload_storage;

#[cfg(feature = "backend-fs")]
pub use provider_store::build_local_store_for_provider;
pub use provider_store::{
    build_store, ensure_provider_state_allows, map_provider_account_error, map_service_error,
    provider_supports_s3_object_store, DriveProviderLocation, DriveProviderObjectStoreFactory,
    ProviderAccessIntent, ResolvedProviderCredentials,
};
pub use runtime::DriveObjectStoreRuntime;
/// In-process `DriveUploadMultipartStorage` adapter for the upload-session
/// service; siblings that finalise Drive uploads use this instead of writing
/// their own provider resolution.
pub use upload_storage::DriveObjectRuntimeUploadStorage;
