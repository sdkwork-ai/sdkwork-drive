//! SDKWork Drive test support utilities.
//!
//! This crate provides test fixtures, in-memory store implementations,
//! and assertion utilities for testing Drive services.

pub mod assertions;
pub mod fixtures;
pub mod in_memory;
pub mod table_absence;
pub mod test_database;

pub use fixtures::{
    content_type_group_for, file_extension_for, insert_plain_folder_node, insert_ready_file_node,
    seed_storage_provider_and_binding, zero_checksum_sha256_hex, ReadyFileNodeSeed,
    StorageProviderSeed, DEFAULT_READY_FILE_CONTENT_TYPE, DEFAULT_READY_FILE_CONTENT_TYPE_GROUP,
    DEFAULT_SEEDED_PROVIDER_BUCKET, DEFAULT_SEEDED_PROVIDER_CREDENTIAL_REF,
};
pub use table_absence::TableAbsenceGuard;
pub use test_database::{
    lazy_postgres_test_pool, postgres_test_database, PostgresTestDatabaseGuard,
};
