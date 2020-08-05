use warp::{
    Filter,
    reply::Reply,
    filters::BoxedFilter,
    reject,
    get,
    post,
    delete,
    path,
    multipart::{
        form,
        FormData,
    }
};
use futures::{
    TryFutureExt,
    TryStreamExt,
};
use bytes::BufMut;
use uuid::Uuid;
use crate::{
    route::utils::{
        filter::cookie,
        handler::{
            HandlerResult,
            fs
        },
    },
    state::State,
    error::Error,
    STORAGE_DIR,
};

fn update_filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    post()
    .and(cookie::to_user_id("USSID", state.clone()))
    .and(
        form_filter!(
            image [ Vec<u8> ]
        )
    )
    .map(|user_id: Uuid, image_data: Vec<u8>| {
        (
            format!("{}/user/{}", *STORAGE_DIR, user_id),
            "avatar.jpg".to_string(),
            image_data,
        )
    })
    .untuple_one()
    .and_then(fs::store_handler())
    .boxed()
}

fn read_filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    get()
    .and(
        cookie::to_user_id("USSID", state.clone())
        .map(|user_id: Uuid| {
            (
                format!("{}/user/{}/avatar.jpg", *STORAGE_DIR, user_id),
                format!("{}/default/user/avatar.jpg", *STORAGE_DIR),
            )
        })
        .untuple_one()
    )
    .and_then(fs::read_with_default_handler)
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
    .and_then(fs::delete_handler)
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