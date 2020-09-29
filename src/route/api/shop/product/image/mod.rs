use warp::{
    Filter,
    reply::Reply,
    filters::BoxedFilter,
    get,
    path,
    query,
};
use crate::{
    route::utils::handler::fs,
    sql::UuidNN,
    STORAGE_DIR,
};

#[derive(Deserialize)]
struct GetArgs {
    shop_id: UuidNN,
    product_key: UuidNN,
}

fn get_filter() -> BoxedFilter<(impl Reply,)> {
    get()
    .and(query())
    .map(|args: GetArgs| {
        (
            format!("{}/shop/{}/product/{}/image.jpg", *STORAGE_DIR, args.shop_id, args.product_key),
        )
    })
    .untuple_one()
    .and_then(fs::read_handler)
    .boxed()
}

pub fn filter() -> BoxedFilter<(impl Reply,)> {
    path::end().and(
        get_filter()
    )
    .boxed()
}