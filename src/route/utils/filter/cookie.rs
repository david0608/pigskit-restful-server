use warp::{
    Filter,
    reject,
    filters::BoxedFilter,
    cookie,
};
use uuid::Uuid;
use crate::{
    state::State,
    sql::UuidNN,
    error::Error,
};

pub fn to_user_id(name: &'static str, state: BoxedFilter<(State,)>) -> BoxedFilter<(Uuid,)> {
    to_uuid_optional(name)
    .and(state)
    .and_then(async move |cookie: Option<Uuid>, state: State| {
        async {
            if let Some(ussid) = cookie {
                let conn = state.db_pool().get().await?;
                let (user_id,) = query_one!(
                    conn,
                    "SELECT get_session_user($1) AS id",
                    &[&UuidNN(ussid)],
                    (id: Uuid),
                )?;
                Ok(user_id)
            } else {
                Err(Error::no_valid_cookie(name))
            }
        }
        .await
        .map_err(|err: Error| reject::custom(err))
    })
    .boxed()
}

pub fn to_uuid_optional(name: &'static str) -> BoxedFilter<(Option<Uuid>,)> {
    cookie::optional(name)
    .map(|cookie: Option<String>| {
        if let Some(s) = cookie {
            if let Ok(id) = Uuid::parse_str(s.as_str()) {
                Some(id)
            } else {
                None
            }
        } else {
            None
        }
    })
    .boxed()
}

pub fn to_uuid(name: &'static str) -> BoxedFilter<(Uuid,)> {
    to_uuid_optional(name)
    .and_then(async move |cookie: Option<Uuid>| {
        cookie.ok_or(
            reject::custom(Error::no_valid_cookie(name))
        )
    })
    .boxed()
}