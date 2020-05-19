use warp::{
    Filter,
    reject,
    filters::BoxedFilter,
    cookie,
};
use uuid::Uuid;
use crate::{
    route::utils::parse_uuid,
    state::State,
    sql::UuidNN,
    error::Error,
};

pub fn session_user_id(state: BoxedFilter<(State,)>) -> BoxedFilter<(Uuid,)> {
    cookie::cookie("session_id")
    .and(state)
    .and_then(async move |session_cookie: String, state: State| {
        async {
            let session_id = parse_uuid(session_cookie)?;
            let conn = state.db_pool().get().await?;
            let (user_id,) = query_one!(
                conn,
                "SELECT get_session_user($1) AS id",
                &[&UuidNN(session_id)],
                (id: Uuid),
            )?;
            Ok(user_id)
        }
        .await
        .map_err(|err: Error| reject::custom(err))
    })
    .boxed()
}