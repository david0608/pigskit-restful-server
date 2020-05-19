use warp::{
    Filter,
    reply::Reply,
    filters::BoxedFilter,
    http,
    options,
};

pub mod form;
pub mod cookie;

pub fn preflight_filter() -> BoxedFilter<(impl Reply,)> {
    options()
    .map(|| {
        Ok(
            http::Response::builder()
            .status(http::StatusCode::OK)
            .header("Access-Control-Allow-Headers", "Content-Type")
            .header("Access-Control-Allow-Origin", "http://localhost")
            .body("")
        )
    })
    .boxed()
}