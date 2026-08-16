use sqlx::PgPool;

/// Default space setup status.
#[derive(Debug, Clone)]
pub struct SpaceSetupStatus {
    pub personal_space_created: bool,
    pub system_space_created: bool,
}

/// Set up default spaces for a tenant.
///
/// This function creates the default personal and system spaces
/// for a newly created tenant.
pub async fn setup_default_spaces(
    pool: &PgPool,
    tenant_id: &str,
) -> Result<SpaceSetupStatus, sqlx::Error> {
    let now = chrono::Utc::now().timestamp_millis();

    // Check if personal space exists
    let personal_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM drive_space WHERE tenant_id = $1 AND space_type = 'personal')",
    )
    .bind(tenant_id)
    .fetch_one(pool)
    .await?;

    // Create personal space if not exists
    if !personal_exists {
        let space_id = format!("personal-{}", tenant_id);
        sqlx::query(
            "INSERT INTO drive_space (id, tenant_id, owner_type, owner_id, space_type, name, version, created_at_ms, updated_at_ms) VALUES ($1, $2, 'user', $3, 'personal', 'Personal', 1, $4, $4)"
        )
        .bind(&space_id)
        .bind(tenant_id)
        .bind(tenant_id)
        .bind(now)
        .execute(pool)
        .await?;
    }

    // Check if system space exists
    let system_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM drive_space WHERE tenant_id = $1 AND space_type = 'system')",
    )
    .bind(tenant_id)
    .fetch_one(pool)
    .await?;

    // Create system space if not exists
    if !system_exists {
        let space_id = format!("system-{}", tenant_id);
        sqlx::query(
            "INSERT INTO drive_space (id, tenant_id, owner_type, owner_id, space_type, name, version, created_at_ms, updated_at_ms) VALUES ($1, $2, 'system', $3, 'system', 'System', 1, $4, $4)"
        )
        .bind(&space_id)
        .bind(tenant_id)
        .bind(tenant_id)
        .bind(now)
        .execute(pool)
        .await?;
    }

    Ok(SpaceSetupStatus {
        personal_space_created: !personal_exists,
        system_space_created: !system_exists,
    })
}
