
mod password;

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
    credentials: password::ArgonPasswordsInFile
}

/// User information
struct UserInfo {
    id: String,
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
    let state: Arc<AppState> = Arc::new(AppState{
        data_server: "http://127.0.0.1:9000".to_string(),
        http_pool: reqwest::Client::new(),
        key: Key::generate(),
        credentials: users,
    });

    let app = Router::new()
        .route("/", routing::get(page_login))
        .route("/main", routing::get(page_main))
        .route("/api/health", routing::get(api_health))
        .route("/api/version", routing::get(|| async { "0.1.0" }))
        .route("/api/post/login", routing::post(api_post_login))
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

        //TODO: actual user info?
        Ok(UserInfo{
            id: "user1".to_string(),
        })
    }
}

/*
/// Fake struct used to hold cookie-related headers in case I'll need to access cookies
/// Used since it's impossible to create axum::extract::FromRequestParts with Arc<> on one side and
/// axum::http::HeaderMap on the other (at least one of those needs to be defined locally
struct CookieHeaderMap {
    headermap: axum::http::HeaderMap,
}
impl From<CookieHeaderMap> for axum::http::HeaderMap {
    fn from(a: CookieHeaderMap) -> Self {
        a.headermap
    }
}
impl From<axum::http::HeaderMap> for CookieHeaderMap {
    fn from(a: axum::http::HeaderMap) -> Self {
        CookieHeaderMap {
            headermap: a,
        }
    }
}
#[axum::async_trait]
impl axum::extract::FromRequestParts<Arc<AppState>> for CookieHeaderMap
{
    type Rejection = (StatusCode, &'static str);
    async fn from_request_parts(parts: &mut Parts, _: &Arc<AppState>) -> Result<Self, Self::Rejection> {
        let mut chm = axum::http::HeaderMap::new();
        if let Some(cv) = parts.headers.get(axum::http::header::COOKIE) {
            chm.insert(axum::http::header::COOKIE, cv.clone());
        }
        Ok(chm.into())
    }
}
*/

/// Login Form - username & password
#[derive(serde::Deserialize)]
struct FormLogin {
    username: String,
    password: String,
}


/// API endpoint for creating a session provided proper authentication data is sent in the request
/// NOTE: Form<> consumes request body and therefore must be the LAST input variable
async fn api_post_login(State(state): State<Arc<AppState>>, Form(form): Form<FormLogin>) -> Response {
    // TODO: implement a user/password backend
    if !state.credentials.check_password(&form.username, &form.password) {
        return (StatusCode::UNAUTHORIZED, "").into_response();
    }
    let session = uuid::Uuid::new_v4();
    let session_data = format!("{}/{:X}", "user1", session.as_simple());
    let query = format!("{}/api/post/one/{}", state.data_server, session_data);
    let Ok(resp) = state.http_pool.post(&query)
        //TODO: store some actual info?
        .body(b"HELLO".as_ref())
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
