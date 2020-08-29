use warp::{
    Filter,
    reply::{
        Reply,
        Response,
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
        response,
        handler::HandlerResult,
    },
    state::State,
    sql::{
        UuidNN,
    },
    error::Error,
};

#[derive(Serialize, Deserialize)]
struct CreateArgs {
    shop_id: UuidNN,
}

fn create_filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    post()
    .and(cookie::to_uuid_optional("GSSID"))
    .and(body::json())
    .and(state)
    .and_then(async move |gssid_cookie: Option<Uuid>, args: CreateArgs, state: State| -> HandlerResult<Response> {
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
        create_filter(state.clone())
    )
    .boxed()
}