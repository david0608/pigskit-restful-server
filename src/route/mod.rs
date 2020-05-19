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
                info!("Encountered a process error: {}", error);
                Ok(error.into_response())
            } else {
                info!("Encountered an error: {:?}", rejection);
                Err(rejection)
            }
        }
    })
    .boxed()
}