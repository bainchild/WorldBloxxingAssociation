use axum::{
    extract::Query,
    http::{HeaderName, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use bloxxing_match::api_404;
use http::Method;
use serde::{Deserialize, Serialize};
use surrealdb::{Connection, Surreal};
use tower_http::cors::CorsLayer;
pub(crate) fn new<T: Connection>() -> Router<Surreal<T>> {
    let app2: Router<Surreal<T>> = Router::new()
        .route(
            // "/versioncompatibility/GetCurrentClientVersionUpload",
            "/GetCurrentClientVersionUpload/",
            get(get_current_client_ver),
        )
        // /GetAllowedMD5Hashes/
        .route("/GetAllowedMD5Hashes/", get(get_allowed_md5s))
        .layer(
            CorsLayer::new()
                .allow_origin([
                    "https://www.roblox.com".parse().unwrap(),
                    "https://web.roblox.com".parse().unwrap(),
                    "https://roblox.com".parse().unwrap(),
                ])
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                .allow_headers([
                    "authorization".parse::<HeaderName>().unwrap(),
                    "x-bound-auth-token".parse::<HeaderName>().unwrap(),
                    "rbxauthorizationticket".parse::<HeaderName>().unwrap(),
                    "x-csrf-token".parse::<HeaderName>().unwrap(),
                ])
                .allow_credentials(true),
        )
        .fallback(api_404);
    // let app = MapRequestLayer::new(hostroute::<Body>).layer(app2);
    // app
    app2
}
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
struct ApiKeyAndBinaryType {
    apiKey: Option<String>,
    binaryType: Option<String>,
}
#[warn(non_snake_case)]
async fn get_current_client_ver(q: Query<ApiKeyAndBinaryType>) -> impl IntoResponse {
    println!("got client ver upload");
    // ?apiKey=76e5a40c-3ae1-4028-9f10-7c62520bd94f&binaryType=RccService
    if q.binaryType.as_ref().is_some_and(|x| *x == "RccService") {
        return (StatusCode::OK, "\"version-b3c3e5ac3b344719\"");
    }
    (StatusCode::OK, "\"version-b3c3e5ac3b344719\"")
}
async fn get_allowed_md5s() -> impl IntoResponse {
    "{\"data\": []}"
}
