use axum::{
    extract::{Query, State},
    http::{HeaderName, StatusCode},
    routing::get,
    Json, Router,
};
use axum_extra::extract::cookie::CookieJar;
use bloxxing_match::{api_404, get_authenticated_user, get_cookie_object};
use http::Method;
use robacking::Roblox::Web::WebAPI::{APIError, APIErrors};
use robacking::{
    internal::{Announcement, Spawns},
    Roblox::privatemessages_v1::{
        AnnouncementsDetailsResponse, AnnouncementsMetadataResponse, GetAnnouncementsResponse,
        GetMessagesResponse, MessageDetailsResponse, UnreadMessagesCountResponse,
        VerifiedSkinnyUserResponse,
    },
};
use serde::Deserialize;
use surrealdb::{Connection, Surreal};
use time::format_description;
use tower_http::cors::CorsLayer;
pub(crate) fn new<T: Connection>() -> Router<Surreal<T>> {
    Router::new()
        .route("/announcements", get(get_announcements))
        .route("/announcements/metadata", get(get_metadata))
        .route("/messages", get(get_messages))
        .route("/messages/unread/count", get(get_unread_count))
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
async fn get_announcements<T: Connection>(
    db: State<Surreal<T>>,
    cook: CookieJar,
) -> Result<(StatusCode, Json<GetAnnouncementsResponse>), (StatusCode, Json<APIErrors>)> {
    // Ok((
    //     StatusCode::OK,
    //     Json(GetAnnouncementsResponse {
    //         totalCollectionSize: 1,
    //         collection: vec![AnnouncementsDetailsResponse {
    //             id: 1,
    //             sender: VerifiedSkinnyUserResponse {
    //                 hasVerifiedBadge: true,
    //                 id: 2,
    //                 name: "John Doe".to_string(),
    //                 displayName: "HeadOfWBAFR".to_string(),
    //             },
    //             subject: "The WBA replacement program has started".to_string(),
    //             body: "# The WBA has started a new program for replacing the web api backend of roblox!!".to_string(),
    //             created: "2024-05-25T19:16:49".to_string(),
    //             updated: "2024-05-25T19:16:49".to_string(),
    //         }],
    //     }),
    // ))
    let user = get_authenticated_user(&db, &cook).await;
    if user.is_ok() {
        let two = user.unwrap();
        match db
            .query("USE NS announces DB management; SELECT * FROM announcement")
            .await
        {
            Ok(mut a) => {
                let sels: Result<Vec<Announcement>, _> = a.take(1);
                if sels.as_ref().is_ok_and(|x| !x.is_empty()) {
                    let announces = sels.unwrap();
                    let mut nvec = Vec::with_capacity(announces.len());
                    // println!(
                    //     "unred annc: {} , annc new len: {}",
                    //     two.unread.announcements,
                    //     announces.len()
                    // );
                    if two.unread.announcements > 0 && announces.len() > 0 {
                        let r = db
                            .query(
                                "USE NS users DB userinfo; UPDATE user:".to_string()
                                    + two.userid.to_string().as_str()
                                    + " SET unread.announcements=$unread",
                            )
                            .bind((
                                "unread",
                                ((two.unread.announcements as usize) - announces.len()),
                            ))
                            .await;
                        // println!("updating went: {:?}", r);
                    }
                    for announce in announces.iter() {
                        nvec.push(AnnouncementsDetailsResponse {
                            created: announce
                                .created
                                .clone()
                                .format(&format_description::well_known::Rfc3339)
                                .unwrap(),
                            updated: announce
                                .updated
                                .clone()
                                .format(&format_description::well_known::Rfc3339)
                                .unwrap(),
                            sender: announce.sender.clone(),
                            subject: announce.subject.clone(),
                            id: announce.announcement_id,
                            body: announce.body.clone(),
                        });
                    }
                    Ok((
                        StatusCode::OK,
                        Json(GetAnnouncementsResponse {
                            collection: nvec,
                            totalCollectionSize: (announces.len().clamp(0, u32::MAX as usize)
                                as u32),
                        }),
                    ))
                } else {
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(APIErrors {
                            errors: vec![APIError {
                                code: 1,
                                message: "Internal server error".to_string(),
                                userFacingMessage: None,
                            }],
                        }),
                    ))
                }
            }
            Err(_) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(APIErrors {
                    errors: vec![APIError {
                        code: 1,
                        message: "Internal server error".to_string(),
                        userFacingMessage: None,
                    }],
                }),
            )),
        }
    } else {
        Err((
            StatusCode::UNAUTHORIZED,
            Json(APIErrors {
                errors: vec![APIError {
                    code: 0,
                    message: "Authorization has been denied for this reques".to_string(),
                    userFacingMessage: Some(
                        "hate the way that you walk, the way that you talk, the way that you dress"
                            .to_string(),
                    ),
                }],
            }),
        ))
    }
}
#[allow(non_camel_case_types)]
#[derive(Deserialize)]
enum MessageTab {
    inbox,
    sent,
    archive,
}
#[allow(non_snake_case)]
#[derive(Deserialize)]
struct Messagetab {
    messageTab: MessageTab,
}
async fn get_messages<T: Connection>(
    db: State<Surreal<T>>,
    cook: CookieJar,
    queros: Query<Messagetab>,
) -> Result<(StatusCode, Json<GetMessagesResponse>), (StatusCode, Json<APIErrors>)> {
    let user = get_authenticated_user(&db, &cook).await;
    if user.is_ok() {
        //     let two = user.unwrap();
        //     // GetMessagesResponse {
        //     //             collection: vec![MessageDetailsResponse {
        //     //                 created: "2024-05-25T20:26:51Z".to_string(),
        //     //                 sender: VerifiedSkinnyUserResponse {
        //     //                     hasVerifiedBadge: true,
        //     //                     id: 2,
        //     //                     name: "John Doe".to_string(),
        //     //                     displayName: "HeadOfWBAFR".to_string(),
        //     //                 },
        //     //                 id: 1,
        //     //                 body: "Congratulations! You're now part of the WBA replacement program!"
        //     //                     .to_string(),
        //     //                 updated: "2024-05-25T20:27:17Z".to_string(),
        //     //                 isRead: false,
        //     //                 isSystemMessage: true,
        //     //                 isReportAbuseDisplayed: true,
        //     //                 recipient: VerifiedSkinnyUserResponse {
        //     //                     id: 228176120,
        //     //                     name: "bainchild".to_string(),
        //     //                     hasVerifiedBadge: true,
        //     //                     displayName: "bainchild".to_string(),
        //     //                 },
        //     //                 subject: "You're part of the replacement program!".to_string(),
        //     //             }],
        //     //             pageNumber: 1,
        //     //             totalPages: 1,
        //     //             totalCollectionSize: 1,
        //     //         },
        //     match db
        //         .query(
        //             "USE NS spawns DB userinfo; SELECT * FROM user:".to_string()
        //                 + two.userid.to_string().as_str(),
        //         )
        //         .await
        //     {
        //         Ok(mut a) => {
        //             let spawns: Result<Vec<Spawns>, _> = a.take(1);
        //             if spawns.is_ok_and(|x| !x.is_empty()) {
        //                 let spawned = spawns.unwrap().get(0).unwrap();
        //             } else {
        //                 Err((
        //                     StatusCode::INTERNAL_SERVER_ERROR,
        //                     Json(APIErrors {
        //                         errors: vec![APIError {
        //                             code: 1,
        //                             message: "Internal server error".to_string(),
        //                             userFacingMessage: None,
        //                         }],
        //                     }),
        //                 ))
        //             }
        //         }
        // Err(_) =>
        Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(APIErrors {
                errors: vec![APIError {
                    code: 1,
                    message: "Internal server error".to_string(),
                    userFacingMessage: None,
                }],
            }),
        )) //,
           // }
    } else {
        Err((
            StatusCode::UNAUTHORIZED,
            Json(APIErrors {
                errors: vec![APIError {
                    code: 0,
                    message: "Authorization has been denied for this request".to_string(),
                    userFacingMessage: Some("log in.".to_string()),
                }],
            }),
        ))
    }
}
async fn get_metadata<T: Connection>(
    db: State<Surreal<T>>,
    cook: CookieJar,
) -> Result<(StatusCode, Json<AnnouncementsMetadataResponse>), (StatusCode, Json<APIErrors>)> {
    let user = get_authenticated_user(&db, &cook).await;
    println!("get metadata {:?}  + + {:?}", user, cook);
    if user.is_ok() {
        let twoser = user.unwrap();
        Ok((
            StatusCode::OK,
            Json(AnnouncementsMetadataResponse {
                numOfAnnouncements: twoser.unread.announcements,
            }),
        ))
    } else {
        Err((
            StatusCode::UNAUTHORIZED,
            Json(APIErrors {
                errors: vec![APIError {
                    code: 0,
                    message: "Authorization has been denied for this request".to_string(),
                    userFacingMessage: Some("log4 in.".to_string()),
                }],
            }),
        ))
    }
}
async fn get_unread_count<T: Connection>(
    db: State<Surreal<T>>,
    cook: CookieJar,
) -> Result<(StatusCode, Json<UnreadMessagesCountResponse>), (StatusCode, Json<APIErrors>)> {
    let user = get_authenticated_user(&db, &cook).await;
    if user.is_ok() {
        let twoser = user.unwrap();
        Ok((
            StatusCode::OK,
            Json(UnreadMessagesCountResponse {
                count: twoser.unread.messages,
            }),
        ))
    } else {
        Err((
            StatusCode::UNAUTHORIZED,
            Json(APIErrors {
                errors: vec![APIError {
                    code: 0,
                    message: "Authorization has been denied for this request".to_string(),
                    userFacingMessage: Some("log in.".to_string()),
                }],
            }),
        ))
    }
}
