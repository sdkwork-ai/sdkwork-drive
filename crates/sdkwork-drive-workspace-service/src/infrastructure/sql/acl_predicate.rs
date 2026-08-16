//! ACL SQL predicates for list and change-feed queries.
//!
//! Keep in sync with `sdkwork-routes-drive-app-api/src/acl_sql.rs`.

const READER_SATISFYING_ROLES: &[&str] = &["owner", "writer", "commenter", "reader"];

fn reader_roles_sql() -> String {
    READER_SATISFYING_ROLES
        .iter()
        .map(|role| format!("'{role}'"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn node_ancestors_cte(node_alias: &str) -> String {
    format!(
        "WITH RECURSIVE node_ancestors(id) AS (
            SELECT {node_alias}.id
            UNION ALL
            SELECT current_node.parent_node_id
            FROM dr_drive_node current_node
            INNER JOIN node_ancestors ancestor ON current_node.id = ancestor.id
            WHERE current_node.tenant_id = {node_alias}.tenant_id
              AND current_node.parent_node_id IS NOT NULL
        )"
    )
}

pub(crate) fn reader_inherited_permission_exists_sql(
    node_alias: &str,
    subject_type_bind: &str,
    subject_id_bind: &str,
) -> String {
    let roles = reader_roles_sql();
    format!(
        "EXISTS (
            {ancestors}
            SELECT 1
            FROM dr_drive_node_permission permission_row
            INNER JOIN node_ancestors ancestor ON permission_row.node_id = ancestor.id
            WHERE permission_row.tenant_id = {node_alias}.tenant_id
              AND permission_row.subject_type = {subject_type_bind}
              AND permission_row.subject_id = {subject_id_bind}
              AND permission_row.lifecycle_status = 'active'
              AND permission_row.role IN ({roles})
            LIMIT 1
        )",
        ancestors = node_ancestors_cte(node_alias),
        node_alias = node_alias,
        subject_type_bind = subject_type_bind,
        subject_id_bind = subject_id_bind,
        roles = roles,
    )
}

/// Returns a predicate that is true when the subject can list or access the space.
pub(crate) fn space_accessible_to_subject_sql(
    space_alias: &str,
    subject_type_bind: &str,
    subject_id_bind: &str,
) -> String {
    let anchor_reader = reader_inherited_permission_exists_sql(
        "space_anchor_node",
        subject_type_bind,
        subject_id_bind,
    );
    format!(
        "(
            ({space_alias}.owner_subject_type = {subject_type_bind}
             AND {space_alias}.owner_subject_id = {subject_id_bind})
            OR EXISTS (
                SELECT 1
                FROM dr_drive_node_permission permission_row
                INNER JOIN dr_drive_node node_row
                   ON node_row.tenant_id = permission_row.tenant_id
                  AND node_row.id = permission_row.node_id
                WHERE node_row.tenant_id = {space_alias}.tenant_id
                  AND node_row.space_id = {space_alias}.id
                  AND permission_row.subject_type = {subject_type_bind}
                  AND permission_row.subject_id = {subject_id_bind}
                  AND permission_row.lifecycle_status = 'active'
            )
            OR EXISTS (
                SELECT 1
                FROM dr_drive_node space_anchor_node
                WHERE space_anchor_node.tenant_id = {space_alias}.tenant_id
                  AND space_anchor_node.space_id = {space_alias}.id
                  AND space_anchor_node.parent_node_id IS NULL
                  AND space_anchor_node.lifecycle_status = 'active'
                  AND ({anchor_reader})
            )
        )",
        space_alias = space_alias,
        subject_type_bind = subject_type_bind,
        subject_id_bind = subject_id_bind,
        anchor_reader = anchor_reader,
    )
}

#[cfg(test)]
mod tests {
    use super::{reader_inherited_permission_exists_sql, space_accessible_to_subject_sql};

    #[test]
    fn reader_predicate_uses_recursive_ancestor_walk() {
        let sql = reader_inherited_permission_exists_sql("n", "$4", "$5");
        assert!(sql.contains("WITH RECURSIVE node_ancestors"));
        assert!(sql.contains("permission_row.subject_type = $4"));
        assert!(sql.contains("'reader'"));
    }

    #[test]
    fn space_accessible_predicate_checks_owner_grants_and_anchor_reader() {
        let sql = space_accessible_to_subject_sql("s", "$2", "$3");
        assert!(sql.contains("s.owner_subject_type = $2"));
        assert!(sql.contains("space_anchor_node.parent_node_id IS NULL"));
        assert!(sql.contains("WITH RECURSIVE node_ancestors"));
    }
}
