use std::convert::Infallible;
use warp::{
    reply::Reply,
    Filter,
    path, options,
    http,
};
use uuid::Uuid;
use crate::state::State;
use crate::error::Error;

mod user;
mod session;
mod shop;
mod member;
mod authority;
mod product;
mod customize;
mod response;

type BoxedFilter<T> = warp::filters::BoxedFilter<(T,)>;
type FilterResult<T> = Result<T, Infallible>;

pub fn routes(state: State) -> BoxedFilter<impl Reply> {
    let state = map_state(state);

    path("api").and(
        path("user").and(
            user::filter(state.clone())
            .or(
                path("session").and(
                    session::filter(state.clone())
                )
            )
        )
        .or(
            path("shop").and(
                shop::filter(state.clone())
                .or(
                    path("member").and(
                        member::filter(state.clone())
                        .or(
                            path("authority").and(
                                authority::filter(state.clone())
                            )
                        )
                    )
                )
                .or(
                    path("product").and(
                        product::filter(state.clone())
                        .or(
                            path("customize").and(
                                customize::filter(state.clone())
                            )
                        )
                    )
                )
            )
        )
    )
    .boxed()
}

fn map_state(state: State) -> BoxedFilter<State> {
    warp::any().map(move || state.clone()).boxed()
}

fn preflight_filter() -> BoxedFilter<impl Reply> {
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

fn parse_uuid_optional(param: Option<String>) -> Result<Uuid, Error> {
    if let Some(id) = param {
        parse_uuid(id)
    } else {
        Err(Error::Other("Optional UUID parameter not specified."))
    }
}

fn parse_uuid(id: String) -> Result<Uuid, Error> {
    Uuid::parse_str(id.as_str()).map_err(|err| err.into())
}