use warp::{
    Filter,
    reply::Reply,
    filters::BoxedFilter,
    reject,
    get,
    post,
    delete,
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
            HandlerResult,
            fs,
        },
    },
    sql::{
        from_str,
        UuidNN,
    },
    state::State,
    error::Error,
    STORAGE_DIR,
};

fn update_filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    post()
    .and(cookie::to_user_id("USSID", state.clone()))
    .and(
        form_filter!(
            shop_id [ Uuid ]
            product_key [ Uuid ]
            image [ Vec<u8> ]
        )
    )
    .and(state)
    .and_then(async move |user_id: Uuid, shop_id: Uuid, product_key: Uuid, image_data: Vec<u8>, state: State| -> HandlerResult<&'static str> {
        async {
            let conn = state.db_pool().get().await?;
            let (auth,) = query_one!(
                conn,
                "SELECT check_shop_user_authority($1, $2, 'product_authority', 'all') AS auth;",
                &[&UuidNN(shop_id), &UuidNN(user_id)],
                (auth: bool),
            )?;

            if !auth { return Err(Error::unauthorized()) }

            if conn.query(
                "SELECT key FROM query_shop_products($1) WHERE key = $2;",
                &[&UuidNN(shop_id), &product_key],
            ).await?.len() == 0 {
                return Err(Error::data_not_found("Product"))
            }

            fs::store(
                format!("{}/shop/{}/product/{}", *STORAGE_DIR, shop_id, product_key),
                "image.jpg".to_string(),
                image_data,
            )
            .await?;

            Ok("Successfully updated.")
        }
        .await
        .map_err(|err: Error| reject::custom(err))
    })
    .boxed()
}

#[derive(Serialize, Deserialize)]
struct Args {
    #[serde(deserialize_with = "from_str")]
    shop_id: UuidNN,
    #[serde(deserialize_with = "from_str")]
    product_key: UuidNN,
}

fn read_filter() -> BoxedFilter<(impl Reply,)> {
    get()
    .and(body::json())
    .map(|args: Args| {
        (
            format!("{}/shop/{}/product/{}/image.jpg", *STORAGE_DIR, args.shop_id, args.product_key),
            format!("{}/default/shop/product/image.jpg", *STORAGE_DIR),
        )
    })
    .untuple_one()
    .and_then(fs::read_with_default_handler)
    .boxed()
}

fn delete_filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    delete()
    .and(cookie::to_user_id("USSID", state.clone()))
    .and(body::json())
    .and(state)
    .and_then(async move |user_id: Uuid, args: Args, state: State| -> HandlerResult<String> {
        async {
            let conn = state.db_pool().get().await?;
            let (auth,) = query_one!(
                conn,
                "SELECT check_shop_user_authority($1, $2, 'product_authority', 'all') AS auth;",
                &[&args.shop_id, &UuidNN(user_id)],
                (auth: bool),
            )?;

            if !auth { return Err(Error::unauthorized()) }

            if conn.query(
                "SELECT key FROM query_shop_products($1) WHERE key = $2;",
                &[&args.shop_id, &args.product_key],
            ).await?.len() == 0 {
                return Err(Error::data_not_found("Product"))
            }

            Ok(format!("{}/shop/{}/product/{}/image.jpg", *STORAGE_DIR, args.shop_id, args.product_key))
        }
        .await
        .map_err(|err: Error| reject::custom(err))
    })
    .and_then(fs::delete_handler)
    .boxed()
}

pub fn filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    path::end().and(
        update_filter(state.clone())
        .or(read_filter())
        .or(delete_filter(state.clone()))
    )
    .boxed()
}