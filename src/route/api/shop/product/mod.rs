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
        TextNN,
        UuidNN,
    },
    state::State,
    error::Error,
    STORAGE_DIR,
};

mod image;

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
                transaction.query(
                    "SELECT shop_set_product_has_picture($1, $2, $3);",
                    &[&UuidNN(shop_id), &UuidNN(product_key), &true],
                ).await?;

                fs::store(
                    format!("{}/shop/{}/product/{}", *STORAGE_DIR, shop_id, product_key),
                    "image.jpg".to_string(),
                    data,
                )
                .await?;
            }

            transaction.commit().await?;
            Ok("Successfully created.")
        }
        .await
        .map_err(|err: Error| reject::custom(err))
    })
    .boxed()
}

#[derive(Serialize, Deserialize)]
struct DeleteArgs {
    shop_id: UuidNN,
    product_key: UuidNN,
}

fn delete_filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    delete()
    .and(cookie::to_user_id("USSID", state.clone()))
    .and(body::json())
    .and(state)
    .and_then(async move |user_id: Uuid, args: DeleteArgs, state: State| -> HandlerResult<&'static str> {
        async {
            let connection = state.db_pool().get().await?;

            let (ok,) = query_one!(
                connection,
                "SELECT check_shop_user_authority($1, $2, 'product_authority', 'all') AS ok;",
                &[&args.shop_id, &UuidNN(user_id)],
                (ok: bool),
            )?;

            if !ok { return Err(Error::unauthorized()) }

            connection.execute(
                "SELECT shop_delete_product($1, $2)",
                &[&args.shop_id, &args.product_key],
            ).await?;

            let _ = fs::delete_all(format!("{}/shop/{}/product/{}", *STORAGE_DIR, args.shop_id, args.product_key)).await;

            Ok("Successfully deleted product.")
        }
        .await
        .map_err(|err: Error| reject::custom(err))
    })
    .boxed()
}

fn patch_filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    patch()
    .and(cookie::to_user_id("USSID", state.clone()))
    .and(
        form_filter!(
            shop_id [ Uuid ]
            product_key [ Uuid ]
            payload [ Option [ String ] ]
            delete_image [ Option [ bool ] ]
            image [ Option [ Vec<u8> ] ]
        )
    )
    .and(state)
    .and_then(async move |user_id: Uuid, shop_id: Uuid, product_key: Uuid, payload: Option<String>, delete_image: Option<bool>, image: Option<Vec<u8>>, state: State| -> HandlerResult<&'static str> {
        async {
            let mut connection = state.db_pool().get().await?;
            let transaction = connection.transaction().await?;

            let (ok,) = query_one!(
                transaction,
                "SELECT check_shop_user_authority($1, $2, 'product_authority', 'all') AS ok;",
                &[
                    &UuidNN(shop_id),
                    &UuidNN(user_id),
                ],
                (ok: bool),
            )?;

            if !ok { return Err(Error::unauthorized()) }

            if let Some(payload) = payload {
                transaction.execute(
                    "SELECT shop_update_product($1, $2, $3);",
                    &[
                        &UuidNN(shop_id),
                        &UuidNN(product_key),
                        &TextNN(payload),
                    ],
                ).await?;
            }
            
            let delete_image = if let Some(delete_image) = delete_image {
                delete_image
            } else {
                false
            };

            if delete_image {
                transaction.execute(
                    "SELECT shop_set_product_has_picture($1, $2, $3);",
                    &[
                        &UuidNN(shop_id),
                        &UuidNN(product_key),
                        &false,
                    ],
                ).await?;
                fs::delete(format!("{}/shop/{}/product/{}/image.jpg", *STORAGE_DIR, shop_id, product_key)).await?;
            } else if let Some(image) = image {
                transaction.execute(
                    "SELECT shop_set_product_has_picture($1, $2, $3);",
                    &[
                        &UuidNN(shop_id),
                        &UuidNN(product_key),
                        &true,
                    ],
                ).await?;
                fs::store(
                    format!("{}/shop/{}/product/{}", *STORAGE_DIR, shop_id, product_key),
                    "image.jpg".to_string(),
                    image,
                )
                .await?;
            }

            transaction.commit().await?;
            Ok("Successfully updated.")
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
        .or(patch_filter(state.clone()))
    )
    .or(
        path("image").and(
            image::filter()
        )
    )
    .boxed()
}