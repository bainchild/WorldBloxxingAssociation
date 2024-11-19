use axum::{
    extract::{Path, Query, State},
    http::HeaderName,
    response::{IntoResponse, Redirect},
    routing::get,
    Json, Router,
};
use bloxxing_match::{api_404, get_asset_from_id, make_error, IdQuery};
use http::{Method, StatusCode};
use robacking::Roblox::{
    assetdelivery_v1::{AssetContentRepresentationSpecifier, IAssetResponseItem},
    Web::WebAPI::APIErrors,
};
use surrealdb::{Connection, Surreal};
use tower_http::cors::CorsLayer;
pub(crate) fn new<T: Connection>() -> Router<Surreal<T>> {
    Router::new()
        .route("/asset", get(get_asset_with_query))
        .route("/asset/", get(get_asset_with_query))
        .route("/assetId/:id", get(get_asset_with_id))
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

async fn get_asset_with_id<T: Connection>(
    db: State<Surreal<T>>,
    Path(id): Path<u64>,
) -> Result<Json<IAssetResponseItem>, (StatusCode, Json<APIErrors>)> {
    let asset = get_asset_from_id(&db.0, id).await;
    match asset {
        Ok(aa) => Ok(Json(IAssetResponseItem {
            IsHashDynamic: false,
            IsArchived: false,
            Errors: None,
            location: "https://tr.rbxcdn.com/".to_string() + aa.hash.as_str(),
            ContentRepresentationSpecifier: Some(AssetContentRepresentationSpecifier {
                format: aa.format,
                majorVersion: aa.version,
                fidelity: "".to_string(),
            }),
            IsCopyrightProtected: false,
            requestId: "abcd".to_string(),
        })),
        Err(_) => Err((
            StatusCode::NOT_FOUND,
            Json(make_error(2, "Asset not found", None)),
        )),
    }
}
async fn get_asset_with_query<T: Connection>(
    db: State<Surreal<T>>,
    q: Query<IdQuery>,
) -> impl IntoResponse {
    let asset = get_asset_from_id(&db.0, q.id).await;
    println!("get asset: {:?}", asset);
    match asset {
        Ok(aa) => Ok(Redirect::temporary(
            ("https://tr.rbxcdn.com/".to_string() + aa.hash.as_str()).as_str(),
        )),
        Err(_) => Err((
            StatusCode::NOT_FOUND,
            Json(make_error(2, "Asset not found", None)),
        )),
    }
}
