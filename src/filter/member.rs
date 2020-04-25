use warp::{
    Filter,
    reply::{Reply, html, Response},
    path, cookie, body,
    post,
};
use uuid::Uuid;
use super::{BoxedFilter, FilterResult};
use super::parse_uuid_optional;
use crate::state::State;
use crate::sql::TextNN;
use crate::error::Error;

#[derive(Serialize, Deserialize)]
struct CreateArgs {
    shop: TextNN,
    member: TextNN,
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
            let (member_id,) = query_one!(
                conn,
                "SELECT username_to_id($1) AS id",
                &[&args.member],
                (id: Uuid),
            )?;
            let (shop_id,) = query_one!(
                conn,
                "SELECT shop_name_to_id($1) AS id",
                &[&args.shop],
                (id: Uuid),
            )?;
            let _ = conn.execute(
                "SELECT add_shop_member($1, $2, $3)",
                &[&user_id, &shop_id, &member_id],
            ).await?;
            Ok(html("Successfully added shop member.").into_response())
        }
        .await
        .or_else(|error: Error| Ok(error.into_response()))
    })
    .boxed()
}

pub fn filter(state: BoxedFilter<State>) -> BoxedFilter<impl Reply> {
    path::end().and(
        create_filter(state.clone())
    )
    .boxed()
}