use warp::{
    reply::{
        reply,
        Reply,
        with_header,
        Response,
    },
    http,
    redirect,
};
use chrono::{
    Utc,
    Duration,
};

pub fn set_cookie(name: &str, value: &str, duration: i64) -> Response {
    let expire = Utc::now() + Duration::days(duration);
    with_header(
        reply(),
        "Set-Cookie",
        format!("{}={}; Path=/; Expires={}; HttpOnly", name, value, expire.format("%a, %d %b %Y %T GMT")),
    ).into_response()
}

pub fn redirect_to(uri: &'static str) -> Response {
    redirect(http::Uri::from_static(uri)).into_response()
}