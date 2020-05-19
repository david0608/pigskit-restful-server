use warp::{
    Filter,
    reply::{
        Reply,
        Response,
        html,
    },
    reject,
    filters::BoxedFilter,
    post,
    get,
    delete,
    path,
    cookie,
    body,
};
use uuid::Uuid;
use crate::{
    route::utils::{
        filter,
        parse_uuid_optional,
        response,
        handler::HandlerResult,
    },
    state::State,
    sql::{
        TextNZ,
        UuidNN,
    },
    error::Error,
};

#[derive(Serialize, Deserialize)]
struct CreateArgs {
    username: TextNZ,
    password: TextNZ,
}

fn create_filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    post()
    .and(cookie::optional("session_id"))
    .and(body::json())
    .and(state)
    .and_then(async move |session_cookie: Option<String>, args: CreateArgs, state: State| -> HandlerResult<Response> {
        async {
            let conn = state.db_pool().get().await?;

            // Delete the session if existed, or do nothing.
            if let Ok(session_id) = parse_uuid_optional(session_cookie) {
                let _ = conn.execute(
                    "SELECT signout_user($1)",
                    &[&UuidNN(session_id)],
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
        .map_err(|err: Error| reject::custom(err))
    })
    .boxed()
}

fn read_filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    get()
    .and(filter::cookie::session_user_id(state.clone()))
    .and_then(async move |_user_id: Uuid| -> HandlerResult<Response> {
        Ok(response::redirect_to("/api/user/session/success"))
    })
    .boxed()
}

fn delete_filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    delete()
    .and(cookie::optional("session_id"))
    .and(state)
    .and_then(async move |session_cookie: Option<String>, state: State| -> HandlerResult<Response> {
        async {
            let conn = state.db_pool().get().await?;
            if let Ok(session_id) = parse_uuid_optional(session_cookie) {
                let _ = conn.execute(
                    "SELECT signout_user($1)",
                    &[&UuidNN(session_id)],
                ).await;
            }
            Ok(response::set_cookie("session_id=; Path=/; HttpOnly".to_owned()))
        }
        .await
        .map_err(|err: Error| reject::custom(err))
    })
    .boxed()
}

fn success_filter() -> BoxedFilter<(impl Reply,)> {
    get()
    .and(cookie::cookie("session_id"))
    .map(|session_id: String| {
        html(format!("session_id: {}", session_id))
    })
    .boxed()
}

pub fn filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    path::end().and(
        create_filter(state.clone())
        .or(read_filter(state.clone()))
        .or(delete_filter(state.clone()))
    )
    .or(success_filter())
    .boxed()
}