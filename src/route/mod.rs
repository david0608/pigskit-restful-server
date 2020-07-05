use warp::{
    reply::{
        Reply,
        Response,
        with_status,
    },
    filters::BoxedFilter,
    reject::Rejection,
    Filter,
    http::StatusCode,
    path,
    options,
};
use crate::{
    state::State,
    error::Error,
};

mod api;
mod fs;
mod utils;

pub fn routes(state: State) -> BoxedFilter<(impl Reply,)> {
    let state = warp::any().map(move || state.clone()).boxed();

    path("api").and(
        api::filter(state.clone())
    )
    .or(
        path("fs").and(
            fs::filter(state.clone())
        )
    )
    .recover(async move |rejection: Rejection| -> Result<Response, Rejection> {
        if rejection.is_not_found() {
            info!("Rejected a request which the end-point was not found.");
            Ok(with_status("Not Found.", StatusCode::NOT_FOUND).into_response())
        } else {
            if let Some(error) = rejection.find::<Error>() {
                if error.is_inner() {
                    info!("Encountered a server internal error: {:?}", error);
                } else {
                    info!("Encountered an error: {:?}", error);
                }
                Ok(error.into_response())
            } else {
                info!("Encountered an unhandled error: {:?}", rejection);
                Err(rejection)
            }
        }
    })
    .boxed()
}

pub fn dev_routes(state: State) -> BoxedFilter<(impl Reply,)> {
    routes(state)
    .or(
        // filter for preflight requests.
        options().map(warp::reply)
    )
    .map(|reply| warp::reply::with_header(reply, "Access-Control-Allow-Headers", "Content-Type"))
    .map(|reply| warp::reply::with_header(reply, "Access-Control-Allow-Credentials", "true"))
    .map(|reply| warp::reply::with_header(reply, "Access-Control-Allow-Origin", "http://localhost:3000"))
    .map(|reply| warp::reply::with_header(reply, "Access-Control-Allow-Methods", "GET, POST, OPTIONS, PUT, PATCH, DELETE"))
    .boxed()
}