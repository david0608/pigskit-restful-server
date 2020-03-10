use std::convert::Infallible;
use warp::Filter;
use uuid::Uuid;
use crate::state::State;
use crate::error::Error;

mod access;
mod response;

pub use access::*;

type BoxedFilter<T> = warp::filters::BoxedFilter<(T,)>;
type FilterResult<T> = Result<T, Infallible>;

pub fn map_state(state: State) -> BoxedFilter<State> {
    warp::any().map(move || state.clone()).boxed()
}

fn parse_uuid_optional(param: Option<String>) -> Result<Uuid, Error> {
    if let Some(id) = param {
        Uuid::parse_str(id.as_str()).map_err(|err| err.into())
    } else {
        Err(Error::Other("Optional UUID parameter not specified."))
    }
}