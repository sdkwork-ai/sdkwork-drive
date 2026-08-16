use serde::{Deserialize, Serialize};

/// Drive request context extracted from HTTP requests.
///
/// This context is populated from authenticated request headers
/// and provides tenant, operator, and organization information
/// for downstream service calls.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveRequestContext {
    /// Tenant ID from the authenticated user session.
    pub tenant_id: String,
    /// Operator (user) ID performing the request.
    pub operator_id: String,
    /// Optional organization ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,
    /// Optional request ID for tracing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

impl DriveRequestContext {
    /// Create a new request context.
    pub fn new(tenant_id: impl Into<String>, operator_id: impl Into<String>) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            operator_id: operator_id.into(),
            organization_id: None,
            request_id: None,
        }
    }

    /// Set the organization ID.
    pub fn with_organization_id(mut self, org_id: impl Into<String>) -> Self {
        self.organization_id = Some(org_id.into());
        self
    }

    /// Set the request ID.
    pub fn with_request_id(mut self, req_id: impl Into<String>) -> Self {
        self.request_id = Some(req_id.into());
        self
    }
}

/// Headers used for request context extraction.
pub mod headers {
    pub const TENANT_ID: &str = "X-Tenant-Id";
    pub const OPERATOR_ID: &str = "X-Operator-Id";
    pub const ORGANIZATION_ID: &str = "X-Organization-Id";
}
