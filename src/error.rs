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
    inner: Option<InnerError>,
}

impl Error {
    fn new(http_status: StatusCode, r#type: String, message: String) -> Self {
        Error {
            http_status: http_status,
            r#type: r#type,
            message: message,
            inner: None,
        }
    }

    fn internal(inner: InnerError) -> Self {
        Error {
            http_status: StatusCode::INTERNAL_SERVER_ERROR,
            r#type: "InternalServerError".to_string(),
            message: "Internal server error.".to_string(),
            inner: Some(inner),
        }
    }

    pub fn unsupported_operation() -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            "UnsuportedOperation".to_string(),
            "Unsupported operation.".to_string(),
        )
    }

    pub fn operation_failed() -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            "OperationFailed".to_string(),
            "Operation failed.".to_string(),
        )
    }

    pub fn unauthorized() -> Self {
        Self::new(
            StatusCode::FORBIDDEN,
            "Unauthorized".to_string(),
            "Unauthorized.".to_string(),
        )
    }

    pub fn missing_body(field: &str) -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            "BodyMissingField".to_string(),
            format!(r#"Missing field "{}" in request body."#, field),
        )
    }

    pub fn invalid_data(field: &str) -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            "InvalidData".to_string(),
            format!(r#"Invalid data in field "{}" in request body."#, field),
        )
    }

    pub fn no_valid_form(part: &str) -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            "FormMissingPart".to_string(),
            format!(r#"Missing or invalid part "{}" in form."#, part),
        )
    }

    pub fn no_valid_cookie(name: &str) -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            "NoValidCookie".to_string(),
            format!(r#"Missing or invalid cookie "{}" in request."#, name),
        )
    }

    pub fn session_expired(name: &str) -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            "SessionExpired".to_string(),
            format!(r#"Session for cookie "{}" has expired."#, name),
        )
    }

    pub fn unique_data_conflict(name: &str) -> Self {
        Self::new(
            StatusCode::CONFLICT,
            "UniqueDataConflict".to_string(),
            format!(r#"The unique data "{}" existed."#, name),
        )
    }

    pub fn data_not_found(name: &str) -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            "DataNotFound".to_string(),
            format!(r#"Data "{}" not found."#, name),
        )
    }

    pub fn payload_too_large() -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            "PayloadTooLarge".to_string(),
            "Size of request payload too large.".to_string(),
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
            "C5001" => {
                Error::new(
                    StatusCode::BAD_REQUEST,
                    "ShopNameUsed".to_string(),
                    "Shop name has been used.".to_string(),
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
}

impl Reply for &Error {
    fn into_response(self) -> Response {
        with_status(
            json(&ApiError {
                status: self.http_status.as_u16(),
                r#type: &self.r#type,
                message: &self.message,
            }),
            self.http_status,
        )
        .into_response()
    }
}