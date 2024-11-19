use axum::{http::StatusCode, Json};
use axum_extra::extract::CookieJar;
use base64::Engine;
use rand::Rng;
use robacking::internal::{Announcement, Message, Session, User};
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
    // (StatusCode::OK, Json(APIErrors { errors: vec![] }))
}
pub const AUTH_COOKIE_NAME: &str = ".ROBLOSECURITREE";
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
#[derive(Serialize, Deserialize)]
pub struct IdQuery {
    pub id: u64,
}
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct ProductInfo {
    pub Name: String,
    pub PriceInRobux: u32,
    pub Created: String,
    pub Updated: String,
    pub ContentRatingTypeId: u8,
    pub MinimumMembershipLevel: u8,
    pub IsPublicDomain: bool,
}
pub async fn announce<T: Connection>(db: &Surreal<T>, ann: Announcement) -> Result<(), APIErrors> {
    match db
        .query("USE NS announces DB management; CREATE announcement CONTENT $cont")
        .bind(("cont", ann))
        .await
    {
        Ok(_) => Ok(()),
        Err(_) => Err(make_error(0, "Internal server error", None)),
    }
}
pub async fn add_message_to_all<T: Connection>(
    db: &Surreal<T>,
    msg: Message,
    update_read: bool,
) -> Result<(), APIErrors> {
    match db
        .query(
            r"
            USE NS users DB userinfo;
            FOR $userid IN (SELECT userid FROM user) {
                USE NS spawns DB userinfo;
                UPDATE user:$userid MERGE {"
                .to_string()
                + "\"messages\""
                + ": [$message]};\n"
                + {
                    if update_read {
                        r"USE NS users DB userinfo;
                      UPDATE user:$userid SET unread.messages+=1"
                    } else {
                        ""
                    }
                }
                + "}",
        )
        .bind(("message", msg))
        .await
    {
        Ok(_) => Ok(()),
        Err(_) => Err(make_error(0, "Internal server error", None)),
    }
}
pub async fn add_message_to_userid<T: Connection>(
    db: &Surreal<T>,
    msg: Message,
    uid: u64,
    update_read: bool,
) -> Result<(), APIErrors> {
    if update_read {
        get_user_from_id(db, uid).await?;
        match db
            .query(
                "USE NS users DB userinfo; UPDATE user:".to_string()
                    + uid.to_string().as_str()
                    + " SET unread.messages+=1",
            )
            .await
        {
            Ok(_) => {}
            Err(_) => return Err(make_error(0, "Invalid userid", None)),
        }
    };
    match db
        .query(
            "USE NS spawns BD userinfo; UPDATE user:".to_string()
                + uid.to_string().as_str()
                + " MERGE {\"messages:\": [$new]}",
        )
        .bind(("new", msg))
        .await
    {
        Ok(_) => Ok(()),
        Err(_) => Err(make_error(0, "Invalid userid", None)),
    }
    // insert statement?
}
pub async fn create_cookie_for_userid<T: Connection>(
    db: &Surreal<T>,
    uid: u64,
) -> Result<Session, APIErrors> {
    // if get_cookie_object(db,cook).is_ok() {
    //     return
    // }
    let cooked = rand::thread_rng()
        .sample_iter(rand::distributions::Alphanumeric)
        .take(128)
        .map(char::from)
        .map(|x| x.to_ascii_uppercase())
        .collect::<String>();
    let new = Session {
        userid: uid,
        cookie: cooked,
        alive: true,
        active: false,
    };
    match db
        .query("USE NS sessions DB userinfo; INSERT INTO session $cook")
        .bind(("cook", new.clone()))
        .await
    {
        Ok(_) => Ok(new),
        Err(_) => Err(APIErrors {
            errors: vec![APIError {
                code: 0,
                message: "Internal server error".to_string(),
                userFacingMessage: None,
            }],
        }),
    }
}
pub async fn get_cookie_object<T: Connection>(
    db: &Surreal<T>,
    cook: &CookieJar,
) -> Result<Session, APIErrors> {
    let cooked = cook.get(AUTH_COOKIE_NAME);
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
            if seshs.as_ref().is_ok_and(|x| x.len() == 1) {
                Ok(seshs.unwrap().first().unwrap().clone())
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
    for c in cook.iter() {
        println!(
            "cooking {:?} = {:?}",
            c.name().to_string(),
            c.value().to_string()
        );
    }
    let cooked = cook.get(AUTH_COOKIE_NAME);
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
            if seshs.as_ref().is_ok_and(|x| !x.is_empty()) {
                get_user_from_id(db, seshs.unwrap().first().unwrap().userid).await
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
pub async fn get_user_from_username<T: Connection>(
    db: &Surreal<T>,
    name: String,
) -> Result<User, APIErrors> {
    // println!("get_user_from_id query: {}", quer);
    match db
        .query("USE NS users DB userinfo; SELECT * FROM user WHERE username=$name")
        .bind(("name", name))
        .await
    {
        Ok(mut useder) => {
            let user: Result<Option<User>, _> = useder.take(1);
            if user.as_ref().is_ok_and(|x| x.is_some()) {
                // let theuser = user.unwrap().unwrap();
                Ok(user.unwrap().unwrap())
            } else {
                if user.is_err() {
                    println!("get_user_from_username bad take {:?}", user.unwrap_err());
                }
                Err(make_error(0, "Invalid user name", None))
            }
        }
        Err(_) => Err(make_error(0, "Invalid user name", None)),
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
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct ApiPageResponse<T> {
    pub previousPageCursor: Option<String>,
    pub nextPageCursor: Option<String>,
    pub data: Vec<T>,
}
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct PagelessPagedResponse<T> {
    pub data: Vec<T>,
}
use serde_repr::{Deserialize_repr, Serialize_repr};
#[derive(Serialize_repr, Deserialize_repr, Debug, Clone)]
#[repr(u8)]
pub enum DBAssetType {
    Image = 1,
    TShirt = 2,
    Audio = 3,
    Mesh = 4,
    Lua = 5,
    Hat = 8,
    Place = 9,
    Model = 10,
    Shirt = 11,
    Pants = 12,
    Decal = 13,
    Head = 17,
    Face = 18,
    Gear = 19,
    Badge = 21,
    Animation = 24,
    Torso = 27,
    RightArm = 28,
    LeftArm = 29,
    LeftLeg = 30,
    RightLeg = 31,
    Package = 32,
    GamePass = 34,
    Plugin = 38,
    MeshPart = 40,
    HairAccessory = 41,
    FaceAccessory = 42,
    NeckAccessory = 43,
    ShoulderAccessory = 44,
    FrontAccessory = 45,
    BackAccessory = 46,
    WaistAccessory = 47,
    ClimbAnimation = 48,
    DeathAnimation = 49,
    FallAnimation = 50,
    IdleAnimation = 51,
    JumpAnimation = 52,
    RunAnimation = 53,
    SwimAnimation = 54,
    WalkAnimation = 55,
    PoseAnimation = 56,
    EarAccessory = 57,
    EyeAccessory = 58,
    EmoteAnimation = 61,
    Video = 62,
    TShirtAccessory = 64,
    ShirtAccessory = 65,
    PantsAccessory = 66,
    JacketAccessory = 67,
    SweaterAccessory = 68,
    ShortsAccessory = 69,
    LeftShoeAccessory = 70,
    RightShoeAccessory = 71,
    DressSkirtAccessory = 72,
    FontFamily = 73,
    EyebrowAccessory = 76,
    EyelashAccessory = 77,
    MoodAnimation = 78,
    DynamicHead = 79,
}
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DBAssetVisibility {
    Public,
    Private,
    Secret, // ooooo
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DBAsset {
    pub allowed_universes: Option<Vec<u64>>,
    pub cost: u32,
    pub description: String,
    pub format: String,
    pub hash: String,
    pub owner: u64,
    pub owner_is_group: bool,
    pub title: String,
    pub asset_type: DBAssetType,
    pub version: u32,
    pub visibility: DBAssetVisibility,
}
pub async fn get_asset_from_id<T: Connection>(
    db: &Surreal<T>,
    id: u64,
) -> Result<DBAsset, APIErrors> {
    let quer =
        "USE NS assets DB userinfo; SELECT * FROM asset:".to_string() + id.to_string().as_str();
    match db.query(quer).await {
        Ok(mut de_bass_et) => {
            let dbasset: Result<Option<DBAsset>, _> = de_bass_et.take(1);
            if dbasset.as_ref().is_ok_and(|x| x.is_some()) {
                Ok(dbasset.unwrap().unwrap())
            } else {
                if dbasset.is_err() {
                    println!("get_asset_from_id bad take {:?}", dbasset.unwrap_err());
                }
                Err(make_error(0, "Invalid asset id", None))
            }
        }
        Err(_) => Err(make_error(0, "Invalid asset id", None)),
    }
}
pub fn unixtime() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
use sha1::{Digest, Sha1};
pub fn new_auth_ticket<'a>(
    time: u64,
    userid: u64,
    username: &'a str,
    jobid: u64,
    appearance: &'a str,
) -> String {
    let mut sha = Sha1::new();
    Digest::update(
        &mut sha,
        [userid.to_string(),
            username.to_string(),
            appearance.to_string(),
            jobid.to_string(),
            time.to_string()]
        .join("\n"),
    );
    let sig1 =
        base64::engine::general_purpose::STANDARD_NO_PAD.encode(Digest::finalize_reset(&mut sha));
    Digest::update(
        &mut sha,
        [userid.to_string(), jobid.to_string(), time.to_string()].join("\n"),
    );
    let sig2 =
        base64::engine::general_purpose::STANDARD_NO_PAD.encode(Digest::finalize_reset(&mut sha));
    time.to_string() + ";" + sig1.as_str() + ";" + sig2.as_str()
}
