#![feature(async_closure)]

#[macro_use] extern crate serde_derive;

use warp::Filter;

#[macro_use] mod error;
#[macro_use] mod state;
mod filter;
mod argument;

use filter::*;
use argument::parse_arguments;
use state::{State, init_pool};

const DEFAULT_PORT: &'static str = "8001";
const PG_CONFIG: &'static str = "host=localhost user=postgres dbname=postgres";

#[tokio::main]
async fn main() {
    ::std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    let args = parse_arguments();
    let port = args.value_of("port").unwrap().parse::<u16>().expect("parse argument PORT.");

    let db_pool = init_pool(PG_CONFIG, 16).await;
    let state = map_state(State::init(db_pool));

    let get_filter = warp::get().and(get_access_filter(state.clone()));
    let post_filter = warp::post().and(post_access_filter(state.clone()));

    warp::serve(
        get_filter
        .or(post_filter)
    ).run(([127, 0, 0, 1], port)).await;
}
