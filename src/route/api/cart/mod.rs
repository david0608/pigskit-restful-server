use warp::{
    Filter,
    reply::Reply,
    filters::BoxedFilter,
    path,
};
use crate::state::State;

mod item;
mod session;
mod order;

pub fn filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    path("item").and(
        item::filter(state.clone())
    )
    .or(
        path("session").and(
            session::filter(state.clone())
        )
    )
    .or(
        path("order").and(
            order::filter(state.clone())
        )
    )
    .boxed()
}