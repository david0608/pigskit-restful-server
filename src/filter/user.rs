use warp::{
    Filter,
    reply::{Reply, html, Response},
    path,
    post,
    body,
};
use super::{BoxedFilter, FilterResult};
use crate::state::State;
use crate::sql::TextNN;
use crate::error::Error;

#[derive(Serialize, Deserialize)]
struct CreateArgs {
    username: TextNN,
    password: TextNN,
    name: TextNN,
    email: TextNN,
    phone: TextNN,
}

fn create_filter(state: BoxedFilter<State>) -> BoxedFilter<impl Reply> {
    post()
    .and(body::json())
    .and(state)
    .and_then(async move |args: CreateArgs, state: State| -> FilterResult<Response> {
        async {
            let conn = state.db_pool().get().await?;
            let _ = conn.execute(
                "SELECT register_user($1, $2, $3, $4, $5)",
                &[&args.username, &args.password, &args.name, &args.email, &args.phone],
            ).await?;
            Ok(html("Successfully registered.").into_response())
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