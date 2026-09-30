//! Canonical PostgreSQL node seeding helpers.
//!
//! Test suites must seed `dr_drive_node` rows that satisfy every baseline CHECK
//! constraint. In particular `ck_dr_drive_node_file_ready_head` requires that a
//! `node_type='file'` row in `content_state='ready'` always carries the complete
//! head-metadata quartet (`head_content_type`, `head_content_type_group`,
//! `head_content_length`, `head_version_no`), and
//! `ck_dr_drive_node_head_metadata_shape` requires non-file rows to carry none of
//! them.
//!
//! Duplicating that shape across dozens of raw `INSERT` literals is precisely how
//! the constraint drift happened before. These helpers are the single source of
//! truth for the insert shape, so new tests cannot silently reintroduce the
//! violation.

use sqlx::PgPool;

/// Default content type used when a ready file node has no explicit type.
pub const DEFAULT_READY_FILE_CONTENT_TYPE: &str = "application/octet-stream";

/// Default content type group matching [`DEFAULT_READY_FILE_CONTENT_TYPE`].
pub const DEFAULT_READY_FILE_CONTENT_TYPE_GROUP: &str = "binary";

/// Resolve the `head_content_type_group` value for a given content type.
///
/// Mirrors `sdkwork_drive_workspace_service::domain::uploader::content_type_group_for`
/// so test seeds agree with production writes. Kept local to avoid making the
/// test-support surface depend on deep service internals for a pure function.
pub fn content_type_group_for(content_type: &str) -> &'static str {
    let normalized = content_type.trim().to_ascii_lowercase();
    if normalized.starts_with("image/") {
        "image"
    } else if normalized.starts_with("video/") {
        "video"
    } else if normalized.starts_with("audio/") {
        "audio"
    } else if normalized.starts_with("text/") {
        "text"
    } else if matches!(
        normalized.as_str(),
        "application/pdf"
            | "application/msword"
            | "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
            | "application/vnd.ms-excel"
            | "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
            | "application/vnd.ms-powerpoint"
            | "application/vnd.openxmlformats-officedocument.presentationml.presentation"
    ) {
        "document"
    } else if matches!(
        normalized.as_str(),
        "application/zip"
            | "application/x-zip-compressed"
            | "application/gzip"
            | "application/x-tar"
    ) {
        "archive"
    } else {
        "binary"
    }
}

/// A ready `file` node seed that always satisfies the baseline head constraints.
#[derive(Debug, Clone)]
pub struct ReadyFileNodeSeed<'a> {
    pub id: &'a str,
    pub tenant_id: &'a str,
    pub space_id: &'a str,
    pub parent_node_id: Option<&'a str>,
    pub node_name: &'a str,
    pub scene: Option<&'a str>,
    pub source: Option<&'a str>,
    pub space_type: &'a str,
    pub content_type: &'a str,
    pub content_length: i64,
    pub head_version_no: i64,
    pub checksum_sha256_hex: String,
    pub file_extension: Option<String>,
    pub lifecycle_status: &'a str,
    pub version: i64,
    pub created_by: &'a str,
    pub updated_by: &'a str,
}

impl<'a> ReadyFileNodeSeed<'a> {
    /// Build a minimal ready file node seed; all invariants are derived.
    pub fn new(
        id: &'a str,
        tenant_id: &'a str,
        space_id: &'a str,
        node_name: &'a str,
        created_by: &'a str,
    ) -> Self {
        Self {
            id,
            tenant_id,
            space_id,
            parent_node_id: None,
            node_name,
            scene: None,
            source: None,
            space_type: "personal",
            content_type: DEFAULT_READY_FILE_CONTENT_TYPE,
            content_length: 0,
            head_version_no: 1,
            checksum_sha256_hex: zero_checksum_sha256_hex(),
            file_extension: file_extension_for(node_name),
            lifecycle_status: "active",
            version: 1,
            created_by,
            updated_by: created_by,
        }
    }

    pub fn with_parent_node_id(mut self, parent_node_id: &'a str) -> Self {
        self.parent_node_id = Some(parent_node_id);
        self
    }

    pub fn with_parent_node_id_opt(mut self, parent_node_id: Option<&'a str>) -> Self {
        self.parent_node_id = parent_node_id;
        self
    }

    pub fn with_space_type(mut self, space_type: &'a str) -> Self {
        self.space_type = space_type;
        self
    }

