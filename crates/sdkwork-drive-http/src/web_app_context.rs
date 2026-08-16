use http::HeaderMap;
use sdkwork_drive_security::DriveAppContext;

use crate::trace_ids::resolve_trace_id;

pub fn apply_trace_id_from_transport(headers: &HeaderMap, app_context: &mut DriveAppContext) {
    app_context.trace_id = resolve_trace_id(headers, &app_context.request_id);
}
