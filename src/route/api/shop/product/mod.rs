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
    multipart::{
        form,
        FormData,
    }
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
    sql::{
        from_str,
        TextNN,
        UuidNN,
    },
    state::State,
    error::Error,
    STORAGE_DIR,
};

fn create_filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    post()
    .and(cookie::to_user_id("USSID", state.clone()))
    .and(
        form_filter!(
            shop_id [ Uuid ]
            payload [ String ]
            image [ Option [ Vec<u8> ] ]
        )
    )
    .and(state)
    .and_then(async move |user_id: Uuid, shop_id: Uuid, payload: String, image: Option<Vec<u8>>, state: State| -> HandlerResult<&'static str> {
        async {
            let mut connection = state.db_pool().get().await?;
            let transaction = connection.transaction().await?;

            let (ok,) = query_one!(
                transaction,
                "SELECT check_shop_user_authority($1, $2, 'product_authority', 'all') AS ok;",
                &[&UuidNN(shop_id), &UuidNN(user_id)],
                (ok: bool),
            )?;

            if !ok { return Err(Error::unauthorized()) }

            let (product_key,) = query_one!(
                transaction,
                "SELECT product_key FROM shop_create_product($1, $2);",
                &[&UuidNN(shop_id), &TextNN(payload)],
                (product_key: Uuid),
            )?;
            
            if let Some(data) = image {
                fs::store(
                    format!("{}/shop/{}/product/{}", *STORAGE_DIR, shop_id, product_key),
                    "image.jpg".to_string(),
                    data,
                )
                .await?;
            }

            transaction.commit().await?;
            Ok("Succefully created.")
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
                Err(Error::unauthorized())
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
                Err(Error::unauthorized())
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