use warp::{
    Filter,
    reply::{
        Reply,
    },
    reject,
    filters::BoxedFilter,
    post,
    patch,
    delete,
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
    sql::{
        from_str,
        UuidNN,
        TextNN,
        IntNN,
    },
    error::Error,
};

#[derive(Serialize, Deserialize)]
struct CreateArgs {
    shop_id: UuidNN,
    product_key: UuidNN,
    remark: Option<String>,
    #[serde(deserialize_with = "from_str")]
    count: IntNN,
    cus_sel: TextNN,
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
                "SELECT cart_create_item($1, $2, $3, $4, $5, $6);",
                &[
                    &UuidNN(gssid),
                    &args.shop_id,
                    &args.product_key,
                    &args.remark,
                    &args.count,
                    &args.cus_sel,
                ],
            ).await?;

            Ok("Successfully create cart item.")
        }
        .await
        .map_err(|err: Error| reject::custom(err))
    })
    .boxed()
}

#[derive(Serialize, Deserialize)]
struct UpdateArgs {
    shop_id: UuidNN,
    item_key: UuidNN,
    payload: TextNN,
}

fn update_filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    patch()
    .and(cookie::to_uuid("GSSID"))
    .and(body::json())
    .and(state)
    .and_then(async move |gssid: Uuid, args: UpdateArgs, state: State| -> HandlerResult<&'static str> {
        async {
            let conn = state.db_pool().get().await?;
            
            conn.execute(
                "SELECT cart_update_item($1, $2, $3, $4);",
                &[
                    &UuidNN(gssid),
                    &args.shop_id,
                    &args.item_key,
                    &args.payload,
                ],
            ).await?;

            Ok("Successfully update cart item.")
        }
        .await.map_err(|err: Error| reject::custom(err))
    })
    .boxed()
}

#[derive(Serialize, Deserialize)]
struct DeleteArgs {
    shop_id: UuidNN,
    item_key: UuidNN,
}

fn delete_filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    delete()
    .and(cookie::to_uuid("GSSID"))
    .and(body::json())
    .and(state)
    .and_then(async move |gssid: Uuid, args: DeleteArgs, state: State| -> HandlerResult<&'static str> {
        async {
            let conn = state.db_pool().get().await?;

            conn.execute(
                "SELECT cart_delete_item($1, $2, $3);",
                &[
                    &UuidNN(gssid),
                    &args.shop_id,
                    &args.item_key,
                ],
            ).await?;

            Ok("Successfully delete cart item.")
        }
        .await
        .map_err(|err: Error| reject::custom(err))
    })
    .boxed()
}

pub fn filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    path::end().and(
        create_filter(state.clone())
        .or(update_filter(state.clone()))
        .or(delete_filter(state.clone()))
    )
    .boxed()
}