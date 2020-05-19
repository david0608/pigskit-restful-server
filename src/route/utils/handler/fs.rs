use warp::{
    reject,
};
use futures::{
    future::Future,
    TryFutureExt,
};
use tokio::{
    fs,
    prelude::*,
};
use crate::{
    route::utils::handler::HandlerResult,
    error::Error,
};

type StoreResult = impl Future<Output = HandlerResult<&'static str>> + Send;
pub fn store() -> impl Fn(String, String, Vec<u8>) -> StoreResult + Clone {
    async move |dir: String, name: String, buf: Vec<u8>| {
        fs::create_dir_all(&dir)
        .and_then(|_| {
            fs::write(format!("{}/{}", dir, name), buf)
        })
        .await
        .or_else(|err| Err(reject::custom(err.into(): Error)))?;
        Ok("Successfully stored.")
    }
}

pub async fn read(file: String) -> HandlerResult<Vec<u8>> {
    let mut data = Vec::new();
    fs::File::open(file)
    .await
    .map_err(|err| reject::custom(err.into(): Error))?
    .read_to_end(&mut data)
    .await
    .map_err(|err| reject::custom(err.into(): Error))?;
    Ok(data)
}

pub async fn read_with_default(file: String, default_file: String) -> HandlerResult<Vec<u8>> {
    let mut data = Vec::new();
    fs::File::open(file)
    .or_else(|_| {
        fs::File::open(default_file)
    })
    .await
    .map_err(|err| reject::custom(err.into(): Error))?
    .read_to_end(&mut data)
    .await
    .map_err(|err| reject::custom(err.into(): Error))?;
    Ok(data)
}

pub async fn delete(file: String) -> HandlerResult<&'static str> {
    fs::remove_file(file)
    .await
    .map_err(|err| reject::custom(err.into(): Error))?;
    Ok("Successfully deleted.")
}