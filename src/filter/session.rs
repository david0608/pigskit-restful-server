use warp::{
    Filter,
    reply::{Reply, html, Response},
    path,
    cookie,
    get,
    post,
    delete,
    body,
};
use uuid::Uuid;
use super::{BoxedFilter, FilterResult};
use super::parse_uuid_optional;
use crate::state::State;
use crate::filter::response;
use crate::sql::TextNN;
use crate::error::Error;

#[derive(Serialize, Deserialize)]
struct CreateArgs {
    username: TextNN,
    password: TextNN,
}

fn create_filter(state: BoxedFilter<State>) -> BoxedFilter<impl Reply> {
    post()
    .and(cookie::optional("session_id"))
    .and(body::json())
    .and(state)
    .and_then(async move |session_cookie: Option<String>, args: CreateArgs, state: State| -> FilterResult<Response> {
        async {
            let conn = state.db_pool().get().await?;

            // Delete the session if existed, or do nothing.
            if let Ok(session_id) = parse_uuid_optional(session_cookie) {
                let _ = conn.execute(
                    "SELECT signout_user($1)",
                    &[&session_id],
                ).await;
            }

            let (session_id,) = query_one!(
                conn,
                "SELECT signin_user($1, $2) AS id",
                &[&args.username, &args.password],
                (id: Uuid),
            )?;

            Ok(response::set_cookie(format!("session_id={}; Path=/; HttpOnly", session_id)))
        }
        .await
        .or_else(|error: Error| Ok(error.into_response()))
    })
    .boxed()
}

fn read_filter(state: BoxedFilter<State>) -> BoxedFilter<impl Reply> {
    get()
    .and(cookie::optional("session_id"))
    .and(state)
    .and_then(async move |session_cookie: Option<String>, state: State| -> FilterResult<Response> {
        async {
            let session_id = parse_uuid_optional(session_cookie)?;
            let conn = state.db_pool().get().await?;
            let (_user_id,) = query_one!(
                conn,
                "SELECT get_session_user($1) AS id",
                &[&session_id],
                (id: Uuid),
            )?;
            Ok(response::redirect_to("/api/user/session/success"))
        }
        .await
        .or_else(|error: Error| Ok(error.into_response()))
    })
    .boxed()
}

fn delete_filter(state: BoxedFilter<State>) -> BoxedFilter<impl Reply> {
    delete()
    .and(cookie::optional("session_id"))
    .and(state)
    .and_then(async move |session_cookie: Option<String>, state: State| -> FilterResult<Response> {
        async {
            let conn = state.db_pool().get().await?;
            if let Ok(session_id) = parse_uuid_optional(session_cookie) {
                let _ = conn.execute(
                    "SELECT signout_user($1)",
                    &[&session_id],
                ).await;
            }
            Ok(response::set_cookie("session_id=; Path=/; HttpOnly".to_owned()))
        }
        .await
        .or_else(|error: Error| Ok(error.into_response()))
    })
    .boxed()
}

fn success_filter() -> BoxedFilter<impl Reply> {
    get()
    .and(cookie::cookie("session_id"))
    .map(|session_id: String| {
        html(format!("session_id: {}", session_id))
    })
    .boxed()
}

pub fn filter(state: BoxedFilter<State>) -> BoxedFilter<impl Reply> {
    path::end().and(
        create_filter(state.clone())
        .or(read_filter(state.clone()))
        .or(delete_filter(state.clone()))
    )
    .or(success_filter())
    .boxed()
}