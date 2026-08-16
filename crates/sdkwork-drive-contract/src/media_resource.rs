use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{DriveNodeId, DriveSpaceId, DriveUri};

/// Drive-backed `MediaResource` per DRIVE_SPEC §10 and MEDIA_RESOURCE_SPEC.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveBackedMediaResource {
    pub id: String,
    pub kind: String,
    pub source: String,
    pub uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<DriveMediaChecksum>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<DriveBackedMediaMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveMediaChecksum {
    pub algorithm: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveBackedMediaMetadata {
    pub drive: DriveBackedMediaDriveMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveBackedMediaDriveMetadata {
    pub space_id: String,
    pub node_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub space_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_version: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BuildDriveBackedMediaResourceInput<'a> {
    pub space_id: &'a DriveSpaceId,
    pub node_id: &'a DriveNodeId,
    pub kind: &'a str,
    pub file_name: Option<&'a str>,
    pub mime_type: Option<&'a str>,
    pub size_bytes: Option<i64>,
    pub checksum_sha256_hex: Option<&'a str>,
    pub space_type: Option<&'a str>,
    pub node_version: Option<&'a str>,
}

pub fn build_drive_backed_media_resource(
    input: BuildDriveBackedMediaResourceInput<'_>,
) -> DriveBackedMediaResource {
    let uri = DriveUri::new(input.space_id, input.node_id);
    DriveBackedMediaResource {
        id: input.node_id.as_str().to_string(),
        kind: input.kind.to_string(),
        source: "drive".to_string(),
        uri: uri.to_string(),
        file_name: input.file_name.map(str::to_string),
        mime_type: input.mime_type.map(str::to_string),
        size_bytes: input.size_bytes.map(|value| value.to_string()),
        checksum: input.checksum_sha256_hex.map(|value| DriveMediaChecksum {
            algorithm: "sha256".to_string(),
            value: value.to_string(),
        }),
        metadata: Some(DriveBackedMediaMetadata {
            drive: DriveBackedMediaDriveMetadata {
                space_id: input.space_id.as_str().to_string(),
                node_id: input.node_id.as_str().to_string(),
                space_type: input.space_type.map(str::to_string),
                node_version: input.node_version.map(str::to_string),
            },
        }),
    }
}

pub fn drive_backed_media_resource_to_json(
    resource: &DriveBackedMediaResource,
) -> Result<Value, serde_json::Error> {
    serde_json::to_value(resource)
}

pub fn drive_uri_from_media_resource_json(value: &Value) -> Option<String> {
    value
        .get("uri")
        .and_then(Value::as_str)
        .filter(|uri| uri.starts_with(DriveUri::PREFIX))
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn builds_drive_backed_media_resource_with_canonical_uri() {
        let resource = build_drive_backed_media_resource(BuildDriveBackedMediaResourceInput {
            space_id: &DriveSpaceId::new("space-001"),
            node_id: &DriveNodeId::new("node-001"),
            kind: "document",
            file_name: Some("report.pdf"),
            mime_type: Some("application/pdf"),
            size_bytes: Some(424_242),
            checksum_sha256_hex: Some(
                "3f786850e387550fdab836ed7e6dc881de23001b000000000000000000000000",
            ),
            space_type: Some("personal"),
            node_version: Some("1"),
        });

        assert_eq!(resource.source, "drive");
        assert_eq!(resource.uri, "drive://spaces/space-001/nodes/node-001");
        assert_eq!(resource.size_bytes.as_deref(), Some("424242"));
        let json = drive_backed_media_resource_to_json(&resource).expect("json");
        assert_eq!(
            drive_uri_from_media_resource_json(&json).as_deref(),
            Some("drive://spaces/space-001/nodes/node-001")
        );
        assert_eq!(json["metadata"]["drive"]["spaceId"], json!("space-001"));
    }
}
