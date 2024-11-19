use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::CookieJar;
use bloxxing_match::{api_404, get_authenticated_user, new_auth_ticket, unixtime};
use http::HeaderName;
use http::Method;
use http::Request;
use robacking::Roblox::auth_v1::SkinnyUserResponse;
use serde::{Deserialize, Serialize};
use surrealdb::{Connection, Surreal};
use tower_http::cors::CorsLayer;
pub(crate) fn new<T: Connection>() -> Router<Surreal<T>> {
    Router::new()
        .route("/game/validate-machine", post(validate_machine)) // gone?
        .route("/game/PlaceLauncher.ashx", post(place_launcher)) // gone
        .route("/game/Join.ashx", get(get_join_script)) // was moved to https://gamejoin.roblox.com/v1/join-game* (different api)
        .route("/game/Negotiate.ashx", post(negotiate)) // replaced with https://auth.roblox.com/v1/authentication-ticket/redeem (same api)
        .route("/game/GetCurrentUser.ashx", get(get_current_user)) // was moved to https://auth.roblox.com/v1/authenticated (some url like that)
        .route("/my/settings/json", get(get_settings_json))
        // https://api.roblox.com/v1.1/avatar-fetch/?placeId=1818&userId=228176120
        // www.roblox.com/Login/Negotiate.ashx
        // game/RequestAuth.ashx
        // game/logout.ashx
        // edit, join, visit, gameserver .ashx
        // (case insensitive? ^)
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
                    "rbxauthorizationticket".parse::<HeaderName>().unwrap(),
                    "x-csrf-token".parse::<HeaderName>().unwrap(),
                ])
                .allow_credentials(true),
        )
        .fallback(api_404)
}

