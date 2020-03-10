use warp::{
    Filter,
    reply::{Reply, html, Response},
    path,
    query,
    cookie,
};
use uuid::Uuid;
use super::{BoxedFilter, FilterResult};
use super::parse_uuid_optional;
use crate::state::State;
use crate::filter::response;
use crate::error::Error;

#[derive(Deserialize)]
struct SigninArgs {
    username: String,
    password: String,
}

fn signin_filter(state: BoxedFilter<State>) -> BoxedFilter<Response> {
    path("signin")
        .and(query::<SigninArgs>())
        .and(cookie::optional("session_id"))
        .and(state)
        .and_then(async move |args: SigninArgs, session_cookie: Option<String>, state: State| -> FilterResult<Response> {
            async {
                let conn = state.db_pool().get().await?;

                // Delete the session if existed, or do nothing.
                if let Ok(session_id) = parse_uuid_optional(session_cookie) {
                    let _ = conn.execute(
                        "DELETE FROM session WHERE id = $1",
                        &[&session_id],
                    ).await;
                }

                let (account_id,) = query_one!(
                    conn,
                    "SELECT id FROM account WHERE username = $1 AND password = $2",
                    &[&args.username, &args.password],
                    (id: Uuid),
                )?;

                let (session_id,) = query_one!(
                    conn,
                    "INSERT INTO session (account_id) VALUES ($1) RETURNING id",
                    &[&account_id],
                    (id: Uuid),
                )?;

                Ok(response::set_cookie(format!("session_id={}; Path=/; HttpOnly", session_id)))
            }
            .await
            .or_else(|error: Error| Ok(error.into_response()))
        })
        .boxed()
}

fn signout_filter(state: BoxedFilter<State>) -> BoxedFilter<Response> {
    path("signout")
        .and(cookie::optional("session_id"))
        .and(state)
        .and_then(async move |session_cookie: Option<String>, state: State| -> FilterResult<Response> {
            async {
                let conn = state.db_pool().get().await?;
                if let Ok(session_id) = parse_uuid_optional(session_cookie) {
                    let _ = conn.execute(
                        "DELETE FROM session WHERE id = $1",
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

fn session_filter(state: BoxedFilter<State>) -> BoxedFilter<Response> {
    path("session")
        .and(cookie::optional("session_id"))
        .and(state)
        .and_then(async move |session_cookie: Option<String>, state: State| -> FilterResult<Response> {
            async {
                let session_id = parse_uuid_optional(session_cookie)?;
                let conn = state.db_pool().get().await?;
                let (account_id,) = query_one!(
                    conn,
                    "SELECT account_id FROM session WHERE id = $1",
                    &[&session_id],
                    (account_id: Uuid),
                )?;
                Ok(response::redirect_to("/access/success"))
            }
            .await
            .or_else(|error: Error| Ok(error.into_response()))
        })
        .boxed()
}

fn success_filter() -> BoxedFilter<impl Reply> {
    warp::path("success")
        .and(cookie::cookie("session_id"))
        .map(|session_id: String| {
            html(format!("session_id: {}", session_id))
        })
        .boxed()
}

pub fn get_access_filter(state: BoxedFilter<State>) -> BoxedFilter<impl Reply> {
    path("access").and(
        session_filter(state.clone())
        .or(success_filter())
    )
    .boxed()
}

pub fn post_access_filter(state: BoxedFilter<State>) -> BoxedFilter<impl Reply> {
    path("access").and(
        signin_filter(state.clone())
        .or(signout_filter(state.clone()))
    )
    .boxed()
}