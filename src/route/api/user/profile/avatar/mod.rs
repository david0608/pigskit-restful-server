use warp::{
    Filter,
    reply::Reply,
    filters::BoxedFilter,
    reject,
    get,
    path,
    query,
};
use uuid::Uuid;
use crate::{
    route::utils::{
        filter::cookie,
        handler::{
            HandlerResult,
            fs,
        },
    },
    state::State,
    error::Error,
    STORAGE_DIR,
};

#[derive(Deserialize)]
struct GetArgs {
    default: Option<bool>
}

fn get_filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    get()
    .and(query())
    .and(cookie::to_user_id("USSID", state.clone()))
    .and_then(async move |args: GetArgs, user_id: Uuid| -> HandlerResult<Vec<u8>> {
        async {
            if let Ok(data) = fs::read(format!("{}/user/{}/avatar.jpg", *STORAGE_DIR, user_id)).await {
                Ok(data)
            } else {
                if args.default.unwrap_or(false) {
                    fs::read(format!("{}/default/user/avatar.jpg", *STORAGE_DIR))
                    .await
                    .map_err(|_| Error::data_not_found("avatar"))
                } else {
                    Err(Error::data_not_found("avatar"))
                }
            }

        }
        .await
        .map_err(|err: Error| reject::custom(err))
    })
    .boxed()
}

pub fn filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    path::end().and(
        get_filter(state.clone())
    )
    .boxed()
}