use sdkwork_utils_rust::sha256_hash;

pub struct BuildStorageObjectKeyCommand<'a> {
    pub tenant_id: &'a str,
    pub space_id: &'a str,
    pub node_id: &'a str,
    pub version_no: i64,
    pub object_id: &'a str,
}

pub struct DriveStorageKeyService;

impl DriveStorageKeyService {
    pub fn build_object_key(command: BuildStorageObjectKeyCommand<'_>) -> Result<String, String> {
        let tenant_id = require_part("tenant_id", command.tenant_id)?;
        let space_id = require_part("space_id", command.space_id)?;
        let node_id = require_part("node_id", command.node_id)?;
        let object_id = require_part("object_id", command.object_id)?;
        if command.version_no < 1 {
            return Err("version_no must be greater than zero".to_string());
        }

        let tenant_shard = shard_prefix(tenant_id);
        let node_shard = shard_prefix(node_id);
        Ok(format!(
            "sdkwork-drive/v1/t/{tenant_shard}/tenants/{tenant_id}/spaces/{space_id}/nodes/n/{node_shard}/{node_id}/versions/{:010}/{object_id}/content",
            command.version_no
        ))
    }
}

fn require_part<'a>(name: &str, value: &'a str) -> Result<&'a str, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(format!("{name} is required"));
    }
    if trimmed.contains('/') || trimmed.contains('\\') {
        return Err(format!("{name} must not contain path separators"));
    }
    Ok(trimmed)
}

fn shard_prefix(value: &str) -> String {
    let digest = sha256_hash(value.as_bytes());
    digest[..2].to_string()
}
