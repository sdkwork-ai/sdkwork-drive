use async_trait::async_trait;

use crate::domain::audit::DriveAuditEvent;
use crate::DriveServiceError;

#[derive(Debug, Clone)]
pub struct NewDriveAuditEvent {
    pub tenant_id: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: String,
    pub operator_id: String,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ListAuditEventsQuery {
    pub tenant_id: Option<String>,
    pub action: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Clone)]
pub struct AuditEventPage {
    pub items: Vec<DriveAuditEvent>,
    pub page: u32,
    pub page_size: u32,
    pub total: i64,
}

#[async_trait]
pub trait DriveAuditStore: Send + Sync {
    async fn append_event(
        &self,
        new_event: &NewDriveAuditEvent,
    ) -> Result<DriveAuditEvent, DriveServiceError>;

    async fn list_events(
        &self,
        query: &ListAuditEventsQuery,
    ) -> Result<AuditEventPage, DriveServiceError>;
}
