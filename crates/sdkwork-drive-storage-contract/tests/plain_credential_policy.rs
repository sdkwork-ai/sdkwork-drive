use sdkwork_drive_storage_contract::{
    resolve_drive_storage_credentials, DriveObjectStoreErrorKind,
};

#[test]
fn plain_credential_ref_is_rejected_in_production_profile() {
    std::env::set_var("SDKWORK_DRIVE_RUNTIME_PROFILE", "production");
    let error = resolve_drive_storage_credentials(
        Some("plain:access:secret"),
        "TEST_ACCESS_KEY",
        "TEST_SECRET_KEY",
        "TEST_SESSION_TOKEN",
        "test provider",
    )
    .expect_err("plain credential refs must be rejected in production");
    assert_eq!(error.kind, DriveObjectStoreErrorKind::InvalidRequest);
    std::env::remove_var("SDKWORK_DRIVE_RUNTIME_PROFILE");
}
