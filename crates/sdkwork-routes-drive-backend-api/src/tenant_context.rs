use sdkwork_drive_security::DriveAppContext;

pub(crate) fn authenticated_tenant_id(app_context: &DriveAppContext) -> String {
    app_context.tenant_id.clone()
}
