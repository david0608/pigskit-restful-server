use warp::{
    Reply,
    reject::Reject,
    http::StatusCode,
    reply::{Response, json, with_status},
};

#[derive(Debug)]
pub enum Error {
    Pool(bb8::RunError::<tokio_postgres::error::Error>),
    Sql(tokio_postgres::error::Error),
    Uuid(uuid::Error),
    Warp(warp::Error),
    Io(std::io::Error),
    Other(&'static str),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pool(err) => write!(f, "Pool error: {}", err),
            Self::Sql(err) => write!(f, "Sql error: {}", err),
            Self::Uuid(err) => write!(f, "Uuid error: {}", err),
            Self::Warp(err) => write!(f, "Warp error: {}", err),
            Self::Io(err) => write!(f, "IO error: {}", err),
            Self::Other(err) => write!(f, "Other error: {}", err),
        }
    }
}

macro_rules! impl_from_for_error {
    ($F:ty, $E:ident) => {
        impl From<$F> for Error {
            fn from(err: $F) -> Self {
                Error::$E(err)
            }
        }
    }
}

impl_from_for_error!(bb8::RunError<tokio_postgres::error::Error>, Pool);
impl_from_for_error!(tokio_postgres::error::Error, Sql);
impl_from_for_error!(uuid::Error, Uuid);
impl_from_for_error!(warp::Error, Warp);
impl_from_for_error!(std::io::Error, Io);
impl_from_for_error!(&'static str, Other);

impl Into<String> for &Error {
    fn into(self) -> String {
        format!("{}", self)
    }
}

impl From<&Error> for ErrorMessage {
    fn from(err: &Error) -> Self {
        let code: String = match err {
            Error::Sql(error) => {
                if let Some(sqlstate) = error.code() {
                    sqlstate.code().to_string()
                } else {
                    "0".to_string()
                }
            },
            _ => "0".to_string()
        };

        ErrorMessage {
            code: code,
            message: err.into()
        }
    }
}

impl Reply for &Error {
    fn into_response(self) -> Response {
        let status = match self {
            Error::Pool(_) | Error::Other(_) | Error::Warp(_) | Error::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Sql(_) | Error::Uuid(_) => StatusCode::BAD_REQUEST,
        };
        let msg = ErrorMessage::from(self);
        with_status(json(&msg), status).into_response()
    }
}

#[derive(Serialize)]
struct ErrorMessage {
    code: String,
    message: String,
}

impl Reject for Error {}