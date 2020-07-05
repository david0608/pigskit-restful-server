use warp::{
    Filter,
    reply::Reply,
    filters::BoxedFilter,
    reject,
    post,
    delete,
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
        from_str,
        TextNN,
        UuidNN,
    },
    state::State,
    error::Error,
};

#[derive(Serialize, Deserialize)]
struct CreateArgs {
    #[serde(deserialize_with = "from_str")]
    shop_id: UuidNN,
    payload: TextNN,
}

fn create_filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    post()
    .and(cookie::to_user_id("USSID", state.clone()))
    .and(body::json())
    .and(state)
    .and_then(async move |user_id: Uuid, args: CreateArgs, state: State| -> HandlerResult<&'static str> {
        async {
            let conn = state.db_pool().get().await?;
            let (ok,) = query_one!(
                conn,
                "SELECT check_shop_user_authority($1, $2, 'product_authority', 'all') AS ok;",
                &[&args.shop_id, &UuidNN(user_id)],
                (ok: bool),
            )?;
            if ok {
                conn.execute(
                    "SELECT shop_create_product($1, $2);",
                    &[&args.shop_id, &args.payload],
                ).await?;
                Ok("Successfully created product.")
            } else {
                Err(Error::permission_denied())
            }
        }
        .await
        .map_err(|err: Error| reject::custom(err))
    })
    .boxed()
}

#[derive(Serialize, Deserialize)]
struct DeleteArgs {
    #[serde(deserialize_with = "from_str")]
    shop_id: UuidNN,
    #[serde(deserialize_with = "from_str")]
    product_key: UuidNN,
}

fn delete_filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    delete()
    .and(cookie::to_user_id("USSID", state.clone()))
    .and(body::json())
    .and(state)
    .and_then(async move |user_id: Uuid, args: DeleteArgs, state: State| -> HandlerResult<&'static str> {
        async {
            let conn = state.db_pool().get().await?;
            let (ok,) = query_one!(
                conn,
                "SELECT check_shop_user_authority($1, $2, 'product_authority', 'all') AS ok;",
                &[&args.shop_id, &UuidNN(user_id)],
                (ok: bool),
            )?;
            if ok {
                conn.execute(
                    "SELECT shop_delete_product($1, $2)",
                    &[&args.shop_id, &args.product_key],
                ).await?;
                Ok("Successfully deleted product.")
            } else {
                Err(Error::permission_denied())
            }
        }
        .await
        .map_err(|err: Error| reject::custom(err))
    })
    .boxed()
}

#[derive(Serialize, Deserialize)]
struct UpdateArgs {
    #[serde(deserialize_with = "from_str")]
    shop_id: UuidNN,
    #[serde(deserialize_with = "from_str")]
    product_key: UuidNN,
    payload: TextNN,
}

fn update_filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    patch()
    .and(cookie::to_user_id("USSID", state.clone()))
    .and(body::json())
    .and(state)
    .and_then(async move |user_id: Uuid, args: UpdateArgs, state: State| -> HandlerResult<&'static str> {
        async {
            let conn = state.db_pool().get().await?;
            let (ok,) = query_one!(
                conn,
                "SELECT check_shop_user_authority($1, $2, 'product_authority', 'all') AS ok;",
                &[&args.shop_id, &UuidNN(user_id)],
                (ok: bool),
            )?;
            if ok {
                conn.execute(
                    "SELECT shop_update_product($1, $2, $3);",
                    &[&args.shop_id, &args.product_key, &args.payload],
                ).await?;
                Ok("Seccessfully updated product.")
            } else {
                Err(Error::permission_denied())
            }
        }
        .await
        .map_err(|err: Error| reject::custom(err))
    })
    .boxed()
}

pub fn filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    path::end().and(
        create_filter(state.clone())
        .or(delete_filter(state.clone()))
        .or(update_filter(state.clone()))
    )
    .boxed()
}