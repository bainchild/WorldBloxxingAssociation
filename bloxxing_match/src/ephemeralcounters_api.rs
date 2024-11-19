use axum::{
    http::{HeaderName, StatusCode},
    response::IntoResponse,
    routing::get, Router,
};
use bloxxing_match::api_404;
use http::Method;
use surrealdb::{Connection, Surreal};
use tower_http::cors::CorsLayer;
pub(crate) fn new<T: Connection>() -> Router<Surreal<T>> {
    Router::new()
        /*
        POST /v1.0/SequenceStatistics/BatchAddToSequencesV2?apiKey=76E5A40C%2D3AE1%2D4028%2D9F10%2D7C62520BD94F
        Host: "ephemeralcounters.api.roblox.com"
        POST /v1.1/Counters/BatchIncrement?apiKey=76E5A40C%2D3AE1%2D4028%2D9F10%2D7C62520BD94F
        Host: "ephemeralcounters.api.roblox.com"
        */
        .route("/v1.0/SequenceStatistics/BatchAddToSequenceV2",get(empty))
        .route("/v1.1/Counters/BatchIncrement",get(empty))
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
        .fallback(api_404)
}
async fn empty() -> impl IntoResponse {
    StatusCode::OK
}
