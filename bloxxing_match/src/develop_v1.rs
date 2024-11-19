/*
GET /v1/search/universes?q=creator:Team&limit=25&sort=-GameCreated
GET /v1/search/universes?q=creator:User&limit=25&sort=-GameCreated
GET /v1/search/universes?q=creator:User&limit=25&sort=-GameCreated
GET /v1/gametemplates?limit=100
GET /v1/user/groups/canmanage
*/
use axum::{extract::Query, http::HeaderName, routing::get, Json, Router};
use bloxxing_match::{api_404, ApiPageResponse};
use http::Method;
use robacking::Roblox::develop_v1::{GameTemplateModel, GroupModel, UniverseModel};
use serde::{Deserialize, Serialize};
use surrealdb::{Connection, Surreal};
use tower_http::cors::CorsLayer;
pub(crate) fn new<T: Connection>() -> Router<Surreal<T>> {
    Router::new()
        .route("/gametemplates", get(get_gametemplates))
        .route("/search/universes", get(search_universes))
        .route("/user/groups/canmanage", get(can_manage))
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
async fn can_manage() -> Json<Vec<GroupModel>> {
    // filler return type, vec is empty.
    Json(vec![GroupModel {
        id: 1,
        name: "Subspace Rift".to_string(),
    }])
}
#[derive(Serialize, Deserialize)]
struct SearchParams {
    q: String,
    limit: Option<u8>,
}
async fn search_universes(q: Query<SearchParams>) -> Json<ApiPageResponse<UniverseModel>> {
    if q.q != "creator:User" {
        Json(ApiPageResponse {
            previousPageCursor: None,
            nextPageCursor: None,
            data: vec![],
        })
    } else {
        Json(ApiPageResponse {
            previousPageCursor: None,
            nextPageCursor: None,
            data: vec![UniverseModel {
                isActive: true,
                privacyType: "Private".to_string(),
                creatorType: "User".to_string(),
                id: 1,
                creatorTargetId: 1,
                name: "Subspace Rift".to_string(),
                description: None,
                creatorName: "JohnDoe".to_string(),
                created: "NOW".to_string(),
                updated: "NEVER".to_string(),
                rootPlaceId: 2,
                isArchived: false,
            }],
        })
    }
}
#[derive(Serialize, Deserialize)]
struct GameTemplatesResponse {
    data: Vec<GameTemplateModel>,
}
async fn get_gametemplates() -> Json<GameTemplatesResponse> {
    Json(GameTemplatesResponse {
        data: vec![GameTemplateModel {
            hasTutorials: false,
            gameTemplateType: "Generic".to_string(),
            universe: UniverseModel {
                isActive: true,
                privacyType: "Private".to_string(),
                creatorType: "User".to_string(),
                id: 1,
                creatorTargetId: 1,
                name: "Subspace Rift".to_string(),
                description: None,
                creatorName: "JohnDoe".to_string(),
                created: "NOW".to_string(),
                updated: "NEVER".to_string(),
                rootPlaceId: 2,
                isArchived: false,
            },
        }],
    })
}
