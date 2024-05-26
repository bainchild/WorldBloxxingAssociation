use axum::{
    extract::{rejection::JsonRejection, State},
    http::{HeaderName, StatusCode},
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::CookieJar;
use bloxxing_match::api_404;
use http::Method;
use robacking::Roblox::Web::WebAPI::{APIError, APIErrors};
use robacking::{internal::User, Roblox::economy_v1::CurrencyResponse};
use surrealdb::{Connection, Response, Surreal};
use tower_http::cors::CorsLayer;
pub(crate) fn new<T: Connection>() -> Router<Surreal<T>> {
    Router::new()
        .route("/user/currency", get(get_auth_currency))
        .route("/users/:id/currency", get(get_currency))
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
async fn get_auth_currency<T: Connection>(
    _db: State<Surreal<T>>,
    cook: CookieJar,
) -> Result<(StatusCode, Json<CurrencyResponse>), (StatusCode, Json<APIErrors>)> {
    if cook.get(".ROBLOSECURITY").is_some() {
        Ok((StatusCode::OK, Json(CurrencyResponse { robux: 443 })))
    } else {
        Err((
            StatusCode::UNAUTHORIZED,
            Json(APIErrors {
                errors: vec![APIError {
                    code: 0,
                    message: "Authorization has been denied for this request.".to_string(),
                    userFacingMessage: None,
                }],
            }),
        ))
    }
}
async fn get_currency<T: Connection>(
    _db: State<Surreal<T>>,
) -> Result<(StatusCode, Json<CurrencyResponse>), (StatusCode, Json<APIErrors>)> {
    Ok((StatusCode::OK, Json(CurrencyResponse { robux: 80 })))
}
