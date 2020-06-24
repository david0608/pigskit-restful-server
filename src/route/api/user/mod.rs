use warp::{
    Filter,
    reply::Reply,
    filters::BoxedFilter,
    path,
};
use crate::state::State;

mod register;
mod session;

pub fn filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    path("register").and(
        register::filter(state.clone())
    )
    .or(
        path("session").and(
            session::filter(state.clone())
        )
    )
    .boxed()
}