use axum::{
    extract::{rejection::PathRejection, Path, State},
    http::{HeaderName, HeaderValue, StatusCode},
    routing::get,
    Json, Router,
};
use axum_extra::extract::CookieJar;
use bloxxing_match::{api_404, get_authenticated_user, get_user_from_id};
use http::Method;
// use robacking::internal::User;
use robacking::Roblox::{
    users_v1::{AuthenticatedUserResponse, GenderResponse, UserRolesResponse},
    Web::WebAPI::{APIError, APIErrors},
};
use robacking::Roblox::{
    users_v1::{BirthdateResponse, DescriptionResponse},
    Users::Api::GetUserResponse,
};
use surrealdb::{Connection, Surreal};
use tower_http::cors::CorsLayer;
pub(crate) fn new<T: Connection>() -> Router<Surreal<T>> {
    Router::new()
        .route("/users/authenticated", get(get_authed_user))
        .route("/users/:id", get(get_user))
        .route("/users/:id/", get(get_user))
        .route("/birthdate", get(get_auth_birthdate))
        .route("/description", get(get_auth_description))
        .route("/gender", get(get_auth_gender))
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
async fn invalid_userid() -> (StatusCode, Json<APIErrors>) {
    (
        StatusCode::NOT_FOUND,
        Json(APIErrors {
            errors: vec![APIError {
                code: 3,
                message: "The user id is invalid".to_string(),
                userFacingMessage: Some("Something went wrong".to_string()),
            }],
        }),
    )
}
async fn get_authed_user<T: Connection>(
    db: State<Surreal<T>>,
    cook: CookieJar,
) -> Result<(StatusCode, Json<AuthenticatedUserResponse>), (StatusCode, Json<APIErrors>)> {
    let user = get_authenticated_user(&db, &cook).await;
    if user.is_ok() {
        let usear = user.unwrap();
        Ok((
            StatusCode::OK,
            Json(AuthenticatedUserResponse {
                id: usear.userid,
                name: usear.username,
                displayName: usear.display_name.unwrap_or("".to_string()),
            }),
        ))
    } else {
        let err = user.unwrap_err();
        Err((
            match err.errors.get(0).unwrap().code {
                0 => StatusCode::UNAUTHORIZED,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            },
            Json(err),
        ))
    }
}
// async fn get_auth_user_roles<T: Connection>(
//     db: State<Surreal<T>>,
//     cook: CookieJar,
// ) -> Result<(StatusCode, Json<UserRolesResponse>), (StatusCode, Json<APIErrors>)> {
//     let user = get_authenticated_user(&db, &cook).await;
//     if user.is_ok() {
//         let usear = user.unwrap();
//         Ok((
//             StatusCode::OK,
//             Json(UserRolesResponse { roles: usear.roles }),
//         ))
//     } else {
//         let err = user.unwrap_err();
//         Err((
//             match err.errors.get(0).unwrap().code {
//                 0 => StatusCode::UNAUTHORIZED,
//                 _ => StatusCode::INTERNAL_SERVER_ERROR,
//             },
//             Json(err),
//         ))
//     }
// }
async fn get_auth_birthdate<T: Connection>(
    db: State<Surreal<T>>,
    cook: CookieJar,
) -> Result<(StatusCode, Json<BirthdateResponse>), (StatusCode, Json<APIErrors>)> {
    let user = get_authenticated_user(&db, &cook).await;
    if user.is_ok() {
        let usear = user.unwrap();
        Ok((
            StatusCode::OK,
            Json(BirthdateResponse {
                birthYear: usear.birth_date.year().unsigned_abs(),
                birthMonth: match usear.birth_date.month() {
                    time::Month::January => 1,
                    time::Month::February => 2,
                    time::Month::March => 3,
                    time::Month::April => 4,
                    time::Month::May => 5,
                    time::Month::June => 6,
                    time::Month::July => 7,
                    time::Month::August => 8,
                    time::Month::September => 9,
                    time::Month::October => 10,
                    time::Month::November => 11,
                    time::Month::December => 12,
                },
                birthDay: usear.birth_date.day().into(),
            }),
        ))
    } else {
        let err = user.unwrap_err();
        Err((
            match err.errors.get(0).unwrap().code {
                0 => StatusCode::UNAUTHORIZED,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            },
            Json(err),
        ))
    }
}
async fn get_auth_description<T: Connection>(
    db: State<Surreal<T>>,
    cook: CookieJar,
) -> Result<(StatusCode, Json<DescriptionResponse>), (StatusCode, Json<APIErrors>)> {
    let user = get_authenticated_user(&db, &cook).await;
    if user.is_ok() {
        let usear = user.unwrap();
        Ok((
            StatusCode::OK,
            Json(DescriptionResponse {
                description: usear.profile_description,
            }),
        ))
    } else {
        let err = user.unwrap_err();
        Err((
            match err.errors.get(0).unwrap().code {
                0 => StatusCode::UNAUTHORIZED,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            },
            Json(err),
        ))
    }
}
async fn get_auth_gender<T: Connection>(
    db: State<Surreal<T>>,
    cook: CookieJar,
) -> Result<(StatusCode, Json<GenderResponse>), (StatusCode, Json<APIErrors>)> {
    let user = get_authenticated_user(&db, &cook).await;
    if user.is_ok() {
        let usear = user.unwrap();
        Ok((
            StatusCode::OK,
            Json(GenderResponse {
                gender: usear.gender,
            }),
        ))
    } else {
        let err = user.unwrap_err();
        Err((
            match err.errors.get(0).unwrap().code {
                0 => StatusCode::UNAUTHORIZED,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            },
            Json(err),
        ))
    }
}
async fn get_user<T: Connection>(
    db: State<Surreal<T>>,
    userid: Result<Path<u64>, PathRejection>,
) -> Result<(StatusCode, Json<GetUserResponse>), (StatusCode, Json<APIErrors>)> {
    if userid.is_ok() {
        let Path(id) = userid.unwrap();
        let user = get_user_from_id(&db, id).await;
        if user.is_ok() {
            let theuser = user.unwrap();
            Ok((
                StatusCode::OK,
                Json(GetUserResponse {
                    created: theuser
                        .creation_date
                        .format(&time::format_description::well_known::Rfc3339)
                        .unwrap(),
                    externalAppDisplayName: theuser.display_name.clone(),
                    description: theuser.profile_description.clone(),
                    name: theuser.username.clone(),
                    id,
                    hasVerifiedBadge: theuser.is_verified,
                    isBanned: theuser.is_banned,
                    displayName: theuser.display_name.clone().unwrap_or("".to_string()),
                }), // Json(GetUserResponse {
                    //     description: "Really real?",
                    //     created: "2006-02-27T21:06:40.3Z",
                    //     isBanned: false,
                    //     externalAppDisplayName: None,
                    //     hasVerifiedBadge: true,
                    //     id: 1234,
                    //     name: "Roguy",
                    //     displayName: "John doe",
                    // }),
            ))
        } else {
            match user.unwrap_err().errors.get(0).unwrap().code {
                1 => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(APIErrors {
                        errors: vec![APIError {
                            code: 0,
                            message: "Internal server error".to_string(),
                            userFacingMessage: Some("Something went wrong.".to_string()),
                        }],
                    }),
                )),
                _ => Err(invalid_userid().await),
            }
        }
    } else {
        Err(api_404().await)
    }
}
