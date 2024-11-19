use axum::{
    body::Bytes,
    extract::{rejection::JsonRejection, FromRequest, Path, Query, State},
    http::{HeaderName, StatusCode},
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::cookie::CookieJar;
use bloxxing_match::{api_404, get_authenticated_user, get_user_from_id};
use http::{Method, Request};
use rand::Rng;
use robacking::{
    internal::{Announcement, Spawns},
    Roblox::privatemessages_v1::{
        AnnouncementsDetailsResponse, AnnouncementsMetadataResponse, GetAnnouncementsResponse,
        GetMessagesResponse, MessageDetailsResponse, SendMessageRequestProper,
        UnreadMessagesCountResponse, VerifiedSkinnyUserResponse,
    },
};
use robacking::{
    internal::{EmperorsNewDefault, Message},
    Roblox::{
        privatemessages_v1::{CanMessageResponse, SendMessageRequest, SendMessageResponse},
        Web::WebAPI::{APIError, APIErrors},
    },
};
use serde::Deserialize;
use surrealdb::{Connection, Surreal};
use time::{format_description, OffsetDateTime};
use tower_http::cors::CorsLayer;
pub(crate) fn new<T: Connection>() -> Router<Surreal<T>> {
    Router::new()
        .route("/announcements", get(get_announcements))
        .route("/announcements/metadata", get(get_metadata))
        .route("/messages", get(get_messages))
        .route("/messages/send", post(send_message))
        .route("/messages/unread/count", get(get_unread_count))
        .route("/messages/:id/can-message", get(can_message))
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
                        /* let r=*/
                        // we don't care cause unread isn't important
                        let _ = db
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
                    userFacingMessage: Some("log in".to_string()),
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
        let two = user.unwrap();
        // match db
        //     .query("USE NS announces DB management; SELECT * FROM announcement")
        //     .await
        // {
        //     Ok(mut a) => {
        //         let sels: Result<Vec<Announcement>, _> = a.take(1);
        //         if sels.as_ref().is_ok_and(|x| !x.is_empty()) {
        //             let announces = sels.unwrap();
        //         } else {

        //         }
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
        match db
            .query(
                "USE NS spawns DB userinfo; SELECT * FROM user:".to_string()
                    + two.userid.to_string().as_str()
                    + "; USE NS users DB userinfo; UPDATE user:"
                    + two.userid.to_string().as_str()
                    + " SET unread.messages=0",
            )
            .await
        {
            Ok(mut a) => {
                let spawns: Result<Vec<Spawns>, _> = a.take(1);
                if spawns.as_ref().is_ok_and(|x| !x.is_empty()) {
                    let targ = match queros.messageTab {
                        MessageTab::sent => spawns.unwrap().get(0).unwrap().sent_messages.clone(),
                        MessageTab::inbox => spawns.unwrap().get(0).unwrap().inbox_messages.clone(),
                        MessageTab::archive => {
                            spawns.unwrap().get(0).unwrap().archive_messages.clone()
                        }
                    };
                    let mut msgs = Vec::new();
                    for msg in targ {
                        msgs.push(MessageDetailsResponse {
                            created: msg
                                .created
                                .format(&format_description::well_known::Rfc3339)
                                .unwrap(),
                            sender: msg.sender.clone(),
                            id: msg.message_id,
                            body: msg.body.clone(),
                            updated: msg
                                .updated
                                .format(&format_description::well_known::Rfc3339)
                                .unwrap(),
                            isRead: msg.is_read,
                            isSystemMessage: msg.system,
                            isReportAbuseDisplayed: msg.can_be_reported,
                            recipient: msg.recipient.clone(),
                            subject: msg.subject.clone(),
                        });
                    }
                    Ok((
                        StatusCode::OK,
                        Json(GetMessagesResponse {
                            pageNumber: 1,
                            totalPages: 1,
                            totalCollectionSize: (msgs.len() as u64),
                            collection: msgs,
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
            )), //,
        }
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
// async fn mark_read<T: Connection>(
//     db: State<Surreal<T>>,
//     cook: CookieJar,
//     messages: Json<BatchMessagesRequest>,
// ) -> Result<(StatusCode, Json<BatchMessagesResponse>), (StatusCode, Json<APIErrors>)> {
//     let usear = get_authenticated_user(&db, &cook).await;
//     match usear {
//         Ok(user) => {
//             todo!()
//             // let mut failed = Vec::new();
//             // // todo: get messages, modify em based on msg.messageId
//             // match db.query("USE NS spawns DB userinfo; UPDATE user:".to_string()+user.userid.to_string().as_str()+" CONTENT $modified").bind(("modified",modified)).await {
//             //     Ok(_) => todo!(),
//             //     Err(_) => todo!(),
//             // }
//             // Ok((
//             //     StatusCode::OK,
//             //     Json(BatchMessagesResponse {
//             //         failedMessages: failed,
//             //     }),
//             // ))
//         }
//         Err(_) => Err((
//             StatusCode::UNAUTHORIZED,
//             Json(APIErrors {
//                 errors: vec![APIError {
//                     code: 1,
//                     message: "Authorization has been denied for this request".to_string(),
//                     userFacingMessage: None,
//                 }],
//             }),
//         )),
//     }
// }
async fn send_message<T: Connection>(
    db: State<Surreal<T>>,
    cook: CookieJar,
    body2: Bytes,
) -> Result<(StatusCode, Json<SendMessageResponse>), (StatusCode, Json<APIErrors>)> {
    let useafterfreer = get_authenticated_user(&db, &cook).await;
    let sendreq: Result<Json<SendMessageRequestProper>, JsonRejection> = Json::from_request(
        Request::builder()
            .header("Content-Type", "application/json")
            .method("POST")
            .body(body2.clone().into())
            .unwrap(),
        &db,
    )
    .await;
    let send: Json<SendMessageRequestProper>;
    if sendreq.is_err() {
        // attempt to parse it as sendmessagerequest (generated by roblox's compose menu)
        // , opposed to proper sendmessagerequest, which is used in the docs & reply feature
        let err = sendreq.unwrap_err();
        if let JsonRejection::JsonDataError(_) = err {
            let r2: Result<Json<SendMessageRequest>, JsonRejection> = Json::from_request(
                Request::builder()
                    .header("Content-Type", "application/json")
                    .method("POST")
                    .body(body2.clone().into())
                    .unwrap(),
                &db,
            )
            .await;
            if r2.is_ok() {
                let smrbad = r2.unwrap();
                let recpid = smrbad.recipientid.parse::<u64>();
                if recpid.is_err() {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        Json(APIErrors {
                            errors: vec![APIError {
                                code: 1,
                                message: "Bad recipient id.".to_string(),
                                userFacingMessage: None,
                            }],
                        }),
                    ));
                }
                let uid: u64;
                if smrbad.userId.is_some() {
                    uid = smrbad.userId.unwrap()
                } else {
                    if useafterfreer.is_ok() {
                        uid = useafterfreer.as_ref().unwrap().userid
                    } else {
                        return Err((
                            StatusCode::BAD_REQUEST,
                            Json(APIErrors {
                                errors: vec![APIError {
                                    code: 1,
                                    message: "No user id.".to_string(),
                                    userFacingMessage: None,
                                }],
                            }),
                        ));
                    }
                }
                send = Json(SendMessageRequestProper {
                    replyMessageId: smrbad.replyMessageId,
                    subject: smrbad.subject.clone(),
                    includePreviousMessage: smrbad.includePreviousMessage,
                    recipientId: recpid.unwrap(),
                    userId: uid,
                    body: smrbad.body.clone(),
                });
            } else {
                println!("double fault {:?}", r2);
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(APIErrors {
                        errors: vec![APIError {
                            code: 1,
                            message: "Internal Server Error".to_string(),
                            userFacingMessage: None,
                        }],
                    }),
                ));
            }
        } else {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(APIErrors {
                    errors: vec![APIError {
                        code: 1,
                        message: "Bad json".to_string(),
                        userFacingMessage: None,
                    }],
                }),
            ));
        }
    } else {
        send = sendreq.unwrap()
    }
    let recpiece = get_user_from_id(&db, send.recipientId).await;
    println!("recp: {:?} user: {:?}", recpiece, useafterfreer);
    if useafterfreer.is_ok() && recpiece.is_ok() {
        let recp = recpiece.unwrap();
        let user = useafterfreer.unwrap();
        println!("{:?}", send);
        let msg = Message {
            sender: VerifiedSkinnyUserResponse {
                id: user.userid,
                name: user.username,
                hasVerifiedBadge: false,
                displayName: user.display_name.unwrap_or("".to_string()),
            },
            recipient: VerifiedSkinnyUserResponse {
                id: recp.userid,
                name: recp.username,
                hasVerifiedBadge: false,
                displayName: recp.display_name.unwrap_or("".to_string()),
            },
            message_id: (rand::thread_rng().gen::<u32>() as u64),
            body: send.body.clone(),
            subject: send.subject.clone(),
            created: EmperorsNewDefault::new(OffsetDateTime::now_utc()),
            updated: EmperorsNewDefault::new(OffsetDateTime::now_utc()),
            system: false,
            is_read: false,
            can_be_reported: true,
        };
        let res = db
            .query(
                "USE NS users DB userinfo; UPDATE user:".to_string()
                    + user.userid.to_string().as_str()
                    + " SET unread.messages+=1; UPDATE user:"
                    + recp.userid.to_string().as_str()
                    + " SET unread.messages+=1; "
                    + "USE NS spawns DB userinfo; UPDATE user:"
                    + user.userid.to_string().as_str()
                    + " MERGE $send; UPDATE user:"
                    + recp.userid.to_string().as_str()
                    + " MERGE $inbox",
            )
            .bind((
                "send",
                Spawns {
                    sent_messages: vec![{
                        let mut m = msg.clone();
                        m.is_read = true;
                        m
                    }],
                    ..Default::default()
                },
            ))
            .bind((
                "inbox",
                Spawns {
                    inbox_messages: vec![msg],
                    ..Default::default()
                },
            ))
            .await;
        println!("the query did something : {:?}", res);
        match res {
            Ok(_) => Ok((
                StatusCode::OK,
                Json(SendMessageResponse {
                    message: "Sent message".to_string(),
                    success: true,
                    shortMessage: "Snt Msg".to_string(),
                }),
            )),
            Err(_) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(APIErrors {
                    errors: vec![APIError {
                        code: 1,
                        message: "Internal Server Error".to_string(),
                        userFacingMessage: None,
                    }],
                }),
            )),
        }
    } else {
        Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(APIErrors {
                errors: vec![APIError {
                    code: 1,
                    message: "Internal Server Error".to_string(),
                    userFacingMessage: None,
                }],
            }),
        ))
    }
}
async fn can_message<T: Connection>(
    db: State<Surreal<T>>,
    cook: CookieJar,
    Path(_id): Path<u64>,
) -> Result<(StatusCode, Json<CanMessageResponse>), (StatusCode, Json<APIErrors>)> {
    Ok((
        StatusCode::OK,
        Json(CanMessageResponse { canMessage: true }),
    ))
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
