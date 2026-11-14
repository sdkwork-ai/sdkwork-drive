//! Admin workspace route manifest for hosts that compose the drive admin
//! surface (spaces + files) next to the admin storage surface.
//!
//! The full backend manifest stays the authority for the drive gateway; this
//! projection keeps the host composition honest without duplicating route
//! declarations by hand.

use sdkwork_web_core::{HttpRoute, HttpRouteManifest};

use crate::http_route_manifest::backend_route_manifest;

/// Operations the admin storage center owns on the drive node workspace plane.
pub const BACKEND_ADMIN_WORKSPACE_OPERATION_IDS: &[&str] =
    &["spaces.admin.list", "files.admin.list"];

/// Drive backend routes for tenant-scoped space and file administration.
///
/// Hosts that mount the drive admin storage contribution merge this manifest so
/// their Web Framework resolves the same dual-token auth profile (and therefore
/// does not answer `404 route-not-in-manifest`) for the admin workspace routes
/// that are registered by [`crate::build_backend_admin_workspace_router`].
pub fn backend_admin_workspace_route_manifest() -> HttpRouteManifest {
    let routes: Vec<HttpRoute> = backend_route_manifest()
        .routes()
        .iter()
        .filter(|route| BACKEND_ADMIN_WORKSPACE_OPERATION_IDS.contains(&route.operation_id))
        .cloned()
        .collect();
    HttpRouteManifest::from_owned_routes(routes)
}

#[cfg(test)]
mod tests {
    use super::{backend_admin_workspace_route_manifest, BACKEND_ADMIN_WORKSPACE_OPERATION_IDS};

    #[test]
    fn manifest_projects_only_the_admin_workspace_operations() {
        let manifest = backend_admin_workspace_route_manifest();
        assert_eq!(
            BACKEND_ADMIN_WORKSPACE_OPERATION_IDS.len(),
            manifest.routes().len(),
            "every admin workspace operation must be projected exactly once"
        );
        assert!(manifest
            .match_route("GET", "/backend/v3/api/drive/spaces")
            .is_some());
        assert!(manifest
            .match_route("GET", "/backend/v3/api/drive/files")
            .is_some());
        assert!(manifest
            .match_route("GET", "/backend/v3/api/drive/labels")
            .is_none());
    }
}
