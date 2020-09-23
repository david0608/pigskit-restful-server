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
    get,
    post,
    patch,
    path,
    body,
    query,
};
use uuid::Uuid;
use regex::Regex;
use crate::{
    route::utils::{
        filter::cookie,
        handler::HandlerResult,
        response::set_cookie,
    },
    state::State,
    error::Error,
};

lazy_static! {
    static ref RE_VALID_EMAIL: Regex = Regex::new(r#"^\w+@(?:\w+\.)+\w+$"#).unwrap();
    static ref RE_VALID_PHONE: Regex = Regex::new(r#"^09\d{8}$"#).unwrap();
    static ref RE_VALID_USERNAME: Regex = Regex::new(r#"^[A-Za-z0-9]+$"#).unwrap();
    static ref RE_VALID_PASSWORD: Regex = Regex::new(r#"^[A-Za-z0-9]+$"#).unwrap();
    static ref RE_VALID_PASSWORD_UPPER: Regex = Regex::new(r#"[A-Z]"#).unwrap();
    static ref RE_VALID_PASSWORD_LOWER: Regex = Regex::new(r#"[a-z]"#).unwrap();
    static ref RE_VALID_PASSWORD_NUMBER: Regex = Regex::new(r#"[0-9]"#).unwrap();    
}

#[derive(Deserialize)]
struct GetArgs {
    operation: Option<String>,
}

#[derive(Serialize)]
struct GetRes {
    data: Option<String>,
}

fn get_filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    get()
    .and(cookie::to_uuid("REGSSID"))
    .and(query::<GetArgs>())
    .and(state)
    .and_then(async move |regssid: Uuid, mut args: GetArgs, state: State| -> HandlerResult<Json> {
        async {
            let conn = state.db_pool().get().await?;
            let operation = args.operation.take().ok_or_else(|| Error::missing_body("operation"))?;
            let field;
            match operation.as_str() {
                "email" | "phone" | "username" => {
                    field = operation.as_str();
                }
                _ => return Err(Error::unsupported_operation())
            }

            if let Some(row) = conn.query_opt(
                format!("SELECT {0} FROM user_register_session WHERE id = $1", field).as_str(),
                &[&regssid],
            ).await? {
                return Ok(json(&GetRes {
                    data: row.get(field),
                }))
            } else {
                return Err(Error::session_expired("REGSSID"))
            }
        }
        .await
        .map_err(|err: Error| reject::custom(err))
    })
    .boxed()
}

fn post_filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    post()
    .and(cookie::to_uuid_optional("REGSSID"))
    .and(state)
    .and_then(async move |regssid_cookie: Option<Uuid>, state: State| -> HandlerResult<Response> {
        async {
            let conn = state.db_pool().get().await?;

            // Delete the session if existed.
            if let Some(regssid) = regssid_cookie {
                let _ = conn.execute(
                    "DELETE FROM user_register_session WHERE id = $1",
                    &[&regssid],
                ).await;
            }

            let (regssid,) = query_one!(
                conn,
                "INSERT INTO user_register_session (id) VALUES (uuid_generate_v4()) RETURNING id",
                &[],
                (id: Uuid),
            )?;
            Ok(set_cookie("REGSSID", &regssid.to_string(), 1))
        }
        .await
        .map_err(|err: Error| reject::custom(err))
    })
    .boxed()
}

#[derive(Serialize, Deserialize)]
struct PatchArgs {
    operation: Option<String>,
    data: Option<String>,
}

fn patch_filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    patch()
    .and(cookie::to_uuid("REGSSID"))
    .and(body::json())
    .and(state)
    .and_then(async move |regssid: Uuid, mut args: PatchArgs, state: State| -> HandlerResult<&'static str> {
        async {
            let conn = state.db_pool().get().await?;
            let operation = args.operation.take().ok_or_else(|| Error::missing_body("operation"))?;

            match operation.as_str() {
                "submit" => {
                    let (ok,) = query_one!(
                        conn,
                        "SELECT register_user($1) AS ok",
                        &[&regssid],
                        (ok: bool),
                    )?;
                    if ok {
                        return Ok("Success.")
                    } else {
                        return Err(Error::operation_failed())
                    }
                }
                _ => {
                    let data = args.data.take().ok_or_else(|| Error::missing_body("data"))?;
                    let field;
                    let is_unique;
                    match operation.as_str() {
                        "email" => {
                            if !RE_VALID_EMAIL.is_match(&data) {
                                return Err(Error::invalid_data("data"))
                            }
                            field = "email";
                            is_unique = true;
                        }
                        "phone" => {
                            if !RE_VALID_PHONE.is_match(&data) {
                                return Err(Error::invalid_data("data"))
                            }
                            field = "phone";
                            is_unique = true;
                        }
                        "username" => {
                            if !RE_VALID_USERNAME.is_match(&data) {
                                return Err(Error::invalid_data("data"))
                            }
                            field = "username";
                            is_unique = true;
                        }
                        "password" => {
                            if !RE_VALID_PASSWORD.is_match(&data)
                                || !RE_VALID_PASSWORD_UPPER.is_match(&data)
                                || !RE_VALID_PASSWORD_LOWER.is_match(&data)
                                || !RE_VALID_PASSWORD_NUMBER.is_match(&data)
                            {
                                return Err(Error::invalid_data("data"))
                            }
                            field = "password";
                            is_unique = false;
                        }
                        _ => return Err(Error::unsupported_operation())
                    }

                    if is_unique {
                        if let Some(_) = conn.query_opt(
                            format!("SELECT {0} FROM users WHERE {0} = $1", field).as_str(),
                            &[&data],
                        ).await? {
                            return Err(Error::unique_data_conflict(field))
                        }
                    }
                    if let 1 = conn.execute(
                        format!("UPDATE user_register_session SET {0} = $1 WHERE id = $2", field).as_str(),
                        &[&data, &regssid],
                    ).await? {
                        return Ok("Success.")
                    } else {
                        return Err(Error::session_expired("REGSSID"))
                    }
                }
            }
        }
        .await
        .map_err(|err: Error| reject::custom(err))
    })
    .boxed()
}

pub fn filter(state: BoxedFilter<(State,)>) -> BoxedFilter<(impl Reply,)> {
    path::end().and(
        get_filter(state.clone())
        .or(
            post_filter(state.clone())
        )
        .or(
            patch_filter(state.clone())
        )
    )
    .boxed()
}