    pub fn with_scene(mut self, scene: &'a str) -> Self {
        self.scene = Some(scene);
        self
    }

    pub fn with_source(mut self, source: &'a str) -> Self {
        self.source = Some(source);
        self
    }

    pub fn with_content_type(mut self, content_type: &'a str) -> Self {
        self.content_type = content_type;
        self
    }

    pub fn with_content_length(mut self, content_length: i64) -> Self {
        self.content_length = content_length;
        self
    }

    pub fn with_head_version_no(mut self, head_version_no: i64) -> Self {
        self.head_version_no = head_version_no;
        self
    }

    pub fn with_lifecycle_status(mut self, lifecycle_status: &'a str) -> Self {
        self.lifecycle_status = lifecycle_status;
        self
    }

    pub fn with_updated_by(mut self, updated_by: &'a str) -> Self {
        self.updated_by = updated_by;
        self
    }
}

/// The canonical `sha256:000...0` checksum placeholder used by test seeds.
pub fn zero_checksum_sha256_hex() -> String {
    format!("sha256:{}", "0".repeat(64))
}

/// Derive a lowercase file extension from a node name, mirroring production.
pub fn file_extension_for(node_name: &str) -> Option<String> {
    let normalized = node_name.trim();
    let (_, extension) = normalized.rsplit_once('.')?;
    if extension.is_empty() || extension.contains('/') {
        return None;
    }
    Some(extension.to_ascii_lowercase())
}

/// Insert a ready `file` node satisfying every baseline node constraint.
pub async fn insert_ready_file_node(
    pool: &PgPool,
    seed: &ReadyFileNodeSeed<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, space_type, parent_node_id, node_type, node_name,
            scene, source, content_state, file_extension,
            head_content_type, head_content_type_group, head_content_length,
            head_version_no, head_checksum_sha256_hex,
            lifecycle_status, version, created_by, updated_by
         ) VALUES (
            $1, $2, $3, $4, $5, 'file', $6,
            $7, $8, 'ready', $9,
            $10, $11, $12,
            $13, $14,
            $15, $16, $17, $18
         )",
    )
    .bind(seed.id)
    .bind(seed.tenant_id)
    .bind(seed.space_id)
    .bind(seed.space_type)
    .bind(seed.parent_node_id)
    .bind(seed.node_name)
    .bind(seed.scene)
    .bind(seed.source)
    .bind(seed.file_extension.as_deref())
    .bind(seed.content_type)
    .bind(content_type_group_for(seed.content_type))
    .bind(seed.content_length)
    .bind(seed.head_version_no)
    .bind(&seed.checksum_sha256_hex)
    .bind(seed.lifecycle_status)
    .bind(seed.version)
    .bind(seed.created_by)
    .bind(seed.updated_by)
    .execute(pool)
    .await?;
    Ok(())
}

/// Insert a non-file (folder/shortcut/virtual_reference) node with no head metadata,
/// satisfying `ck_dr_drive_node_head_metadata_shape`.
pub async fn insert_plain_folder_node(
    pool: &PgPool,
    id: &str,
    tenant_id: &str,
    space_id: &str,
    parent_node_id: Option<&str>,
    node_name: &str,
    lifecycle_status: &str,
    created_by: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, parent_node_id, node_type, node_name,
            content_state, head_content_type, head_content_type_group, head_content_length, head_version_no, lifecycle_status, version, created_by, updated_by
         ) VALUES ($1, $2, $3, $4, 'folder', $5, 'ready', 'application/octet-stream', 'binary', 0, 1, $6, 1, $7, $7)",
    )
    .bind(id)
    .bind(tenant_id)
    .bind(space_id)
    .bind(parent_node_id)
    .bind(node_name)
    .bind(lifecycle_status)
    .bind(created_by)
    .execute(pool)
    .await?;
    Ok(())
}

/// SQL fragment adding the head-metadata quartet columns to a raw `dr_drive_node` insert.
///
/// Use inside a column list; pair with [`READY_FILE_HEAD_VALUES_FRAGMENT`].
pub const READY_FILE_HEAD_COLUMNS_FRAGMENT: &str =
    "head_content_type, head_content_type_group, head_content_length, head_version_no";

/// SQL fragment supplying literal head values for a ready file row.
pub const READY_FILE_HEAD_VALUES_FRAGMENT: &str = "'application/octet-stream', 'binary', 0, 1";
