use sdkwork_web_core::{HttpMethod, HttpRoute, HttpRouteManifest};

const HTTP_ROUTES: &[HttpRoute] = &[
    HttpRoute::public(
        HttpMethod::Get,
        "/open/v3/api/drive/share_links/{token}",
        "driveOpen",
        "openShareLinks.resolve",
    ),
    HttpRoute::public(
        HttpMethod::Post,
        "/open/v3/api/drive/share_links/{token}/download_url",
        "driveOpen",
        "openShareLinks.downloadUrls.create",
    ),
];

pub fn open_route_manifest() -> HttpRouteManifest {
    HttpRouteManifest::new(HTTP_ROUTES)
}
