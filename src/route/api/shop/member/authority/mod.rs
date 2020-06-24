use warp::{
    Filter,
    reply::Reply,
    reject,
    filters::BoxedFilter,
    patch,
    path,
    body,
};
use uuid::Uuid;
use crate::{
    route::utils::{
        filter::cookie,
        handler::HandlerResult,
    },
    sql::{
        UuidNN,
        AuthorityNN,
        PermissionNN,
    },
    state::State,
    error::Error,
};

#[derive(Serialize, Deserialize)]
struct UpdateArgs {
    shop_id: UuidNN,
    member_id: UuidNN,
    authority: AuthorityNN,
    permission: PermissionNN,
}

fn update_filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    patch()
    .and(cookie::to_user_id("USSID", state.clone()))
    .and(body::json())
    .and(state)
    .and_then(async move |user_id: Uuid, args: UpdateArgs, state: State| -> HandlerResult<&'static str> {
        async {
            let conn = state.db_pool().get().await?;
            conn.execute(
                "SELECT shop_user_update_authority($1, $2, $3, $4, $5)",
                &[
                    &UuidNN(user_id),
                    &args.shop_id,
                    &args.member_id,
                    &args.authority,
                    &args.permission,
                ],
            ).await?;
            Ok("Successfully setted shop member authority.")
        }
        .await
        .map_err(|err: Error| reject::custom(err))
    })
    .boxed()
}

pub fn filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    path::end().and(
        update_filter(state.clone())
    )
    .boxed()
}