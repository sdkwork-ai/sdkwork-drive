//! Seeding helpers for storage providers and their bindings.
//!
//! Upload and download resolution walks a binding chain
//! (`space` binding -> `space_type` binding -> `tenant` binding) before it ever
//! touches the provider row, so a suite that seeds only a provider still fails
//! with `NotFound("active storage provider not found")`. Centralizing both
//! inserts here keeps every suite on the same, resolvable shape.

use sqlx::PgPool;

/// Default bucket used by seeded uploader/downloader fixtures.
pub const DEFAULT_SEEDED_PROVIDER_BUCKET: &str = "bucket-uploader";
/// Default credential reference used by seeded provider fixtures.
pub const DEFAULT_SEEDED_PROVIDER_CREDENTIAL_REF: &str = "plain:test-access-key:test-secret-key";

/// A storage provider plus the binding that makes it resolvable for a tenant.
#[derive(Debug, Clone)]
pub struct StorageProviderSeed {
    pub provider_id: String,
    pub bucket: String,
    pub tenant_id: String,
    /// `None` seeds a tenant-wide binding; `Some(space)` seeds a space binding.
    pub space_id: Option<String>,
    pub credential_ref: String,
    pub storage_root_prefix: String,
    pub actor_id: String,
}

impl StorageProviderSeed {
    /// Seeds a tenant-wide `primary` binding with default upload fixture values.
    pub fn new(provider_id: &str, tenant_id: &str) -> Self {
        Self {
            provider_id: provider_id.to_string(),
            bucket: DEFAULT_SEEDED_PROVIDER_BUCKET.to_string(),
            tenant_id: tenant_id.to_string(),
            space_id: None,
            credential_ref: DEFAULT_SEEDED_PROVIDER_CREDENTIAL_REF.to_string(),
            storage_root_prefix: format!("tenants/{tenant_id}"),
            actor_id: "user-owner".to_string(),
        }
    }

    pub fn with_bucket(mut self, bucket: &str) -> Self {
        self.bucket = bucket.to_string();
        self
    }

    pub fn with_space_id(mut self, space_id: &str) -> Self {
        self.space_id = Some(space_id.to_string());
        self
    }

    pub fn with_credential_ref(mut self, credential_ref: &str) -> Self {
        self.credential_ref = credential_ref.to_string();
        self
    }

    pub fn with_actor_id(mut self, actor_id: &str) -> Self {
        self.actor_id = actor_id.to_string();
        self
    }
}

/// Inserts an active S3-compatible provider and a matching active binding.
///
/// The binding id is derived from the provider id so a suite can seed the same
/// provider twice (across `TRUNCATE`-driven reruns) without a primary-key clash.
pub async fn seed_storage_provider_and_binding(pool: &PgPool, seed: &StorageProviderSeed) {
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES (
            $1, 's3_compatible', $1, 'https://s3.example.com', 'us-east-1',
            $2, TRUE, $3, 'AES256', 'STANDARD', 'active', 1, $4, $4
        )
        ON CONFLICT (id) DO NOTHING",
    )
    .bind(&seed.provider_id)
    .bind(&seed.bucket)
    .bind(&seed.credential_ref)
    .bind(&seed.actor_id)
    .execute(pool)
    .await
    .expect("seed storage provider should succeed");

    let (binding_scope, purpose) = match seed.space_id {
        Some(_) => ("space", "primary"),
        None => ("tenant", "primary"),
    };
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider_binding (
            id, tenant_id, space_id, provider_id, binding_scope, purpose,
            storage_root_prefix, lifecycle_status, version, created_by, updated_by
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, 'active', 1, $8, $8)
        ON CONFLICT (id) DO NOTHING",
    )
    .bind(format!("binding-{}", seed.provider_id))
    .bind(&seed.tenant_id)
    .bind(&seed.space_id)
    .bind(&seed.provider_id)
    .bind(binding_scope)
    .bind(purpose)
    .bind(&seed.storage_root_prefix)
    .bind(&seed.actor_id)
    .execute(pool)
    .await
    .expect("seed storage provider binding should succeed");
}
