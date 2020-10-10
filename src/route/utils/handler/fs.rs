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

pub async fn store(dir: String, name: String, buf: Vec<u8>) -> Result<(), std::io::Error> {
    fs::create_dir_all(&dir)
    .and_then(|_| {
        fs::write(format!("{}/{}", dir, name), buf)
    })
    .await
}

pub async fn read(file: String) -> Result<Vec<u8>, std::io::Error> {
    let mut data = Vec::new();
    fs::File::open(file)
    .await?
    .read_to_end(&mut data)
    .await?;
    Ok(data)
}

pub async fn delete(file: String) -> Result<(), std::io::Error> {
    fs::remove_file(file)
    .await
}

pub async fn delete_all(file: String) -> Result<(), std::io::Error> {
    fs::remove_dir_all(file)
    .await
}

type StoreResult = impl Future<Output = HandlerResult<&'static str>> + Send;
#[allow(dead_code)]
pub fn store_handler() -> impl Fn(String, String, Vec<u8>) -> StoreResult + Clone {
    async move |dir: String, name: String, buf: Vec<u8>| {
        store(dir, name, buf)
        .await
        .map_err(|err| {
            let err: Error = err.into();
            reject::custom(err)
        })?;
        Ok("Successfully stored.")
    }
}

pub async fn read_handler(file: String) -> HandlerResult<Vec<u8>> {
    let data = read(file)
    .await
    .map_err(|err| {
        let err: Error = err.into();
        reject::custom(err)
    })?;
    Ok(data)
}

#[allow(dead_code)]
pub async fn delete_handler(file: String) -> HandlerResult<&'static str> {
    delete(file)
    .await
    .map_err(|err| {
        let err: Error = err.into();
        reject::custom(err)
    })?;
    Ok("Successfully deleted.")
}