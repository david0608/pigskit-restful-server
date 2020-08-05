use warp::{
    Filter,
    reply::{
        Reply,
        Response,
        json,
        Json,
    },
    reject,
    filters::BoxedFilter,
    post,
    get,
    delete,
    path,
    body,
};
use uuid::Uuid;
use crate::{
    route::utils::{
        filter::cookie,
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
    .and(cookie::to_uuid_optional("USSID"))
    .and(body::json())
    .and(state)
    .and_then(async move |ussid_cookie: Option<Uuid>, args: CreateArgs, state: State| -> HandlerResult<Response> {
        async {
            let conn = state.db_pool().get().await?;

            // Delete the session if existed, or do nothing.
            if let Some(ussid) = ussid_cookie {
                let _ = conn.execute(
                    "SELECT signout_user($1)",
                    &[&UuidNN(ussid)],
                ).await;
            }

            let row = conn.query_one(
                "SELECT signin_user($1, $2) AS id",
                &[&args.username, &args.password],
            ).await?;
            
            if let Ok(session_id) = row.try_get::<&str, Uuid>("id") {
                Ok(response::set_cookie(format!("USSID={}; Path=/; HttpOnly", session_id)))
            } else {
                return Err(Error::unauthorized())
            }
        }
        .await
        .map_err(|err: Error| reject::custom(err))
    })
    .boxed()
}

#[derive(Serialize)]
struct GetRes {
    username: Option<String>,
    nickname: Option<String>,
}
fn get_filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    get()
    .and(cookie::to_user_id("USSID", state.clone()))
    .and(state)
    .and_then(async move |user_id: Uuid, state: State| -> HandlerResult<Json> {
        async {
            let conn = state.db_pool().get().await?;
            let row = conn.query_one(
                "SELECT username, nickname FROM users WHERE id = $1",
                &[&user_id],
            ).await?;
            Ok(json(&GetRes {
                username: row.get("username"),
                nickname: row.get("nickname"),
            }))
        }
        .await
        .map_err(|err: Error| reject::custom(err))
    })
    .boxed()
}

fn delete_filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    delete()
    .and(cookie::to_uuid_optional("USSID"))
    .and(state)
    .and_then(async move |ussid_cookie: Option<Uuid>, state: State| -> HandlerResult<Response> {
        async {
            let conn = state.db_pool().get().await?;
            if let Some(ussid) = ussid_cookie {
                let _ = conn.execute(
                    "SELECT signout_user($1)",
                    &[&UuidNN(ussid)],
                ).await;
            }
            Ok(response::set_cookie("USSID=; Path=/; HttpOnly".to_owned()))
        }
        .await
        .map_err(|err: Error| reject::custom(err))
    })
    .boxed()
}

pub fn filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    path::end().and(
        create_filter(state.clone())
        .or(get_filter(state.clone()))
        .or(delete_filter(state.clone()))
    )
    .boxed()
}