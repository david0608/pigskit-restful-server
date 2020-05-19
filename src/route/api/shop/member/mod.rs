use warp::{
    Filter,
    reply::Reply,
    reject,
    filters::BoxedFilter,
    post,
    path,
    body,
};
use uuid::Uuid;
use crate::{
    route::utils::{
        filter::cookie,
        handler::HandlerResult,
    },
    sql::UuidNN,
    state::State,
    error::Error,
};

mod authority;

#[derive(Serialize, Deserialize)]
struct CreateArgs {
    shop_id: UuidNN,
    member_id: UuidNN,
}

fn create_filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    post()
    .and(cookie::session_user_id(state.clone()))
    .and(body::json())
    .and(state)
    .and_then(async move |user_id: Uuid, args: CreateArgs, state: State| -> HandlerResult<&'static str> {
        async {
            let conn = state.db_pool().get().await?;
            conn.execute(
                "SELECT shop_user_create($1, $2, $3)",
                &[
                    &UuidNN(user_id),
                    &args.shop_id,
                    &args.member_id,
                ],
            ).await?;
            Ok("Successfully added shop member.")
        }
        .await
        .map_err(|err: Error| reject::custom(err))
    })
    .boxed()
}

pub fn filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    path::end().and(
        create_filter(state.clone())
    )
    .or(
        path("authority").and(
            authority::filter(state.clone())
        )
    )
    .boxed()
}