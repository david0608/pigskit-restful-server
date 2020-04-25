use warp::{
    Filter,
    path, cors, cookie, body,
    reply::{Reply, html, Response},
    post,
};
use uuid::Uuid;
use super::{BoxedFilter, FilterResult};
use super::{
    parse_uuid_optional,
    preflight_filter,
};
use crate::state::State;
use crate::sql::TextNN;
use crate::error::Error;

#[derive(Serialize, Deserialize)]
struct CreateArgs {
    name: TextNN,
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
            let _ = conn.execute(
                "SELECT create_shop($1, $2)",
                &[&user_id, &args.name],
            ).await?;
            Ok(html("Successfully created shop.").into_response())
        }
        .await
        .or_else(|error: Error| Ok(error.into_response()))
    })
    .with(cors().allow_origin("http://localhost"))
    .boxed()
}

pub fn filter(state: BoxedFilter<State>) -> BoxedFilter<impl Reply> {
    path::end().and(
        create_filter(state.clone())
        .or(preflight_filter())
    )
    .boxed()
}