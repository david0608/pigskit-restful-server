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
    query,
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
            let mut connection = state.db_pool().get().await?;
            let transaction = connection.transaction().await?;

            let (auth,) = query_one!(
                transaction,
                "SELECT check_shop_user_authority($1, $2, 'product_authority', 'all') AS auth;",
                &[&UuidNN(shop_id), &UuidNN(user_id)],
                (auth: bool),
            )?;

            if !auth { return Err(Error::unauthorized()) }

            transaction.query(
                "SELECT shop_set_product_has_picture($1, $2, $3);",
                &[&UuidNN(shop_id), &UuidNN(product_key), &true],
            ).await?;

            fs::store(
                format!("{}/shop/{}/product/{}", *STORAGE_DIR, shop_id, product_key),
                "image.jpg".to_string(),
                image_data,
            )
            .await?;

            transaction.commit().await?;

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
    .and(query())
    .map(|args: Args| {
        (
            format!("{}/shop/{}/product/{}/image.jpg", *STORAGE_DIR, args.shop_id, args.product_key),
        )
    })
    .untuple_one()
    .and_then(fs::read_handler)
    .boxed()
}

fn delete_filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    delete()
    .and(cookie::to_user_id("USSID", state.clone()))
    .and(body::json())
    .and(state)
    .and_then(async move |user_id: Uuid, args: Args, state: State| -> HandlerResult<&'static str> {
        async {
            let mut connection = state.db_pool().get().await?;
            let transaction = connection.transaction().await?;

            let (auth,) = query_one!(
                transaction,
                "SELECT check_shop_user_authority($1, $2, 'product_authority', 'all') AS auth;",
                &[&args.shop_id, &UuidNN(user_id)],
                (auth: bool),
            )?;

            if !auth { return Err(Error::unauthorized()) }

            transaction.query(
                "SELECT shop_set_product_has_picture($1, $2, $3);",
                &[&args.shop_id, &args.product_key, &false],
            ).await?;

            fs::delete(format!("{}/shop/{}/product/{}/image.jpg", *STORAGE_DIR, args.shop_id, args.product_key)).await?;

            transaction.commit().await?;

            Ok("Successfully delete image.")
        }
        .await
        .map_err(|err: Error| reject::custom(err))
    })
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