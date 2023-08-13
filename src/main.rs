
mod password;
mod measurements;
mod database;

use std::sync::Arc;

use axum::{
    Form,
    routing,
    Router,
    response::{Response, IntoResponse},
    extract::State,
    http::{request::Parts, status::StatusCode},
};

use axum_extra::extract::cookie::{SignedCookieJar, Cookie, Key};

use password::VerifyPassword;

/// Internal state shared between requests
struct AppState {
    /// Address of the login service server
    data_server: String,
    /// Reqwest pool for making requests
    http_pool: reqwest::Client,
    /// Key used to sign cookies
    key: Key,
    /// User password info
    credentials: password::ArgonPasswordsInFile,
    /// Data(base?)
    data: database::InMemoryStorage,
}

/// User information
struct UserInfo {
    id: u32,
}

/// Name of the signed cookie used for storing the session info
const LOGIN_COOKIE: &str = "phdsa";

/// Main function: defines all the routes
#[tokio::main]
async fn main() {
    // TODO: load data from environment variables
    // Default settings for development
    let Ok(users) = password::ArgonPasswordsInFile::try_from("./passwords.txt") else {
        println!("ERROR: missing passwords file.");
        return;
    };
    let Ok(db) = database::InMemoryStorage::try_from("./database.bin") else {
        println!("ERROR: unable to generate database.");
        return;
    };
    let state: Arc<AppState> = Arc::new(AppState{
        data_server: "http://127.0.0.1:9000".to_string(),
        http_pool: reqwest::Client::new(),
        key: Key::generate(),
        credentials: users,
        data: db,
    });

    let app = Router::new()
        .route("/", routing::get(page_login))
        .route("/main", routing::get(page_main))
        .route("/api/health", routing::get(api_health))
        .route("/api/version", routing::get(|| async { "0.1.0" }))
        .route("/api/post/login", routing::post(api_post_login))
        .route("/api/post/data", routing::post(api_post_data))
        .route("/api/get/data/all", routing::get(api_get_data))
        .with_state(state);

    axum::Server::bind(&"0.0.0.0:8999".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}


/// extractor used to check if a request is made by a logged in user
#[axum::async_trait]
impl axum::extract::FromRequestParts<Arc<AppState>> for UserInfo
{
    type Rejection = (StatusCode, &'static str);
    async fn from_request_parts(parts: &mut Parts, state: &Arc<AppState>) -> Result<Self, Self::Rejection> {
        let jar = SignedCookieJar::from_headers(&parts.headers, state.key.clone());
        let Some(cookie) = jar.get(LOGIN_COOKIE) else {
            return Err((StatusCode::UNAUTHORIZED, ""));
        };
        let query = format!("{}/api/get/one/{}", state.data_server, cookie.value());
        let Ok(resp) = state.http_pool.get(&query).send().await else {
            return Err((StatusCode::UNAUTHORIZED, ""));
        };
        if resp.status() != StatusCode::OK {
            return Err((StatusCode::UNAUTHORIZED, ""));
        }
        let Ok(body) = resp.bytes().await else {
            return Err((StatusCode::INTERNAL_SERVER_ERROR, ""));
        };
        let Ok((_, uid)) = nom::number::complete::le_u32::<&[u8], ()>(body.as_ref()) else {
            return Err((StatusCode::INTERNAL_SERVER_ERROR, ""));
        };

        Ok(UserInfo{
            id: uid,
        })
    }
}


/// Login Form - username & password
#[derive(serde::Deserialize)]
struct FormLogin {
    username: String,
    password: String,
}


/// API endpoint for creating a session provided proper authentication data is sent in the request
/// NOTE: Form<> consumes request body and therefore must be the LAST input variable
async fn api_post_login(State(state): State<Arc<AppState>>, Form(form): Form<FormLogin>) -> Response {
    let Some(uid) = state.credentials.check_password(&form.username, &form.password) else {
        return (StatusCode::UNAUTHORIZED, "").into_response();
    };
    let uid_str = format!("{}", uid);
    let uid = Vec::from(uid.to_le_bytes());
    let session = uuid::Uuid::new_v4();
    let session_data = format!("{}/{:X}", uid_str, session.as_simple());
    let query = format!("{}/api/post/one/{}", state.data_server, session_data);
    let Ok(resp) = state.http_pool.post(&query)
        .body(uid)
        .send().await else {
            return (StatusCode::SERVICE_UNAVAILABLE, "").into_response();
    };
    if resp.status() != StatusCode::OK {
        println!("INTERNAL RESPONSE: [{}] [{}]", query, resp.status());
        return (StatusCode::INTERNAL_SERVER_ERROR, "").into_response();
    }
    let jar = SignedCookieJar::new(state.key.clone())
        .add(Cookie::build(LOGIN_COOKIE, session_data)
             .path("/")
             .http_only(true)
             .same_site(axum_extra::extract::cookie::SameSite::Strict)
             .max_age(time::Duration::days(30))
             .finish()
         );

    (StatusCode::OK, jar, "/main").into_response()
}

async fn api_post_data(State(state): State<Arc<AppState>>, user: UserInfo, axum::extract::Json(body): axum::extract::Json<database::PerUserData>) -> Response {
    state.data.store_data(user.id, body).await;
    (StatusCode::OK, "").into_response()
}

async fn api_get_data(State(state): State<Arc<AppState>>, user: UserInfo) -> axum::Json<database::PerUserData>{
    let data = state.data.retrieve_data(user.id).await;
    axum::Json(data)
}

/// Server health status:
/// 200 OK: everything is OK
/// 503 _: there is a problem with one or more backends
async fn api_health(State(state): State<Arc<AppState>>) -> Response {
    let query = format!("{}/api/health", state.data_server);

    let Ok(resp) = reqwest::get(&query).await else {
        return (StatusCode::SERVICE_UNAVAILABLE, "SESSION BACKEND: MISSING").into_response();
    };
    if resp.status() != StatusCode::OK {
        return (StatusCode::SERVICE_UNAVAILABLE, "SESSION BACKEND: UNAVAILABLE").into_response();
    }
    let Ok(resp) = resp.bytes().await else {
        return (StatusCode::SERVICE_UNAVAILABLE, "SESSION BACKEND: COMM ERROR").into_response();
    };
    if resp.as_ref() != b"OK" {
        return (StatusCode::SERVICE_UNAVAILABLE, "SESSION BACKEND: UNAVAILABLE").into_response();
    }

    (StatusCode::OK, "OK").into_response()
}

/// HTML page to display as login page
async fn page_login() -> Response {
    axum::response::Html::from(include_str!("../resources/login.html")).into_response()
}

/// HTML page to display as main page
async fn page_main(_: UserInfo) -> Response {
    axum::response::Html::from(include_str!("../resources/main.html")).into_response()
}
