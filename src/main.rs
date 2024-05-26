use axum::http::{Request, StatusCode};
use axum::ServiceExt;
use axum::{routing::get, Json, Router};
use axum_server::tls_rustls::RustlsConfig;
use bloxxing_match::api_404;
use std::net::SocketAddr;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use tower::util::MapRequestLayer;
use tower::Layer;
use tower_http::cors::CorsLayer;
mod auth_v1;
mod economy_v1;
mod privatemessages_v1;
mod users_v1;
fn hostroute<B>(mut request: Request<B>) -> Request<B> {
    let parts = request.into_parts();
    let newparts = parts.0.clone();
    let mut host = newparts.uri.authority().unwrap().to_string();
    let mut len = host.len();
    if host == "roblox.com".to_string() || host.split(".").collect::<Vec<&str>>().len() == 0 {
        host = "www.roblox.com".to_string();
        len = host.len();
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
        let urinary_tract_infection = parts.0.uri.scheme_str().unwrap().to_string()
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
                .get(0)
                .expect("host to have at least 1 period")
            + parts.0.uri.path_and_query().unwrap().as_str();
        // println!("uti: {:?}", urinary_tract_infection);
        builder = builder.header("Host", "roblox.com");
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
        // /:id/docs
        .nest("/privatemessages/v1/", privatemessages_v1::new())
        .nest("/auth/v1/", auth_v1::new())
        .nest("/users/v1/", users_v1::new())
        .nest("/economy/v1/", economy_v1::new())
        .layer(CorsLayer::very_permissive())
        .fallback(api_404)
        .with_state(db);
    let app = MapRequestLayer::new(hostroute).layer(app2);

    let config = RustlsConfig::from_pem_file("cert/cert.pem", "cert/key.pem")
        .await
        .unwrap();

    let addr = SocketAddr::from(([127, 0, 0, 1], 443));
    println!("listening on {}", addr);
    axum_server::bind_rustls(addr, config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Msg {
    message: String,
}

async fn root() -> (StatusCode, Json<Msg>) {
    (
        StatusCode::OK,
        Json(Msg {
            message: "OK".to_string(),
        }),
    )
}