async fn get_settings_json() -> String {
    "{
      \"ChangeUsernameEnabled\": true,
      \"IsAdmin\": true,
      \"UserId\": 228176120,
      \"Name\": \"bainchild\",
      \"DisplayName\": \"bainchild\",
      \"IsEmailOnFile\": false,
      \"IsEmailVerified\": true,
      \"IsPhoneFeatureEnabled\": false,
      \"RobuxRemainingForUsernameChange\": 0,
      \"PreviousUserNames\": \"\",
      \"UseSuperSafePrivacyMode\": false,
      \"IsAppChatSettingEnabled\": true,
      \"IsGameChatSettingEnabled\": true,
      \"IsParentalSpendControlsEnabled\": false,
      \"IsSetPasswordNotificationEnabled\": false,
      \"ChangePasswordRequiresTwoStepVerification\": false,
      \"ChangeEmailRequiresTwoStepVerification\": false,
      \"UserEmail\": \"bainchild@letterbomb.ftp.sh\",
      \"UserEmailMasked\": true,
      \"UserEmailVerified\": true,
      \"CanHideInventory\": true,
      \"CanTrade\": false,
      \"MissingParentEmail\": false,
      \"IsUpdateEmailSectionShown\": true,
      \"IsUnder13UpdateEmailMessageSectionShown\": false,
      \"IsUserConnectedToFacebook\": false,
      \"IsTwoStepToggleEnabled\": false,
      \"AgeBracket\": 0,
      \"UserAbove13\": true,
      \"ClientIpAddress\": \"0.0.0.0\",
      \"AccountAgeInDays\": 2677,
      \"IsPremium\": false,
      \"IsBcRenewalMembership\": false,
      \"PremiumFeatureId\": null,
      \"HasCurrencyOperationError\": false,
      \"CurrencyOperationErrorMessage\": null,
      \"Tab\": null,
      \"ChangePassword\": false,
      \"IsAccountPinEnabled\": false,
      \"IsAccountRestrictionsFeatureEnabled\": false,
      \"IsAccountSettingsSocialNetworksV2Enabled\": false,
      \"IsUiBootstrapModalV2Enabled\": true,
      \"IsDateTimeI18nPickerEnabled\": true,
      \"InApp\": false,
      \"MyAccountSecurityModel\": {
        \"IsEmailSet\": true,
        \"IsEmailVerified\": true,
        \"IsTwoStepEnabled\": false,
        \"ShowSignOutFromAllSessions\": true,
      },
      \"ApiProxyDomain\": \"https://api.roblox.com\",
      \"AccountSettingsApiDomain\": \"https://accountsettings.roblox.com\",
      \"AuthDomain\": \"https://auth.roblox.com\",
      \"IsDisconnectFacebookEnabled\": true,
      \"IsDisconnectXboxEnabled\": true,
      \"NotificationSettingsDomain\": \"https://notifications.roblox.com\",
      \"AllowedNotificationSourceTypes\": [
        \"Test\",
        \"FriendRequestReceived\",
        \"FriendRequestAccepted\",
        \"PartyInviteReceived\",
        \"PartyMemberJoined\",
        \"ChatNewMessage\",
        \"PrivateMessageReceived\",
        \"UserAddedToPrivateServerWhiteList\",
        \"ConversationUniverseChanged\",
        \"TeamCreateInvite\",
        \"GameUpdate\",
        \"DeveloperMetricsAvailable\",
        \"GroupJoinRequestAccepted\",
        \"Sendr\",
        \"ExperienceInvitation\"
      ],
      \"AllowedReceiverDestinationTypes\": [
        \"NotificationStream\"
      ],
      \"BlacklistedNotificationSourceTypesForMobilePush\": [],
      \"MinimumChromeVersionForPushNotifications\": 50,
      \"PushNotificationsEnabledOnFirefox\": false,
      \"LocaleApiDomain\": \"https://locale.roblox.com\",
      \"HasValidPasswordSet\": true,
      \"IsFastTrackAccessible\": true,
      \"HasFreeNameChange\": true,
      \"IsAgeDownEnabled\": true,
      \"IsDisplayNamesEnabled\": true,
      \"IsBirthdateLocked\": false
    }"
    .to_string()
}
async fn get_current_user<T: Connection>(
    db: State<Surreal<T>>,
    cook: CookieJar,
) -> Result<(StatusCode, String), StatusCode> {
    let user = get_authenticated_user(&db, &cook).await;
    if user.is_ok() {
        let usear = user.unwrap();
        Ok((StatusCode::OK, usear.userid.to_string()))
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ValidateMachineResponse {
    success: bool,
    message: String,
}

async fn validate_machine() -> (StatusCode, Json<ValidateMachineResponse>) {
    (
        StatusCode::OK,
        Json(ValidateMachineResponse {
            success: true,
            message: "".to_string(),
        }),
    )
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
enum MembershipType {
    None,
    BuildersClub,
    TurboBuildersClub,
    OutrageousBuildersClub,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
enum CreatorType {
    User,
    Group,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
enum ChatStyle {
    ChatAndBubble,
    BubbleChat,
}
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct JSONJoinScript {
    // I think this is in the order that it handles them
    pub GameId: String,
    pub UserId: u64,
    pub BrowserTrackerId: String,
    pub ClientTicket: String,
    pub PlaceId: u64,
    pub UniverseId: u64,
    pub IsRobloxPlace: bool,
    pub MachineAddress: String,
    pub CreatorId: u64,
    pub CreatorTypeEnum: CreatorType,
    pub ChatStyle: ChatStyle,
    pub SessionId: String,
    pub ServerPort: u16,
    pub ClientPort: u16,
    pub SuperSafeChat: bool,
    pub IsUnknownOrUnder13: bool,
    pub MembershipType: MembershipType,
    pub AccountAge: u64,
    pub UserName: String,
    pub CharacterAppearance: String,
    pub FollowUserId: Option<u64>,
    pub ScreenShotInfo: Option<String>,
    pub VideoInfo: Option<String>,
    pub SeleniumTestMode: bool,
    pub CookieStoreEnabled: bool,
    pub VendorId: u64,
    pub DataCenterId: u64,
    pub BaseUrl: String,
    pub PingUrl: String,
    pub GenerateTeleportJoin: bool,
}
#[warn(non_snake_case)]
async fn get_join_script<T: std::fmt::Debug>(req: Request<T>) -> String {
    println!("get_join_script: {:?}", req);
    let joinscript = JSONJoinScript {
        MachineAddress: "127.0.0.1".to_string(),
        GameId: "gameid_ICUP".to_string(),
        UserId: 3,
        BrowserTrackerId: "".to_string(),
        ClientTicket: req
            .headers()
            .get("RBXAuthenticationTicket")
            .and_then(|x| Some(x.to_str().unwrap().to_string()))
            .unwrap_or_else(|| {
                new_auth_ticket(unixtime(), 1, "ROBLOX", 1, "https://avatar.roblox.com")
            }),
        PlaceId: 1818,
        UniverseId: 1,
        IsRobloxPlace: true,
        CreatorId: 3,
        CreatorTypeEnum: CreatorType::User,
        ChatStyle: ChatStyle::ChatAndBubble,
        SessionId: "sessionid".to_string(),
        ServerPort: 54321,
        ClientPort: 54321,
        SuperSafeChat: false,
        IsUnknownOrUnder13: false,
        MembershipType: MembershipType::None,
        AccountAge: 2400,
        UserName: "Jane Doe".to_string(),
        CharacterAppearance: "3".to_string(),
        ScreenShotInfo: None,
        VideoInfo: None,
        SeleniumTestMode: false,
        CookieStoreEnabled: false,
        VendorId: 12345,
        DataCenterId: 0,
        BaseUrl: "https://assetgame.roblox.com/".to_string(),
        PingUrl: "https://users.roblox.com/".to_string(),
        GenerateTeleportJoin: false,
        FollowUserId: None,
    };
    "--rbxsig\n".to_string() + serde_json::to_string(&joinscript).unwrap().as_str()
}
async fn negotiate<T: std::fmt::Debug>(req: Request<T>) -> String {
    println!("negotiate : {:?}", req);
    "{}".to_string()
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
enum PlaceLauncherRequestType {
    RequestPrivateGame,
    RequestFollowUser,
    RequestGame,
    CheckGameJobStatus,
}
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
struct PlaceLauncherAshx {
    pub request: PlaceLauncherRequestType,
    pub placeId: Option<u64>,
    pub userId: Option<u64>,
    pub jobId: Option<u64>,
    pub accessCode: Option<u64>,
}
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
struct PlacelauncherAshxResponse {
    pub jobId: Option<String>,
    pub placeId: Option<u64>,
    pub status: u64,
    pub joinScriptUrl: Option<String>,
    pub authenticationUrl: Option<String>,
    pub authenticationTicket: Option<String>,
    pub message: Option<String>,
}
// {"jobId":"mobiletest1","status":2,"joinScriptUrl":"http://buttass69.local/game/YOURSCRIPT.php.whatever","authenticationUrl":"https://www.roblox.com/Login/Negotiate.ashx","authenticationTicket": "hi","message":"this text is useless except in like 2013L"}
// Place join status results (?)
// Waiting = 0,
// Loading = 1,
// Joining = 2,
// Disabled = 3,
// Error = 4,
// GameEnded = 5,
// GameFull = 6
// UserLeft = 10
// Restricted = 11
async fn place_launcher(
    Query(req_west): Query<PlaceLauncherAshx>,
) -> (StatusCode, Json<PlacelauncherAshxResponse>) {
    println!("placelauncher query: {:?}", req_west);
    // if req_west.request == PlaceLauncherRequestType::CheckGameJobStatus {
    (
        StatusCode::OK,
        Json(PlacelauncherAshxResponse {
            jobId: Some("1".to_string()),
            placeId: Some(1818),
            status: 2,
            joinScriptUrl: Some("https://assetgame.roblox.com/game/Join.ashx".to_string()),
            authenticationUrl: Some("https://assetgame.roblox.com/game/Negotiate.ashx".to_string()),
            authenticationTicket: Some(new_auth_ticket(
                unixtime(),
                1,
                "ROBLOX",
                1,
                "https://avatar.roblox.com/",
            )),
            message: Some("thumbs up".to_string()),
        }),
    )
    // } else {
    //     (
    //         StatusCode::OK,
    //         Json(PlacelauncherAshxResponse {
    //             jobId: None,
    //             status: 1,
    //             joinScriptUrl: None,
    //             authenticationUrl: None,
    //             authenticationTicket: None,
    //             message: None,
    //         }),
    //     )
    // }
}
