use crate::{DriveContractError, DriveNodeId, DriveSpaceId};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Canonical Drive URI: `drive://spaces/{spaceId}/nodes/{nodeId}`
///
/// This is a stable reference to a Drive resource, not a delivery URL.
/// It must not contain provider bucket names, object keys, credentials,
/// or signed query strings.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DriveUri(String);

impl DriveUri {
    pub const PREFIX: &'static str = "drive://spaces/";

    /// Create a new DriveUri from space and node IDs.
    pub fn new(space_id: &DriveSpaceId, node_id: &DriveNodeId) -> Self {
        Self(format!("drive://spaces/{}/nodes/{}", space_id, node_id))
    }

    /// Parse a DriveUri from a string, validating the format.
    pub fn parse(raw: &str) -> Result<Self, DriveContractError> {
        if !raw.starts_with(Self::PREFIX) {
            return Err(DriveContractError::InvalidUri {
                reason: format!("must start with {}", Self::PREFIX),
            });
        }

        let remainder = &raw[Self::PREFIX.len()..];
        let parts: Vec<&str> = remainder.splitn(2, "/nodes/").collect();
        if parts.len() != 2 {
            return Err(DriveContractError::InvalidUri {
                reason: "missing /nodes/ separator".to_string(),
            });
        }

        let space_id = parts[0];
        let node_id = parts[1];

        if space_id.is_empty() {
            return Err(DriveContractError::InvalidUri {
                reason: "empty space id".to_string(),
            });
        }

        if node_id.is_empty() {
            return Err(DriveContractError::InvalidUri {
                reason: "empty node id".to_string(),
            });
        }

        Ok(Self(raw.to_string()))
    }

    /// Extract the space ID from this URI.
    pub fn space_id(&self) -> DriveSpaceId {
        let remainder = &self.0[Self::PREFIX.len()..];
        let space_str = remainder.split("/nodes/").next().unwrap_or("");
        DriveSpaceId::new(space_str)
    }

    /// Extract the node ID from this URI.
    pub fn node_id(&self) -> DriveNodeId {
        let remainder = &self.0[Self::PREFIX.len()..];
        let node_str = remainder.split_once("/nodes/").map(|x| x.1).unwrap_or("");
        DriveNodeId::new(node_str)
    }

    /// Get the raw URI string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DriveUri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for DriveUri {
    type Error = DriveContractError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl TryFrom<&str> for DriveUri {
    type Error = DriveContractError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_uri_roundtrip() {
        let uri = DriveUri::new(&DriveSpaceId::new("space-1"), &DriveNodeId::new("node-2"));
        assert_eq!(uri.as_str(), "drive://spaces/space-1/nodes/node-2");
        assert_eq!(uri.space_id(), DriveSpaceId::new("space-1"));
        assert_eq!(uri.node_id(), DriveNodeId::new("node-2"));
        assert_eq!(uri.to_string(), "drive://spaces/space-1/nodes/node-2");
    }

    #[test]
    fn parse_valid_uri() {
        let uri = DriveUri::parse("drive://spaces/s123/nodes/n456").unwrap();
        assert_eq!(uri.space_id(), DriveSpaceId::new("s123"));
        assert_eq!(uri.node_id(), DriveNodeId::new("n456"));
    }

    #[test]
    fn parse_invalid_prefix() {
        let result = DriveUri::parse("http://spaces/s1/nodes/n1");
        assert!(result.is_err());
    }

    #[test]
    fn parse_missing_nodes() {
        let result = DriveUri::parse("drive://spaces/s1/nodes/");
        assert!(result.is_err());
    }

    #[test]
    fn parse_empty_space() {
        let result = DriveUri::parse("drive://spaces//nodes/n1");
        assert!(result.is_err());
    }

    #[test]
    fn serde_roundtrip() {
        let uri = DriveUri::new(&DriveSpaceId::new("s1"), &DriveNodeId::new("n1"));
        let json = serde_json::to_string(&uri).unwrap();
        let parsed: DriveUri = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, uri);
    }
}
