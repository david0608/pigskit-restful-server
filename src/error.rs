use warp::{
    Reply,
    reply::{
        Response,
        json,
        with_status,
    },
    reject::Reject,
    http::StatusCode,
};
use serde::Serialize;

#[derive(Debug)]
pub struct Error {
    http_status: StatusCode,
    r#type: String,
    message: String,
    data: Option<String>,
    inner: Option<InnerError>,
}

impl Error {
    fn new(http_status: StatusCode, r#type: &str, message: &str, data: Option<String>) -> Self {
        Error {
            http_status: http_status,
            r#type: r#type.to_string(),
            message: message.to_string(),
            data: data,
            inner: None,
        }
    }

    pub fn bad_request(r#type: &str, message: &str, data: Option<String>) -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            r#type,
            message,
            data,
        )
    }

    fn internal(inner: InnerError) -> Self {
        Error {
            http_status: StatusCode::INTERNAL_SERVER_ERROR,
            r#type: "InternalServerError".to_string(),
            message: "Internal server error.".to_string(),
            data: None,
            inner: Some(inner),
        }
    }

    pub fn unsupported_operation() -> Self {
        Self::bad_request(
            "UnsuportedOperation",
            "Unsupported operation.",
            None,
        )
    }

    pub fn operation_failed() -> Self {
        Self::bad_request(
            "OperationFailed",
            "Operation failed.",
            None,
        )
    }

    pub fn unauthorized() -> Self {
        Self::new(
            StatusCode::FORBIDDEN,
            "Unauthorized",
            "Unauthorized.",
            None,
        )
    }

    pub fn missing_body(field: &str) -> Self {
        Self::bad_request(
            "BodyMissingField",
            format!(r#"Missing field "{}" in request body."#, field).as_str(),
            None,
        )
    }

    pub fn invalid_data(field: &str) -> Self {
        Self::bad_request(
            "InvalidData",
            format!(r#"Invalid data in field "{}" in request body."#, field).as_str(),
            None,
        )
    }

    pub fn no_valid_form(part: &str) -> Self {
        Self::bad_request(
            "FormMissingPart",
            format!(r#"Missing or invalid part "{}" in form."#, part).as_str(),
            None,
        )
    }

    pub fn no_valid_cookie(name: &str) -> Self {
        Self::bad_request(
            "NoValidCookie",
            format!(r#"Missing or invalid cookie "{}" in request."#, name).as_str(),
            None,
        )
    }

    pub fn session_expired(name: &str) -> Self {
        Self::bad_request(
            "SessionExpired",
            format!(r#"Session for cookie "{}" has expired."#, name).as_str(),
            None,
        )
    }

    pub fn unique_data_conflict(name: &str) -> Self {
        Self::new(
            StatusCode::CONFLICT,
            "UniqueDataConflict",
            format!(r#"The unique data "{}" existed."#, name).as_str(),
            None,
        )
    }

    pub fn data_not_found(name: &str) -> Self {
        Self::bad_request(
            "DataNotFound",
            format!(r#"Data "{}" not found."#, name).as_str(),
            None,
        )
    }

    pub fn payload_too_large() -> Self {
        Self::bad_request(
            "PayloadTooLarge",
            "Size of request payload too large.",
            None,
        )
    }

    pub fn is_inner(&self) -> bool {
        self.inner.is_some()
    }
}

#[derive(Debug)]
enum InnerError {
    Bb8(bb8::RunError::<tokio_postgres::error::Error>),
    Sql(tokio_postgres::error::Error),
    Uuid(uuid::Error),
    Warp(warp::Error),
    Io(std::io::Error),
}

macro_rules! impl_from_for_error {
    ($from:ty, $inner:ident) => {
        impl From<$from> for InnerError {
            fn from(err: $from) -> Self {
                InnerError::$inner(err)
            }
        }

        impl From<$from> for Error {
            fn from(err: $from) -> Self {
                Error::internal(err.into())
            }
        }
    }
}

impl_from_for_error!(bb8::RunError<tokio_postgres::error::Error>, Bb8);
impl_from_for_error!(uuid::Error, Uuid);
impl_from_for_error!(warp::Error, Warp);
impl_from_for_error!(std::io::Error, Io);

impl From<tokio_postgres::error::Error> for InnerError {
    fn from(err: tokio_postgres::error::Error) -> Self {
        InnerError::Sql(err)
    }
}

impl From<tokio_postgres::error::Error> for Error {
    fn from(err: tokio_postgres::error::Error) -> Self {
        let code = if let Some(state) = err.code() {
            state.code()
        } else {
            ""
        };

        match code {
            "C2002" => {
                Error::session_expired("USSID")
            }
            "C3001" => {
                Error::session_expired("GSSID")
            }
            "C4101" => {
                Error::data_not_found("customize_selection")
            }
            "C4301" => {
                Error::bad_request(
                    "CusSelNotProvided",
                    "Customize selection not provided.",
                    None,
                )
            }
            "C6001" => {
                Error::bad_request(
                    "ShopNameUsed",
                    "Shop name has been used.",
                    None,
                )
            }
            "C6009" => {
                Error::data_not_found("shop_product")
            }
            "C8001" => {
                Error::data_not_found("cart")
            }
            "C8002" => {
                Error::data_not_found("cart_item")
            }
            "C8103" => {
                Error::bad_request(
                    "CartItemExpired",
                    "Cart item expired.",
                    None,
                )
            }
            _ => {
                Error::internal(err.into())
            }
        }
    }
}

impl Reject for Error {}

#[derive(Serialize)]
struct ApiError<'a> {
    status: u16,
    r#type: &'a String,
    message: &'a String,
    data: serde_json::Value,
}

impl Reply for &Error {
    fn into_response(self) -> Response {
        let data = if let Some(data) = &self.data {
            if let Ok(data) = serde_json::from_str(&data) {
                data
            } else {
                serde_json::Value::Null
            }
        } else {
            serde_json::Value::Null
        };

        with_status(
            json(&ApiError {
                status: self.http_status.as_u16(),
                r#type: &self.r#type,
                message: &self.message,
                data: data,
            }),
            self.http_status,
        )
        .into_response()
    }
}