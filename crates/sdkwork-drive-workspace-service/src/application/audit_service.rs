use crate::domain::audit::DriveAuditEvent;
use crate::ports::audit_store::{
    AuditEventPage, DriveAuditStore, ListAuditEventsQuery, NewDriveAuditEvent,
};
use crate::DriveServiceError;

#[derive(Debug, Clone)]
pub struct RecordAuditEventCommand {
    pub tenant_id: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: String,
    pub operator_id: String,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ListAuditEventsCommand {
    pub tenant_id: Option<String>,
    pub action: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct DriveAuditService<S>
where
    S: DriveAuditStore,
{
    store: S,
}

impl<S> DriveAuditService<S>
where
    S: DriveAuditStore,
{
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub async fn record_event(
        &self,
        command: RecordAuditEventCommand,
    ) -> Result<DriveAuditEvent, DriveServiceError> {
        if command.tenant_id.trim().is_empty() {
            return Err(DriveServiceError::Validation(
                "tenant_id is required".to_string(),
            ));
        }
        if command.action.trim().is_empty() {
            return Err(DriveServiceError::Validation(
                "action is required".to_string(),
            ));
        }
        if command.resource_type.trim().is_empty() || command.resource_id.trim().is_empty() {
            return Err(DriveServiceError::Validation(
                "resource_type and resource_id are required".to_string(),
            ));
        }
        if command.operator_id.trim().is_empty() {
            return Err(DriveServiceError::Validation(
                "operator_id is required".to_string(),
            ));
        }

        self.store
            .append_event(&NewDriveAuditEvent {
                tenant_id: command.tenant_id,
                action: command.action,
                resource_type: command.resource_type,
                resource_id: command.resource_id,
                operator_id: command.operator_id,
                request_id: command.request_id,
                trace_id: command.trace_id,
            })
            .await
    }

    pub async fn list_events(
        &self,
        command: ListAuditEventsCommand,
    ) -> Result<AuditEventPage, DriveServiceError> {
        let page = command.page.unwrap_or(1);
        if page == 0 {
            return Err(DriveServiceError::Validation(
                "page must be greater than 0".to_string(),
            ));
        }

        let page_size = command.page_size.unwrap_or(20);
        if !(1..=100).contains(&page_size) {
            return Err(DriveServiceError::Validation(
                "page_size must be in range [1, 100]".to_string(),
            ));
        }

        self.store
            .list_events(&ListAuditEventsQuery {
                tenant_id: normalize_filter(command.tenant_id),
                action: normalize_optional_identifier("action", command.action, 128)?,
                resource_type: normalize_optional_identifier(
                    "resource_type",
                    command.resource_type,
                    64,
                )?,
                resource_id: normalize_optional_identifier(
                    "resource_id",
                    command.resource_id,
                    128,
                )?,
                request_id: normalize_optional_identifier("request_id", command.request_id, 64)?,
                trace_id: normalize_optional_identifier("trace_id", command.trace_id, 128)?,
                page,
                page_size,
            })
            .await
    }
}

fn normalize_filter(input: Option<String>) -> Option<String> {
    let value = input?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.to_string())
}

fn normalize_optional_identifier(
    field_name: &str,
    value: Option<String>,
    max_len: usize,
) -> Result<Option<String>, DriveServiceError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = value.trim().to_string();
    if value.is_empty() {
        return Ok(None);
    }
    validate_identifier(field_name, &value, max_len)?;
    Ok(Some(value))
}

fn validate_identifier(
    field_name: &str,
    value: &str,
    max_len: usize,
) -> Result<(), DriveServiceError> {
    if value.len() > max_len {
        return Err(DriveServiceError::Validation(format!(
            "{field_name} length must be <= {max_len}"
        )));
    }
    if !value.chars().all(is_allowed_identifier_char) {
        return Err(DriveServiceError::Validation(format!(
            "{field_name} contains invalid characters; allowed: [A-Za-z0-9._:@-]"
        )));
    }
    Ok(())
}

fn is_allowed_identifier_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | ':' | '@' | '-')
}
