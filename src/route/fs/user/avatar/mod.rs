use warp::{
    Filter,
    reply::Reply,
    filters::BoxedFilter,
    get,
    post,
    delete,
    path,
};
use uuid::Uuid;
use crate::{
    route::utils::{
        filter::{
            form,
            cookie,
        },
        handler::fs,
    },
    state::State,
    STORAGE_DIR,
};

fn update_filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    post()
    .and(
        cookie::to_user_id("USSID", state.clone())
        .map(|user_id: Uuid| {
            format!("{}/user/{}", *STORAGE_DIR, user_id)
        })
    )
    .and(
        form::image("avatar", 512000)
    )
    .and_then(fs::store())
    .boxed()
}

fn read_filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    get()
    .and(
        cookie::to_user_id("USSID", state.clone())
        .map(|user_id: Uuid| {
            (
                format!("{}/user/{}/avatar.jpg", *STORAGE_DIR, user_id),
                format!("{}/user/default/avatar.jpg", *STORAGE_DIR),
            )
        })
        .untuple_one()
    )
    .and_then(fs::read_with_default)
    .boxed()
}

fn delete_filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    delete()
    .and(
        cookie::to_user_id("USSID", state.clone())
        .map(|user_id: Uuid| {
            format!("{}/user/{}/avatar.jpg", *STORAGE_DIR, user_id)
        })
    )
    .and_then(fs::delete)
    .boxed()
}

pub fn filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    path::end().and(
        update_filter(state.clone())
        .or(read_filter(state.clone()))
        .or(delete_filter(state.clone()))
    )
    .boxed()
}