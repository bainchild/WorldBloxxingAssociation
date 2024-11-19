use axum::{
    http::HeaderName,
    routing::get, Router,
};
use bloxxing_match::api_404;
use http::Method;
use surrealdb::{Connection, Surreal};
use tower_http::cors::CorsLayer;
pub(crate) fn new<T: Connection>() -> Router<Surreal<T>> {
    Router::new()
        .route("/tags/c/36/studio.json", get(get_studio_json))
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
                    "RBXAuthenticationTicket".parse::<HeaderName>().unwrap(),
                    "x-csrf-token".parse::<HeaderName>().unwrap(),
                ])
                .allow_credentials(true),
        )
        .fallback(api_404)
}
async fn get_studio_json() -> String {
    "{\"flair_groups\":[],\"topic_list\":[],\"users\":[],\"primary_groups\":[]}".to_string()
}
