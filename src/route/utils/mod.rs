use uuid::Uuid;
use crate::error::Error;

pub mod filter;
pub mod handler;
pub mod response;

pub fn parse_uuid_optional(param: Option<String>) -> Result<Uuid, Error> {
    if let Some(id) = param {
        parse_uuid(id)
    } else {
        Err(Error::Other("Optional UUID parameter not specified."))
    }
}

pub fn parse_uuid(id: String) -> Result<Uuid, Error> {
    Uuid::parse_str(id.as_str()).map_err(|err| err.into())
}