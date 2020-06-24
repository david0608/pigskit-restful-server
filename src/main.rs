#![feature(async_closure)]
#![feature(type_alias_impl_trait)]

#[macro_use] extern crate serde_derive;
#[macro_use] extern crate log;
#[macro_use] extern crate lazy_static;

mod error;
mod state;
#[macro_use] mod sql;
mod route;
mod argument;

use route::routes;
use argument::parse_arguments;
use state::{State, init_pool};

const DEFAULT_PORT: &'static str = "80";
const PG_CONFIG: &'static str = "host=postgres-server user=postgres dbname=postgres";
const PG_CONFIG_DEV: &'static str = "host=localhost user=postgres dbname=postgres";

lazy_static! {
    static ref STORAGE_DIR: String = {
        let mut path = std::env::current_exe().unwrap();
        assert!(path.pop());
        assert!(path.pop());
        path.push("storage");
        if let Err(err) = std::fs::metadata(path.as_path()) {
            panic!("Failed to config storage path: {:?}, {}", path, err);
        }
        path.as_path().to_str().unwrap().to_owned()
    };
}

#[tokio::main]
async fn main() {
    ::std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    info!("Storage path configed: {}", *STORAGE_DIR);

    let args = parse_arguments();
    let port = args.value_of("port").unwrap().parse::<u16>().expect("parse argument PORT.");
    let pg_config = if args.is_present("dev") {
        PG_CONFIG_DEV
    } else {
        PG_CONFIG
    };

    let db_pool = init_pool(pg_config, 16).await;
    let routes = routes(State::init(db_pool));

    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}
