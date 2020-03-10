use warp::{
    Reply,
    http,
    redirect,
    reply::{Response, reply, with_header}
};

pub fn set_cookie(cookie: String) -> Response {
    with_header(
        reply(),
        "Set-Cookie",
        cookie,
    ).into_response()
}

pub fn redirect_to(uri: &'static str) -> Response {
    redirect(http::Uri::from_static(uri)).into_response()
}
