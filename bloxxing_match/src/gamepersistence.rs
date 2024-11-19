use axum::{
    extract::Query,
    http::{HeaderName, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use bloxxing_match::api_404;
use http::Request;
use http::{HeaderMap, Method};
use serde::{Deserialize, Serialize};
use surrealdb::{Connection, Surreal};
use tower::Layer;
use tower_http::cors::CorsLayer;
pub(crate) fn new<T: Connection>() -> Router<Surreal<T>> {
    let app: Router<Surreal<T>> = Router::new()
        .route("/persistence/set", post(persistence_set))
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
    app
}
async fn persistence_set(headers: HeaderMap, body: String) -> String {
    print!("persistence set {:?} {:?}", headers, body);
    "OK".to_string()
}
