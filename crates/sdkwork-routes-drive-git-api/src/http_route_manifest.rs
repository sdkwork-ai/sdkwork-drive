use sdkwork_web_core::{HttpMethod, HttpRoute, HttpRouteManifest};

const HTTP_ROUTES: &[HttpRoute] = &[
    HttpRoute::public(
        HttpMethod::Get,
        "/git/{*path}",
        "gitSmartHttp",
        "gitSmartHttp.dispatchGet",
    ),
    HttpRoute::public(
        HttpMethod::Post,
        "/git/{*path}",
        "gitSmartHttp",
        "gitSmartHttp.dispatchPost",
    ),
];

pub fn git_route_manifest() -> HttpRouteManifest {
    HttpRouteManifest::new(HTTP_ROUTES)
}
