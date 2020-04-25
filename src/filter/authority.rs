use warp::{
    Filter,
    reply::{Reply, html, Response},
    path, cookie, body,
    patch,
};
use uuid::Uuid;
use super::{BoxedFilter, FilterResult};
use super::parse_uuid_optional;
use crate::state::State;
use crate::sql::{
    Permission,
    TextNN,
};
use crate::error::Error;

#[derive(Serialize, Deserialize)]
struct UpdateArgs {
    shop: TextNN,
    member: TextNN,
    authority: TextNN,
    permission: Permission,
}

fn update_filter(state: BoxedFilter<State>) -> BoxedFilter<impl Reply> {
    patch()
    .and(cookie::optional("session_id"))
    .and(body::json())
    .and(state)
    .and_then(async move |session_cookie: Option<String>, args: UpdateArgs, state: State| -> FilterResult<Response> {
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
                "SELECT set_shop_member_authority($1, $2, $3, $4, $5)",
                &[&user_id, &shop_id, &member_id, &args.authority, &args.permission],
            ).await?;
            Ok(html("Successfully setted shop member authority.").into_response())
        }
        .await
        .or_else(|error: Error| Ok(error.into_response()))
    })
    .boxed()
}

pub fn filter(state: BoxedFilter<State>) -> BoxedFilter<impl Reply> {
    path::end().and(
        update_filter(state.clone())
    )
    .boxed()
}