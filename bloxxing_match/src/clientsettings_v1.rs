use std::fs;

use axum::{
    extract::{Path, Query},
    http::{HeaderName, StatusCode},
    routing::get,
    Json, Router,
};
use bloxxing_match::api_404;
use http::Method;
use robacking::internal::{ApplicationName, BinaryType};
use robacking::Roblox::clientsettings_v1::ClientVersionResponse;
use robacking::Roblox::Web::WebAPI::{APIError, APIErrors};
use serde::{Deserialize, Serialize};
use surrealdb::{Connection, Surreal};
use tower_http::cors::CorsLayer;
pub(crate) fn new<T: Connection>() -> Router<Surreal<T>> {
    Router::new()
        .route("/client-version/:typpe", get(client_version_for_type))
        .route("/settings/application", get(get_fflags))
        // .route("/mobile-client-version", get(mobile_client_version))
        // .route("/installer-cdns", get(installer_cdns))
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
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
struct UnimportantApplicationName {
    pub applicationName: ApplicationName,
}
#[warn(non_snake_case)]
async fn get_fflags(
    Query(appl): Query<UnimportantApplicationName>,
) -> Result<(StatusCode, String), (StatusCode, Json<APIErrors>)> {
    let applname = match appl.applicationName {
        ApplicationName::AndroidApp => "AndroidApp",
        ApplicationName::iOSApp => "iOSApp",
        ApplicationName::MacClientBootstrapper => "MacClientBootstrapper",
        ApplicationName::MacDesktopClient => "MacDesktopClient",
        ApplicationName::MacStudioBootstrapper => "MacStudioBootstrapper",
        ApplicationName::PCClientBootstrapper => "PCClientBootstrapper",
        ApplicationName::PCDesktopClient => "PCDesktopClient",
        ApplicationName::PCStudioBootstrapper => "PCStudioBootstrapper",
        ApplicationName::PCStudioApp => "PCStudioApp",
        ApplicationName::StudioApp => "StudioApp",
        ApplicationName::XboxClient => "XboxClient",
    };
    match fs::read_to_string("fflags/".to_string() + applname + ".json") {
        Ok(a) => Ok((
            StatusCode::OK,
            "{\"applicationSettings\":".to_string() + a.as_str() + "}",
        )),
        Err(_) => Err((
            StatusCode::BAD_REQUEST,
            Json(APIErrors {
                errors: vec![APIError {
                    code: 0,
                    message: "Application not supported".to_string(),
                    userFacingMessage: None,
                }],
            }),
        )),
    }
}
async fn client_version_for_type(
    Path(id): Path<BinaryType>,
) -> Result<(StatusCode, Json<ClientVersionResponse>), (StatusCode, Json<APIErrors>)> {
    Ok((
        StatusCode::OK,
        Json(match id {
            BinaryType::WindowsPlayer => ClientVersionResponse {
                version: "version-9563a5b30aec4fcc".to_string(),
                bootstrapperVersion: "0.421.0.385673".to_string(),
                nextClientVersionUpload: None,
                nextClientVersion: None,
                clientVersionUpload: "0.421.0.385673".to_string(),
            },
            BinaryType::WindowsStudio => ClientVersionResponse {
                version: "version-ecd9f4b89d284f7e".to_string(),
                bootstrapperVersion: "0.421.0.385201".to_string(),
                nextClientVersionUpload: None,
                nextClientVersion: None,
                clientVersionUpload: "0.421.0.385201".to_string(),
            },
            BinaryType::WindowsStudio64 => ClientVersionResponse {
                version: "version-ecd9f4b89d284f7e".to_string(),
                bootstrapperVersion: "1, 421, 0, 385201".to_string(),
                nextClientVersionUpload: None,
                nextClientVersion: None,
                clientVersionUpload: "version-ecd9f4b89d284f7e".to_string(),
            },
            BinaryType::Studio => ClientVersionResponse {
                version: "version-bb87f719312c4bb4".to_string(),
                bootstrapperVersion: "0.429.0.403252".to_string(),
                nextClientVersionUpload: None,
                nextClientVersion: None,
                clientVersionUpload: "0.429.0.403252".to_string(),
            },
            BinaryType::Studio64 => ClientVersionResponse {
                version: "version-78f0b28df63247af".to_string(),
                bootstrapperVersion: "0.421.0.385201".to_string(),
                nextClientVersionUpload: None,
                nextClientVersion: None,
                clientVersionUpload: "0.421.0.385201".to_string(),
            },
            _ => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(APIErrors {
                        errors: vec![APIError {
                            code: 0,
                            message: "unsupported client".to_string(),
                            userFacingMessage: None,
                        }],
                    }),
                ))
            }
        }),
    ))
}
