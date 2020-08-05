use warp::{
    Filter,
    reply::Reply,
    filters::BoxedFilter,
    path,
};
use crate::{
    state::State,
};

mod image;

pub fn filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    path("image").and(
        image::filter(state.clone())
    )
    .boxed()
}