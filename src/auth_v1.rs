use axum::{
    extract::{rejection::JsonRejection, State},
    http::{HeaderName, StatusCode},
    routing::post,
    Json, Router,
};
use axum_extra::extract::CookieJar;
use bloxxing_match::api_404;
use http::Method;
use robacking::{
    internal::User,
    Roblox::auth_v1::{
        LoginRequest, LoginResponse, SignupRequest, SignupResponse, SkinnyUserResponse,
    },
};
use robacking::{
    internal::{AvatarInfo, EmperorsNewDefault},
    Roblox::Web::WebAPI::{APIError, APIErrors},
};
use surrealdb::{Connection, Response, Surreal};
use tower_http::cors::CorsLayer;
pub(crate) fn new<T: Connection>() -> Router<Surreal<T>> {
    Router::new()
        .route("/signup", post(signup))
        .route("/login", post(login))
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
async fn set_counter<T: Connection>(
    db: &Surreal<T>,
    count: u64,
    code: u64,
    message: String,
) -> Result<(), (StatusCode, Json<APIErrors>)> {
    match db
        .query("USE NS users DB userinfo; UPDATE metainfo:0 SET latest_userid=$count")
        .bind(("count", count))
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("latest userid update error {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(APIErrors {
                    errors: vec![APIError {
                        code,
                        message,
                        userFacingMessage: Some("Something went wrong.".to_string()),
                    }],
                }),
            ))
        }
    }
}
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use time::{format_description, OffsetDateTime};
fn hash_argon2(str: String) -> Result<String, argon2::password_hash::Error> {
    let password = str.as_bytes();
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password, &salt)?.to_string();
    return Ok(password_hash);
}
async fn get_counter<T: Connection>(
    db: &Surreal<T>,
    code: u64,
    message: String,
) -> Result<u64, (StatusCode, Json<APIErrors>)> {
    let res: Result<Response, surrealdb::Error> = db
        .query("USE NS users DB userinfo; SELECT latest_userid FROM metainfo:0")
        .await;
    match res.and_then(|x| x.check()) {
        Ok(mut resp) => {
            let w: Result<Option<u64>, _> = resp.take((1, "latest_userid"));
            if w.is_ok() && w.as_ref().unwrap().is_some() {
                Ok(w.unwrap().unwrap().to_owned())
            } else {
                if w.is_err() {
                    println!("error getting the count: {:?}", w.unwrap_err());
                }
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(APIErrors {
                        errors: vec![APIError {
                            code,
                            message,
                            userFacingMessage: Some("Something went wrong.".to_string()),
                        }],
                    }),
                ))
            }
        }
        Err(e) => {
            println!("error getting the count 2: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(APIErrors {
                    errors: vec![APIError {
                        code,
                        message,
                        userFacingMessage: Some("Something went wrong.".to_string()),
                    }],
                }),
            ))
        }
    }
}
async fn signup<T: Connection>(
    db: State<Surreal<T>>,
    infos: Result<Json<SignupRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<SignupResponse>), (StatusCode, Json<APIErrors>)> {
    // 400:
    //   16: user agreement ids are null
    //   21: empty account switch blob required
    // 403:
    //   0: token invalid
    //   2: captcha failed
    //   4: invalid birthday
    //   5: invalid username
    //   6: username already taken
    //   7: invalid password
    //   8: password and username are same
    //   9: password is too simple
    //  10: email is invalid
    //  11: asset is invalid
    //  12: too many attempts
    //  17: otp session not valid
    //  22: maximum logged in accounts reached
    // 429 3: too many attempts
    if infos.is_ok() {
        let inf2s = infos.unwrap();
        let id = get_counter(&db, 0, "Error allocating userid".to_string()).await? + 1;
        println!("{:?}", inf2s);
        match db
            .query(
                "USE NS users DB userinfo; CREATE ".to_string()
                    + "user:"
                    + id.to_string().as_str()
                    + " CONTENT $user",
            )
            .bind((
                "user",
                User {
                    userid: id,
                    robux: 5,
                    gender: inf2s.gender,
                    birth_date: EmperorsNewDefault::new(
                        OffsetDateTime::parse(
                            inf2s.birthday.as_str(),
                            &format_description::well_known::Rfc3339,
                        )
                        .unwrap(),
                    ),
                    username: inf2s.username.clone(),
                    password: match hash_argon2(inf2s.password.clone()) {
                        Ok(p) => p,
                        Err(e) => {
                            println!("signup error {:?}", e);
                            return Err((
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(APIErrors {
                                    errors: vec![APIError {
                                        code: 15,
                                        message: "Insert acceptances failed".to_string(),
                                        userFacingMessage: Some(
                                            "Something went wrong.".to_string(),
                                        ),
                                    }],
                                }),
                            ));
                        }
                    },
                    display_name: {
                        let ndn = inf2s.displayName.clone();
                        if ndn == "".to_string() {
                            None
                        } else {
                            Some(ndn)
                        }
                    },
                    profile_description: "".to_string(),
                    creation_date: EmperorsNewDefault::new(OffsetDateTime::now_utc()),
                    is_verified: false,
                    is_banned: false,
                    avatar: AvatarInfo::default(),
                    ..Default::default()
                },
            ))
            .await
        {
            Ok(_) => {}
            Err(e) => {
                println!("signup error 2 {:?}", e);
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(APIErrors {
                        errors: vec![APIError {
                            code: 15,
                            message: "Insert acceptances failed".to_string(),
                            userFacingMessage: Some("Something went wrong.".to_string()),
                        }],
                    }),
                ));
            }
        };
        set_counter(&db, id + 1, 0, "Error allocating userid".to_string()).await?;
        Ok((
            StatusCode::OK,
            Json(SignupResponse {
                accountBlob: "blobbing..".to_string(),
                returnUrl: "https://example.com".to_string(),
                userId: id,
                starterPlaceId: 0,
            }),
        ))
    } else {
        Err(api_404().await)
    }
}
async fn login<T: Connection>(
    _db: State<Surreal<T>>,
    _cook: CookieJar,
    infos: Result<Json<LoginRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<LoginResponse>), (StatusCode, Json<APIErrors>)> {
    if infos.is_ok() {
        let inf2s = infos.unwrap();
        println!("sign in {:?}", inf2s);
        Ok((
            StatusCode::OK,
            Json(LoginResponse {
                user: SkinnyUserResponse {
                    id: 228176120,
                    name: "bainchild".to_string(),
                    displayName: "".to_string(),
                },
                twoStepVerificationData: None,
                identityVerificationLoginTicket: None,
                isBanned: false,
                accountBlob: "blobbing".to_string(),
            }),
        ))
    } else {
        Err(api_404().await)
    }
}
