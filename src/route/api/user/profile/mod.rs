use warp::{
    Filter,
    reply::{
        Reply,
    },
    reject,
    filters::BoxedFilter,
    patch,
    path,
    multipart::{
        form,
        FormData,
    },
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
            fs,
            HandlerResult,
        },
    },
    state::State,
    error::Error,
    STORAGE_DIR,
};

mod avatar;

fn patch_filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    patch()
    .and(cookie::to_user_id("USSID", state.clone()))
    .and(
        form_filter!(
            nickname [ Option [ String ] ]
            avatar [ Option [ Vec<u8> ] ]
            delete_avatar [ Option [ bool ] ]
        )
    )
    .and(state)
    .and_then(async move |user_id: Uuid, nickname: Option<String>, avatar: Option<Vec<u8>>, delete_avatar: Option<bool>, state: State| -> HandlerResult<&'static str> {
        async {
            let connection = state.db_pool().get().await?;
    
            if let Some(nickname) = nickname {
                connection.execute(
                    "UPDATE users SET nickname = $1 WHERE id = $2",
                    &[
                        &nickname,
                        &user_id,
                    ],
                ).await?;
            }

            let should_delete_avatar = if let Some(delete_avatar) = delete_avatar {
                delete_avatar
            } else {
                false
            };

            if should_delete_avatar {
                fs::delete(format!("{}/user/{}/avatar.jpg", *STORAGE_DIR, user_id)).await?;
            } else {
                if let Some(avatar) = avatar {
                    fs::store(
                        format!("{}/user/{}", *STORAGE_DIR, user_id),
                        "avatar.jpg".to_string(),
                        avatar,
                    )
                    .await?;
                }
            }
    
            Ok("Successfully updated.")
        }
        .await
        .map_err(|err: Error| reject::custom(err))
    })
    .boxed()
}

pub fn filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    path::end().and(
        patch_filter(state.clone())
    )
    .or(
        path("avatar").and(
            avatar::filter(state.clone())
        )
    )
    .boxed()
}