use axum::{
    http::{HeaderName, HeaderValue},
    routing::get,
    Json, Router,
};
use bloxxing_match::{api_404, PagelessPagedResponse};
use http::Method;
use surrealdb::{Connection, Surreal};
use tower_http::cors::CorsLayer;
pub(crate) fn new<T: Connection>() -> Router<Surreal<T>> {
    Router::new()
        .route("/locales", get(get_locales))
        .layer(
            CorsLayer::new()
                .allow_origin("https://www.roblox.com".parse::<HeaderValue>().unwrap())
                .allow_methods([Method::GET, Method::OPTIONS])
                .allow_headers([
                    "authorization".parse::<HeaderName>().unwrap(),
                    "x-bound-auth-token".parse::<HeaderName>().unwrap(),
                ])
                .allow_credentials(true), // CorsLayer::permissive(),
        )
        .fallback(api_404)
}
use serde::{Deserialize, Serialize};
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
struct LangInfo {
    id: u64,
    name: String,
    nativeName: String,
    languageCode: String,
    isRightToLeft: bool,
}
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
struct LocaleInfo {
    id: u64,
    locale: String,
    name: String,
    nativeName: String,
    language: LangInfo,
}
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
struct LocaleItem {
    locale: LocaleInfo,
    isEnabledForFullExperience: bool,
    isEnabledForSignupAndLogin: bool,
    isEnabledForInGameUgc: bool,
}
async fn get_locales() -> Json<PagelessPagedResponse<LocaleItem>> {
    Json(PagelessPagedResponse {
        data: vec![
            LocaleItem {
                locale: LocaleInfo {
                    id: 1,
                    locale: "en_us".to_string(),
                    name: "English(US)".to_string(),
                    nativeName: "English".to_string(),
                    language: LangInfo {
                        id: 41,
                        name: "English".to_string(),
                        nativeName: "English".to_string(),
                        languageCode: "en".to_string(),
                        isRightToLeft: false,
                    },
                },
                isEnabledForFullExperience: true,
                isEnabledForSignupAndLogin: true,
                isEnabledForInGameUgc: true,
            },
            LocaleItem {
                locale: LocaleInfo {
                    id: 2,
                    locale: "es_es".to_string(),
                    name: "Spanish(Spain)".to_string(),
                    nativeName: "Español".to_string(),
                    language: LangInfo {
                        id: 148,
                        name: "Spanish".to_string(),
                        nativeName: "Español".to_string(),
                        languageCode: "es".to_string(),
                        isRightToLeft: false,
                    },
                },
                isEnabledForFullExperience: true,
                isEnabledForSignupAndLogin: true,
                isEnabledForInGameUgc: true,
            },
        ],
    })
}
