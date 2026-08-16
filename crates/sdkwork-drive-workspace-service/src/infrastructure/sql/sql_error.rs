pub(crate) fn is_unique_constraint_violation(message: &str) -> bool {
    message.contains("duplicate key value violates unique constraint")
}

pub(crate) fn normalize_timestamp_text(value: String) -> String {
    let trimmed = value.trim();
    if trimmed.contains('T') {
        return trimmed.to_string();
    }
    let normalized = trimmed.replace(' ', "T");
    if normalized.ends_with('Z') {
        normalized
    } else {
        format!("{normalized}Z")
    }
}

#[cfg(test)]
mod tests {
    use super::{is_unique_constraint_violation, normalize_timestamp_text};

    #[test]
    fn detects_postgres_unique_constraint_message() {
        assert!(is_unique_constraint_violation(
            "duplicate key value violates unique constraint \"dr_drive_storage_provider_pkey\""
        ));
    }

    #[test]
    fn ignores_unrelated_database_errors() {
        assert!(!is_unique_constraint_violation(
            "insert or update on table violates foreign key constraint"
        ));
    }

    #[test]
    fn normalizes_postgres_timestamp_text() {
        assert_eq!(
            normalize_timestamp_text("2026-06-17 12:34:56.789+00".to_string()),
            "2026-06-17T12:34:56.789+00Z"
        );
    }
}
