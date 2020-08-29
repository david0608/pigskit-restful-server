use warp::{
    Filter,
    reply::Reply,
    filters::BoxedFilter,
    path,
};
use crate::{
    state::State,
};

mod user;
mod shop;
mod cart;

pub fn filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    path("user").and(
        user::filter(state.clone())
    )
    .or(
        path("shop").and(
            shop::filter(state.clone())
        )
    )
    .or(
        path("cart").and(
            cart::filter(state.clone())
        )
    )
    .boxed()
}