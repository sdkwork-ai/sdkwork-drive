fn drive_role_rank(role: &str) -> u8 {
    match role {
        "reader" => 1,
        "commenter" => 2,
        "writer" => 3,
        "owner" => 4,
        _ => 0,
    }
}

/// Returns true when `effective` satisfies the access level required by `required`.
pub fn drive_role_satisfies(effective: &str, required: &str) -> bool {
    drive_role_rank(effective) >= drive_role_rank(required)
}

#[cfg(test)]
mod tests {
    use super::drive_role_satisfies;

    #[test]
    fn owner_satisfies_reader_and_writer() {
        assert!(drive_role_satisfies("owner", "reader"));
        assert!(drive_role_satisfies("owner", "writer"));
        assert!(drive_role_satisfies("owner", "owner"));
    }

    #[test]
    fn writer_satisfies_reader_but_not_owner() {
        assert!(drive_role_satisfies("writer", "reader"));
        assert!(!drive_role_satisfies("writer", "owner"));
    }

    #[test]
    fn reader_does_not_satisfy_writer() {
        assert!(!drive_role_satisfies("reader", "writer"));
    }

    #[test]
    fn reader_does_not_satisfy_commenter() {
        assert!(!drive_role_satisfies("reader", "commenter"));
    }

    #[test]
    fn commenter_satisfies_commenter_but_not_writer() {
        assert!(drive_role_satisfies("commenter", "commenter"));
        assert!(drive_role_satisfies("commenter", "reader"));
        assert!(!drive_role_satisfies("commenter", "writer"));
    }
}
