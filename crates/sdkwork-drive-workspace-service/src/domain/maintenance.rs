#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveSweepResult {
    pub scanned_count: i64,
    pub affected_count: i64,
    pub dry_run: bool,
}
