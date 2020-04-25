use warp::{
    Filter,
    reply::{Reply, html, Response},
    path, cookie, body,
    post, delete,
};
use uuid::Uuid;
use super::{
    BoxedFilter,
    FilterResult,
};
use super::parse_uuid_optional;
use crate::state::State;
use crate::sql::TextNN;
use crate::error::Error;

#[derive(Serialize, Deserialize)]
struct CreateArgs {
    name: TextNN,
    shop: TextNN,
    product: TextNN,
}

#[derive(Serialize, Deserialize)]
struct DeleteArgs {
    name: TextNN,
    shop: TextNN,
    product: TextNN,
}

fn create_filter(state: BoxedFilter<State>) -> BoxedFilter<impl Reply> {
    post()
    .and(cookie::optional("session_id"))
    .and(body::json())
    .and(state)
    .and_then(async move |session_cookie: Option<String>, args: CreateArgs, state: State| -> FilterResult<Response> {
        async {
            let session_id = parse_uuid_optional(session_cookie)?;
            let conn = state.db_pool().get().await?;
            let (user_id,) = query_one!(
                conn,
                "SELECT get_session_user($1) AS id",
                &[&session_id],
                (id: Uuid),
            )?;
            let (shop_id,) = query_one!(
                conn,
                "SELECT shop_name_to_id($1) AS id",
                &[&args.shop],
                (id: Uuid),
            )?;
            let _ = conn.execute(
                "SELECT check_shop_products_write_authority($1, $2)",
                &[&user_id, &shop_id],
            ).await?;
            let _ = conn.execute(
                "SELECT shop_product_create_customize($1, $2, $3)",
                &[&shop_id, &args.product, &args.name],
            ).await?;
            Ok(html("Successfully created customize.").into_response())
        }
        .await
        .or_else(|error: Error| Ok(error.into_response()))
    })
    .boxed()
}

fn delete_filter(state: BoxedFilter<State>) -> BoxedFilter<impl Reply> {
    delete()
    .and(cookie::optional("session_id"))
    .and(body::json())
    .and(state)
    .and_then(async move |session_cookie: Option<String>, args: DeleteArgs, state: State| -> FilterResult<Response> {
        async {
            let session_id = parse_uuid_optional(session_cookie)?;
            let conn = state.db_pool().get().await?;
            let (user_id,) = query_one!(
                conn,
                "SELECT get_session_user($1) AS id",
                &[&session_id],
                (id: Uuid),
            )?;
            let (shop_id,) = query_one!(
                conn,
                "SELECT shop_name_to_id($1) AS id",
                &[&args.shop],
                (id: Uuid),
            )?;
            let _ = conn.execute(
                "SELECT check_shop_products_write_authority($1, $2)",
                &[&user_id, &shop_id],
            ).await?;
            let _ = conn.execute(
                "SELECT shop_product_delete_customize($1, $2, $3)",
                &[&shop_id, &args.product, &args.name],
            ).await?;
            Ok(html("Successfully deleted customize.").into_response())
        }
        .await
        .or_else(|error: Error| Ok(error.into_response()))
    })
    .boxed()
}

pub fn filter(state: BoxedFilter<State>) -> BoxedFilter<impl Reply> {
    path::end().and(
        create_filter(state.clone())
        .or(delete_filter(state.clone()))
    )
    .boxed()
}