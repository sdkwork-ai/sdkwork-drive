use async_trait::async_trait;

pub const MAX_EMBEDDED_OUTBOX_TARGETS_PER_EVENT: usize = 200;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveDomainOutboxEmbeddedTarget {
    pub channel_id: String,
    pub source_scope_uuid: String,
}

#[derive(Debug, Clone, Copy)]
pub struct ResolveDriveDomainOutboxEmbeddedTargetsRequest<'a> {
    pub tenant_id: &'a str,
    pub space_id: &'a str,
    pub payload_json: &'a str,
}

#[derive(Debug, Clone, Copy)]
pub struct DeliverDriveDomainOutboxEmbeddedEventRequest<'a> {
    pub outbox_id: &'a str,
    pub tenant_id: &'a str,
    pub space_id: &'a str,
    pub attempt_count: i32,
    pub channel_id: &'a str,
    pub source_scope_uuid: &'a str,
    pub payload_json: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DriveDomainOutboxEmbeddedRelayError {
    InvalidEvent(String),
    Delivery(String),
}

impl std::fmt::Display for DriveDomainOutboxEmbeddedRelayError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidEvent(detail) => {
                write!(formatter, "invalid embedded outbox event: {detail}")
            }
            Self::Delivery(detail) => {
                write!(formatter, "embedded outbox delivery failed: {detail}")
            }
        }
    }
}

impl std::error::Error for DriveDomainOutboxEmbeddedRelayError {}

/// Optional process-local delivery boundary for standalone applications that embed Drive.
///
/// The Drive dispatcher remains the retry and dead-letter authority. Consumers resolve only
/// their root-scoped targets and receive exact event bytes through this typed port.
#[async_trait]
pub trait DriveDomainOutboxEmbeddedRelay: Send + Sync {
    async fn resolve_targets(
        &self,
        request: ResolveDriveDomainOutboxEmbeddedTargetsRequest<'_>,
    ) -> Result<Vec<DriveDomainOutboxEmbeddedTarget>, DriveDomainOutboxEmbeddedRelayError>;

    async fn deliver(
        &self,
        request: DeliverDriveDomainOutboxEmbeddedEventRequest<'_>,
    ) -> Result<(), DriveDomainOutboxEmbeddedRelayError>;
}
