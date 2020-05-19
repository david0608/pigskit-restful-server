use warp::{
    Filter,
    filters::BoxedFilter,
    reject,
    multipart::{
        form,
        FormData,
        Part,
    },
};
use futures::{
    future,
    TryFutureExt,
    TryStreamExt,
};
use bytes::BufMut;
use crate::{
    error::Error,
};

pub fn image(name: &'static str, max_length: u64) -> BoxedFilter<(String, Vec<u8>)> {
    form()
    .max_length(max_length)
    .and_then(async move |form: FormData| {
        form.try_filter_map(async move |part| -> Result<Option<(String, Part)>, warp::Error> {
            if let Some(content_type) = part.content_type() {
                if part.name() == "image" {
                    match content_type {
                        "image/jpeg" => Ok(Some((format!("{}.jpg", name), part))),
                        _ => Ok(None),
                    }
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        })
        .and_then(|(name, part)| {
            part.stream()
            .try_fold(
                Vec::new(),
                |mut buf, data| {
                    buf.put(data);
                    async move { Ok(buf) }
                },
            )
            .and_then(|buf| {
                future::ok::<(String, Vec<u8>), warp::Error>((name, buf))
            })
        })
        .try_collect()
        .await
        .map_err(|err| err.into(): Error)
        .and_then(|mut collect: Vec<(String, Vec<u8>)>| {
            if collect.len() == 1 {
                Ok(collect.remove(0))
            } else {
                Err(Error::Other("No available formdata found."))
            }
        })
        .map_err(|err| reject::custom(err))
    })
    .untuple_one()
    .boxed()
}