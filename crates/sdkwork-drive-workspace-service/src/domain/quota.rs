#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveQuotaSummary {
    pub tenant_id: String,
    pub total_bytes: i64,
    pub object_count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveTenantQuotaPolicy {
    pub tenant_id: String,
    pub max_bytes: Option<i64>,
}
