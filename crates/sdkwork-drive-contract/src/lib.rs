//! SDKWork Drive contract types.
//!
//! This crate defines the public contract types for the SDKWork Drive domain.
//! It re-exports storage contract types and defines domain-specific types
//! for spaces, nodes, uploads, downloads, and API contracts.

pub use sdkwork_drive_storage_contract as storage;

pub mod api;
pub mod drive;

mod error;
pub use error::DriveContractError;

mod ids;
pub use ids::{DriveNodeId, DriveProviderId, DriveSpaceId, DriveUploadSessionId};

mod uri;
pub use uri::DriveUri;

mod media_resource;
pub use media_resource::{
    build_drive_backed_media_resource, drive_backed_media_resource_to_json,
    drive_uri_from_media_resource_json, BuildDriveBackedMediaResourceInput,
    DriveBackedMediaResource, DriveMediaChecksum,
};
