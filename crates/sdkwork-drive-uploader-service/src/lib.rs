//! Drive server-side uploader service facade.
//!
//! Rust services that generate, import, or transform bytes into SDKWork-owned storage
//! should depend on this crate instead of calling Drive App API routes over HTTP.

pub mod service {
    pub use sdkwork_drive_workspace_service::domain::uploader::{DriveUploadItem, DriveUploadPart};
    pub use sdkwork_drive_workspace_service::infrastructure::sql::uploader_store::SqlUploaderStore;
    pub use sdkwork_drive_workspace_service::uploader::{
        CompleteStoredUploaderUploadCommand, DriveUploaderService, MarkUploaderPartUploadedCommand,
        PrepareUploaderUploadCommand, UploadBytesCommand, UploaderActor, UploaderRetention,
        UploaderTarget,
    };
}

#[cfg(test)]
mod tests {
    use super::service::PrepareUploaderUploadCommand;

    #[test]
    fn facade_exports_prepare_uploader_upload_command() {
        let _ = std::any::type_name::<PrepareUploaderUploadCommand>();
    }
}
