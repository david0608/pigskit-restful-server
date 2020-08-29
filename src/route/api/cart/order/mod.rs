use warp::{
    Filter,
    reply::{
        Reply,
    },
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
    state::State,
    sql::UuidNN,
    error::Error,
};

#[derive(Serialize, Deserialize)]
struct CreateArgs {
    shop_id: UuidNN,
}

fn create_filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    post()
    .and(cookie::to_uuid("GSSID"))
    .and(body::json())
    .and(state)
    .and_then(async move |gssid: Uuid, args: CreateArgs, state: State| -> HandlerResult<&'static str> {
        async {
            let conn = state.db_pool().get().await?;

            conn.execute(
                "SELECT create_order($1, $2);",
                &[
                    &UuidNN(gssid),
                    &args.shop_id,
                ],
            ).await?;

            Ok("Successfully create order.")
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
    .boxed()
}