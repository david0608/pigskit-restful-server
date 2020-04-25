use warp::{
    Reply,
    http::StatusCode,
    reply::{Response, json, with_status},
};

#[derive(Debug)]
pub enum Error {
    Pool(bb8::RunError::<tokio_postgres::error::Error>),
    Sql(tokio_postgres::error::Error),
    Uuid(uuid::Error),
    Other(&'static str),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pool(err) => write!(f, "Pool error: {}", err),
            Self::Sql(err) => write!(f, "Sql error: {}", err),
            Self::Uuid(err) => write!(f, "Uuid error: {}", err),
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
impl_from_for_error!(&'static str, Other);

impl Into<String> for Error {
    fn into(self) -> String {
        format!("{}", self)
    }
}

impl Into<ErrorMessage> for Error {
    fn into(self) -> ErrorMessage {
        let code: String = match &self {
            Self::Sql(error) => {
                if let Some(sqlstate) = error.code() {
                    sqlstate.code().to_string()
                } else {
                    "0".to_string()
                }
            },
            _ => "0".to_string(),
        };

        ErrorMessage {
            code: code,
            message: self.into(),
        }
    }
}

impl Reply for Error {
    fn into_response(self) -> Response {
        let status = match self {
            Self::Pool(_) | Self::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Sql(_) | Self::Uuid(_) => StatusCode::BAD_REQUEST,
        };

        let msg: ErrorMessage = self.into();

        with_status(json(&msg), status).into_response()
    }
}

#[derive(Serialize)]
struct ErrorMessage {
    code: String,
    message: String,
}