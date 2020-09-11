use warp::{
    Filter,
    reply::{
        Reply,
        Response,
    },
    reject,
    filters::BoxedFilter,
    path,
    body,
    put,
};
use uuid::Uuid;
use crate::{
    route::utils::{
        filter::cookie,
        response,
        handler::HandlerResult,
    },
    state::State,
    sql::{
        UuidNN,
    },
    error::Error,
};

mod item;
mod order;

#[derive(Serialize, Deserialize)]
struct PutArgs {
    shop_id: UuidNN,
}

fn put_filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    put()
    .and(cookie::to_uuid_optional("GSSID"))
    .and(body::json())
    .and(state)
    .and_then(async move |gssid_cookie: Option<Uuid>, args: PutArgs, state: State| -> HandlerResult<Response> {
        async {
            let conn = state.db_pool().get().await?;
            
            let (gssid,) = query_one!(
                conn,
                "SELECT put_cart($1, $2) AS gssid;",
                &[&gssid_cookie, &args.shop_id],
                (gssid: Uuid),
            )?;

            Ok(response::set_cookie(format!("GSSID={}; Path=/; HttpOnly", gssid)))
        }
        .await
        .map_err(|err: Error| reject::custom(err))
    })
    .boxed()
}

pub fn filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    path::end().and(
        put_filter(state.clone())
    )
    .or(
        path("item").and(
            item::filter(state.clone())
        )
    )
    .or(
        path("order").and(
            order::filter(state.clone())
        )
    )
    .boxed()
}