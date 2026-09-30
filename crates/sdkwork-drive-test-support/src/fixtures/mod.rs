//! Test fixtures for Drive entities.

pub mod node_seed;
pub mod nodes;
pub mod provider_seed;
pub mod providers;
pub mod spaces;
pub mod upload_sessions;

pub use node_seed::{
    content_type_group_for, file_extension_for, insert_plain_folder_node, insert_ready_file_node,
    zero_checksum_sha256_hex, ReadyFileNodeSeed, DEFAULT_READY_FILE_CONTENT_TYPE,
    DEFAULT_READY_FILE_CONTENT_TYPE_GROUP,
};
pub use provider_seed::{
    seed_storage_provider_and_binding, StorageProviderSeed, DEFAULT_SEEDED_PROVIDER_BUCKET,
    DEFAULT_SEEDED_PROVIDER_CREDENTIAL_REF,
};
