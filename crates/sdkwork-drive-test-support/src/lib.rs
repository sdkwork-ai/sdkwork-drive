//! SDKWork Drive test support utilities.
//!
//! This crate provides test fixtures, in-memory store implementations,
//! and assertion utilities for testing Drive services.

pub mod assertions;
pub mod fixtures;
pub mod in_memory;
pub mod test_database;

pub use test_database::{
    lazy_postgres_test_pool, postgres_test_database, PostgresTestDatabaseGuard,
};
