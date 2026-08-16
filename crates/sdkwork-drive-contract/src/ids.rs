use serde::{Deserialize, Serialize};
use std::fmt;

/// Strongly-typed ID for a Drive space.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DriveSpaceId(String);

impl DriveSpaceId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DriveSpaceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for DriveSpaceId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for DriveSpaceId {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

/// Strongly-typed ID for a Drive node.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DriveNodeId(String);

impl DriveNodeId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DriveNodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for DriveNodeId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for DriveNodeId {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

/// Strongly-typed ID for a Drive storage provider.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DriveProviderId(String);

impl DriveProviderId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DriveProviderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for DriveProviderId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for DriveProviderId {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

/// Strongly-typed ID for a Drive upload session.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DriveUploadSessionId(String);

impl DriveUploadSessionId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DriveUploadSessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for DriveUploadSessionId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for DriveUploadSessionId {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn space_id_roundtrip() {
        let id = DriveSpaceId::new("space-123");
        assert_eq!(id.as_str(), "space-123");
        assert_eq!(id.to_string(), "space-123");
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"space-123\"");
        let parsed: DriveSpaceId = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, id);
    }

    #[test]
    fn node_id_from_str() {
        let id = DriveNodeId::from("node-456");
        assert_eq!(id.as_str(), "node-456");
    }
}
