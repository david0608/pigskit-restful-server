use warp::{
    Filter,
    reply::Reply,
    filters::BoxedFilter,
    path,
};
use crate::state::State;

mod register;
mod session;
mod profile;

pub fn filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    path("register").and(
        register::filter(state.clone())
    )
    .or(
        path("session").and(
            session::filter(state.clone())
        )
    )
    .or(
        path("profile").and(
            profile::filter(state.clone())
        )
    )
    .boxed()
}