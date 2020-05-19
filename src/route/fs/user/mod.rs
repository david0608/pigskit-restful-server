use warp::{
    Filter,
    reply::Reply,
    filters::BoxedFilter,
    path,
};
use crate::{
    state::State,
};

mod avatar;

pub fn filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    path("avatar").and(
        avatar::filter(state.clone())
    )
    .boxed()
}