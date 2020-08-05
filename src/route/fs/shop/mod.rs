use warp::{
    Filter,
    reply::Reply,
    filters::BoxedFilter,
    path,
};
use crate::{
    state::State,
};

mod product;

pub fn filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    path("product").and(
        product::filter(state.clone())
    )
    .boxed()
}