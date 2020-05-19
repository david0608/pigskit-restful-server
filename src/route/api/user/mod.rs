use warp::{
    Filter,
    reply::Reply,
    reject,
    filters::BoxedFilter,
    post,
    path,
    body,
};
use crate::{
    route::utils::handler::HandlerResult,
    state::State,
    sql::TextNZ,
    error::Error,
};

mod session;

#[derive(Serialize, Deserialize)]
struct CreateArgs {
    username: TextNZ,
    password: TextNZ,
    name: TextNZ,
    email: TextNZ,
    phone: TextNZ,
}

fn create_filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    post()
    .and(body::json())
    .and(state)
    .and_then(async move |args: CreateArgs, state: State| -> HandlerResult<&'static str> {
        async {
            let conn = state.db_pool().get().await?;
            let _ = conn.execute(
                "SELECT register_user($1, $2, $3, $4, $5)",
                &[&args.username, &args.password, &args.name, &args.email, &args.phone],
            ).await?;
            Ok("Successfully registered.")
        }
        .await
        .map_err(|err: Error| reject::custom(err))
    })
    .boxed()
}

pub fn filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    path::end().and(
        create_filter(state.clone())
    )
    .or(
        path("session").and(
            session::filter(state.clone())
        )
    )
    .boxed()
}