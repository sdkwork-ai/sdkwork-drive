use serde::{Deserialize, Serialize};

/// Download URL information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveDownloadUrl {
    pub url: String,
    pub expires_at_epoch_ms: i64,
    pub method: String,
}

/// Download package item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveDownloadPackageItem {
    pub node_id: String,
    pub name: String,
    pub size_bytes: u64,
}

/// Download package entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveDownloadPackage {
    pub id: String,
    pub tenant_id: String,
    pub items: Vec<DriveDownloadPackageItem>,
    pub total_size_bytes: u64,
    pub status: String,
    pub created_at_ms: i64,
    pub expires_at_ms: i64,
}

/// Create download URL request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDownloadUrlRequest {
    pub space_id: String,
    pub node_id: String,
    pub requested_ttl_seconds: Option<i32>,
    pub operator_id: String,
}

/// Create download package request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDownloadPackageRequest {
    pub items: Vec<DriveDownloadPackageItem>,
    pub operator_id: String,
}
