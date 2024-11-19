#![feature(future_join)]
use axum::extract::Query;
use axum::routing::post;
use axum::ServiceExt;
use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, head},
    Json, Router,
};
use axum_extra::extract::CookieJar;
use axum_server::tls_rustls::RustlsConfig;
use bloxxing_match::{api_404, make_error, ProductInfo};
use bloxxing_match::{get_asset_from_id, get_authenticated_user};
use http::Request;
use http::{HeaderMap, HeaderName};
use robacking::Roblox::auth_v1::SkinnyUserResponse;
use robacking::Roblox::develop_v1::UniverseModel;
use robacking::Roblox::Users::Api::AuthenticatedUserResponse;
use robacking::Roblox::Web::WebAPI::APIErrors;
use rustls::crypto::CryptoProvider;
use serde::{Deserialize, Serialize};
use std::future::join;
use std::net::SocketAddr;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::{Connection, Surreal};
use tower::util::MapRequestLayer;
use tower::Layer;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
mod assetdelivery_v1;
mod assetgame;
mod auth_v1;
mod auth_v2;
mod clientsettings_v1;
mod clientsettings_v2;
mod develop_v1;
mod devforum;
mod economy_v1;
mod ecsv2;
mod ephemeralcounters_api;
mod gamepersistence;
mod locale_v1;
mod privatemessages_v1;
mod users_v1;
mod versioncompatibility_api;
fn hostroute<B>(mut request: Request<B>) -> Request<B> {
    let parts = request.into_parts();
    let newparts = parts.0.clone();
    let mut host;
    match newparts.uri.host() {
        Some(h) => {
            host = h.to_string();
        }
        None => {
            // println!("host header: {:?}", newparts.headers.get("Host"));
            match newparts.headers.get("host") {
                Some(h) => match h.to_str() {
                    Ok(a) => {
                        host = a.to_string();
                    }
                    Err(_) => {
                        return Request::from_parts(parts.0, parts.1);
                    }
                },
                None => {
                    return Request::from_parts(parts.0, parts.1);
                }
            }
            // println!("no uri host??? {:?}", newparts.uri);
            // host = "clientsettingscdn.roblox.com".to_string();
        }
    }
    let igothooooost = host.clone();
    let mut len = host.len();
    if host == "roblox.com" || host.split(".").collect::<Vec<&str>>().is_empty() {
        host = "www.roblox.com".to_string();
        len = host.len();
    }
    if host == "www.roblox.com" && {
        let new = parts.0.uri.path().split("/").collect::<Vec<&str>>();
        if new.get(1).is_some() {
            let firs = new.get(1).unwrap().to_lowercase();
            firs == "game" // || (firs == "login".to_string() && new.len() > 1)
        } else {
            false
        }
    } {
        host = "assetgame.roblox.com".to_string();
        len = host.len();
    }
    if host == "clientsettingscdn.roblox.com" {
        host = "clientsettings.roblox.com".to_string();
        len = host.len();
    }
    if host == "setup.rbxcdn.com" {
        println!("headers: {:?}", newparts.headers);
    }
    if host != "ecsv2.roblox.com" && host != "ephemeralcounters.api.roblox.com" {
        println!("{} {:?}", newparts.method, newparts.uri);
        if igothooooost != host {
            println!(
                "Host: {:?} -> {:?}",
                igothooooost.to_string(),
                host.to_string()
            );
        } else {
            println!("Host: {:?}", host.to_string());
        }
    }
    // println!("b4 uri: {} ({})", newparts.uri.to_string(), {
    //     if len < 11 {
    //         host.clone()
    //     } else {
    //         host.get((len - 11)..).unwrap().to_string()
    //     }
    // });
    if len > 11 && host.get((len - 11)..).unwrap() == ".roblox.com" {
        // println!("b4 host: {:?}", host.clone());
        let mut builder = Request::builder()
            .method(parts.0.method)
            .version(parts.0.version);
        // if parts.0.extensions.get::<T>().is_some() {
        //     builder.extension(parts.0.extensions.get().unwrap());
        // }
        for (head, val) in parts.0.headers.iter() {
            builder = builder.header(head, val);
        }
        let urinary_tract_infection = parts.0.uri.scheme_str().unwrap_or("https").to_string()
            + "://"
            + host
                .split(".")
                // .skip(1)
                .collect::<Vec<&str>>()
                .join(".")
                .as_str()
            + "/"
            + host
                .split(".")
                .collect::<Vec<&str>>()
                .first()
                .expect("host to have at least 1 period")
            + parts.0.uri.path_and_query().unwrap().as_str();
        // println!("uti: {:?}", urinary_tract_infection);
        builder = builder.header("Host", "roblox.com");
        request = builder.uri(urinary_tract_infection).body(parts.1).unwrap();
    } else if len > 11 && host.get((len - 11)..).unwrap() == ".rbxcdn.com" {
        // println!("b4 host: {:?}", host.clone());
        let mut builder = Request::builder()
            .method(parts.0.method)
            .version(parts.0.version);
        // if parts.0.extensions.get::<T>().is_some() {
        //     builder.extension(parts.0.extensions.get().unwrap());
        // }
        for (head, val) in parts.0.headers.iter() {
            builder = builder.header(head, val);
        }
        let urinary_tract_infection = parts.0.uri.scheme_str().unwrap_or("https").to_string()
            + "://"
            + host
                .split(".")
                // .skip(1)
                .collect::<Vec<&str>>()
                .join(".")
                .as_str()
            + "/"
            + host
                .split(".")
                .collect::<Vec<&str>>()
                .first()
                .expect("host to have at least 1 period")
            + parts.0.uri.path_and_query().unwrap().as_str();
        // println!("uti: {:?}", urinary_tract_infection);
        builder = builder.header("Host", "rbxcdn.com");
        request = builder.uri(urinary_tract_infection).body(parts.1).unwrap();
    } else {
        request = Request::from_parts(parts.0, parts.1);
    }
    // println!("a4 uri: {}", request.uri().to_string());
    // for (a, b) in request.headers() {
    //     println!("a4 header {:?} {:?}", a, b);
    // }
    request
}
#[tokio::main]
async fn main() {
    let db: Surreal<Client> = match Surreal::new::<Ws>("localhost:8000").await {
        Ok(d) => d,
        Err(e) => {
            println!("Error connecting to db: {}", e);
            std::process::exit(1);
        }
    };
    match db
        .signin(Root {
            username: "root",
            password: "root",
        })
        .await
    {
        Ok(_) => {}
        Err(e) => {
            println!("Error logging in to db: {}", e);
            std::process::exit(2);
        }
    }
    let app2 = Router::new()
        .route("/", get(root))
        .route("/www/login/RequestAuth.ashx", get(request_auth))
        .route("/www/login/NegotiateAuth.ashx", get(negotiate_auth))
        .route("/api/users/account-info", get(account_info))
        .route("/api/device/initialize", post(device_initialize))
        .route("/api/marketplace/productinfo", get(product_info))
        .route(
            "/api/universes/get-universe-containing-place",
            get(universe_containing_place),
        )
        .route(
            "/api/users/:uid/canmanage/:assetid",
            get(user_can_manage_asset),
        )
        // /:id/docs
        .route("/setup/version", head(okay))
        .nest_service("/setup/", ServeDir::new("setup.rbxcdn.com"))
        .nest_service("/tr/", ServeDir::new("assets")) // sc2.rbxcdn.com, for assets
        .nest_service("/blog/", ServeDir::new("blog"))
        .nest("/assetgame/", assetgame::new())
        .nest("/devforum/", devforum::new())
        .nest("/ecsv2/", ecsv2::new())
        .nest("/ephemeralcounters/", ephemeralcounters_api::new())
        .nest("/versioncompatibility/", versioncompatibility_api::new())
        .nest("/gamepersistence/", gamepersistence::new())
        .nest("/locale/v1/", locale_v1::new())
        .nest("/assetdelivery/v1/", assetdelivery_v1::new())
        .nest("/auth/v1/", auth_v1::new())
        .nest("/auth/v2/", auth_v2::new())
        .nest("/clientsettings/v1/", clientsettings_v1::new())
        .nest("/clientsettings/v2/", clientsettings_v2::new())
        .nest("/economy/v1/", economy_v1::new())
        .nest("/develop/v1/", develop_v1::new())
        .nest("/privatemessages/v1/", privatemessages_v1::new())
        .nest("/users/v1/", users_v1::new())
        .layer(CorsLayer::very_permissive())
        .fallback(api_404)
        .with_state(db);
    let app = MapRequestLayer::new(hostroute).layer(app2);

    let cert_is_good = std::fs::exists("cert/cert.pem")
        .and(std::fs::exists("cert/key.pem"))
        .is_ok_and(|x| x);
    println!(
        "listening on 127.0.0.1 (https is {}enabled)",
        (if cert_is_good { "" } else { "NOT " })
    );
    if cert_is_good {
        let _ = CryptoProvider::install_default(rustls::crypto::aws_lc_rs::default_provider());
        let config = RustlsConfig::from_pem_file("cert/cert.pem", "cert/key.pem")
            .await
            .unwrap();
        let (res1, res2) = join!(
            axum_server::bind(SocketAddr::from(([127, 0, 0, 1], 80)))
                .serve(app.clone().into_make_service()),
            axum_server::bind_rustls(SocketAddr::from(([127, 0, 0, 1], 443)), config)
                .serve(app.into_make_service())
        )
        .await;
        res1.unwrap();
        res2.unwrap();
    } else {
        axum_server::bind(SocketAddr::from(([127, 0, 0, 1], 80)))
            .serve(app.clone().into_make_service())
            .await
            .unwrap();
    }
}
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct UserCanManageAsset {
    pub CanManage: bool,
}
async fn user_can_manage_asset() -> Json<UserCanManageAsset> {
    Json(UserCanManageAsset { CanManage: true })
}
async fn device_initialize(head: HeaderMap, str: String) -> StatusCode {
    // println!("head {:?}", head);
    // println!("strings {:?}", str);
    StatusCode::OK
}
#[derive(Serialize, Deserialize)]
pub struct AssetIdQuery {
    pub assetid: u64,
}
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct UniverseContainingPlaceQuery {
    pub placeId: u64,
}
async fn universe_containing_place<T: Connection>(
    db: State<Surreal<T>>,
    q: Query<UniverseContainingPlaceQuery>,
) -> Result<Json<UniverseModel>, (StatusCode, Json<APIErrors>)> {
    if q.placeId == 2 {
        Ok(Json(UniverseModel {
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
        }))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(make_error(1, "Not found", None)),
        ))
    }
}
async fn product_info<T: Connection>(
    db: State<Surreal<T>>,
    q: Query<AssetIdQuery>,
) -> Result<Json<ProductInfo>, StatusCode> {
    let asset = get_asset_from_id(&db, q.assetid).await;
    match asset {
        Ok(ast) => Ok(Json(ProductInfo {
            Name: ast.title,
            PriceInRobux: ast.cost,
            Created: "".to_string(),
            Updated: "".to_string(),
            ContentRatingTypeId: 0,
            MinimumMembershipLevel: 0,
            IsPublicDomain: true,
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn account_info<T: Connection>(
    db: State<Surreal<T>>,
    cook: CookieJar,
) -> Result<(StatusCode, Json<SkinnyUserResponse>), StatusCode> {
    let user = get_authenticated_user(&db, &cook).await;
    if user.is_ok() {
        let usear = user.unwrap();
        Ok((
            StatusCode::OK,
            Json(SkinnyUserResponse {
                id: usear.userid,
                name: usear.username,
                displayName: usear.display_name.unwrap_or("".to_string()),
            }),
        ))
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

async fn request_auth() -> String {
    "https://www.roblox.com/login/NegotiateAuth.ashx".to_string()
}
async fn negotiate_auth<T: Connection>(
    db: State<Surreal<T>>,
    cook: CookieJar,
) -> Result<(StatusCode, HeaderMap, Json<AuthenticatedUserResponse>), (StatusCode, Json<APIErrors>)>
{
    let user = get_authenticated_user(&db, &cook).await;
    println!("/login/NegotiateAuth.ashx handler thing {:?}", user);
    if user.is_ok() {
        let usear = user.unwrap();
        let mut newheads = HeaderMap::new();
        newheads.insert(
            "RBXAuthenticationNegotiation"
                .parse::<HeaderName>()
                .unwrap(),
            "1".parse().unwrap(),
        );
        Ok((
            StatusCode::OK,
            newheads,
            Json(AuthenticatedUserResponse {
                id: usear.userid,
                name: usear.username,
                displayName: usear.display_name.unwrap_or("".to_string()),
            }),
        ))
    } else {
        let err = user.unwrap_err();
        Err((
            match err.errors.first().unwrap().code {
                0 => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            },
            Json(err),
        ))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Msg {
    message: String,
}
async fn okay() -> StatusCode {
    StatusCode::OK
}

async fn root() -> (StatusCode, Json<Msg>) {
    (
        StatusCode::OK,
        Json(Msg {
            message: "OK".to_string(),
        }),
    )
}
