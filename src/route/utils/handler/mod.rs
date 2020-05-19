use warp::reject::Rejection;

pub mod fs;

pub type HandlerResult<T> = Result<T, Rejection>;