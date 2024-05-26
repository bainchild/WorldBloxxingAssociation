use axum::response::IntoResponse;
use axum::{http::StatusCode, Json};
use axum_extra::extract::CookieJar;
use robacking::internal::{Session, User};
use robacking::Roblox::auth_v1::LogoutFromAllSessionsAndReauthenticateRequest;
use robacking::Roblox::Web::WebAPI::{APIError, APIErrors};
use surrealdb::{Connection, Surreal};
pub async fn api_404() -> (StatusCode, Json<APIErrors>) {
    (
        StatusCode::NOT_FOUND,
        Json(APIErrors {
            errors: vec![APIError {
                code: 0,
                message: "NotFound".to_string(),
                userFacingMessage: None,
            }],
        }),
    )
}
pub fn make_error(code: u64, msg: &str, user: Option<&str>) -> APIErrors {
    APIErrors {
        errors: vec![APIError {
            code,
            message: msg.to_string(),
            userFacingMessage: {
                if user.is_some() {
                    Some(user.unwrap().to_string())
                } else {
                    None
                }
            },
        }],
    }
}
pub async fn get_cookie_object<T: Connection>(
    db: &Surreal<T>,
    cook: &CookieJar,
) -> Result<Session, APIErrors> {
    let cooked = cook.get(".ROBLOSECURITY");
    if cooked.is_none() {
        return Err(make_error(
            0,
            "Authorization denied for this request.",
            None,
        ));
    }
    let cookiestr = cooked.unwrap().value().to_string();
    match db
        .query(
            "USE NS sessions DB userinfo; SELECT * FROM session WHERE cookie=$cook AND alive=true",
        )
        .bind(("cook", cookiestr))
        .await
    {
        Ok(mut id) => {
            let seshs: Result<Vec<Session>, _> = id.take(1);
            if seshs.as_ref().is_ok_and(|x| x.len() > 0) {
                Ok(seshs.unwrap().get(0).unwrap().clone())
            } else {
                Err(make_error(
                    0,
                    "Authorization denied for this request.",
                    None,
                ))
            }
        }
        Err(_) => {
            // println!("query failed");
            Err(make_error(
                1,
                "Internal server error",
                Some("Something went wrong."),
            ))
        }
    }
}
pub async fn get_authenticated_user<T: Connection>(
    db: &Surreal<T>,
    cook: &CookieJar,
) -> Result<User, APIErrors> {
    // for c in cook.iter() {
    //     println!(
    //         "cooking {:?} = {:?}",
    //         c.name().to_string(),
    //         c.value().to_string()
    //     );
    // }
    let cooked = cook.get(".ROBLOSECURITY");
    if cooked.is_none() {
        // println!("cookie non existant");
        return Err(make_error(
            0,
            "Authorization denied for this request.",
            None,
        ));
    }
    let cookiestr = cooked.unwrap().value().to_string();
    // println!("ze cook es {:?}", cookiestr);
    match db
        .query(
            "USE NS sessions DB userinfo; SELECT * FROM session WHERE cookie=$cook AND alive=true",
        )
        .bind(("cook", cookiestr))
        .await
    {
        Ok(mut id) => {
            let seshs: Result<Vec<Session>, _> = id.take(1);
            println!("user sesh: {:?}", seshs);
            if seshs.as_ref().is_ok_and(|x| x.len() > 0) {
                get_user_from_id(db, seshs.unwrap().get(0).unwrap().userid).await
            } else {
                // println!("is NOT okay, or is empty");
                Err(make_error(
                    0,
                    "Authorization denied for this request.",
                    None,
                ))
            }
        }
        Err(_) => {
            // println!("query failed");
            Err(make_error(
                1,
                "Internal server error",
                Some("Something went wrong."),
            ))
        }
    }
}
pub async fn get_user_from_id<T: Connection>(db: &Surreal<T>, id: u64) -> Result<User, APIErrors> {
    let quer =
        "USE NS users DB userinfo; SELECT * FROM user:".to_string() + id.to_string().as_str();
    // println!("get_user_from_id query: {}", quer);
    match db.query(quer).await {
        Ok(mut useder) => {
            let user: Result<Option<User>, _> = useder.take(1);
            if user.as_ref().is_ok_and(|x| x.is_some()) {
                // let theuser = user.unwrap().unwrap();
                Ok(user.unwrap().unwrap())
            } else {
                if user.is_err() {
                    println!("get_user_from_id bad take {:?}", user.unwrap_err());
                }
                Err(make_error(0, "Invalid user id", None))
            }
        }
        Err(_) => Err(make_error(0, "Invalid user id", None)),
    }
}
