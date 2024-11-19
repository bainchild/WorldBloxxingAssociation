use axum::{
    http::{HeaderName, StatusCode},
    routing::{get, post}, Router,
};
use bloxxing_match::api_404;
use http::Method;
use surrealdb::{Connection, Surreal};
use tower_http::cors::CorsLayer;
pub(crate) fn new<T: Connection>() -> Router<Surreal<T>> {
    Router::new()
        .route("/pe", post(empty))
        .route("/pe", get(empty))
        .route("/e.png", post(empty))
        .route("/e.png", get(empty))
        .route("/studio/e.png", post(empty))
        .route("/studio/e.png", get(empty))
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
                ])
                .allow_credentials(true),
        )
        .fallback(api_404)
}
async fn empty() -> StatusCode {
    StatusCode::OK
}
