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
        handler::HandlerResult,
        filter::cookie,
    },
    sql::{
        UuidNN,
        TextNZ,
    },
    state::State,
    error::Error,
};

mod member;
mod product;

#[derive(Serialize, Deserialize)]
struct CreateArgs {
    shop_name: TextNZ,
}

fn create_filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    post()
    .and(cookie::to_user_id("USSID", state.clone()))
    .and(body::json())
    .and(state)
    .and_then(async move |user_id: Uuid, args: CreateArgs, state: State| -> HandlerResult<&'static str> {
        async {
            let conn = state.db_pool().get().await?;
            conn.execute(
                "SELECT create_shop($1, $2)",
                &[&UuidNN(user_id), &args.shop_name],
            ).await?;
            Ok("Successfully created shop.")
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
        path("member").and(
            member::filter(state.clone())
        )
    )
    .or(
        path("product").and(
            product::filter(state.clone())
        )
    )
    .boxed()
}