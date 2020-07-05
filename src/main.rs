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

use state::{State, init_pool};

const DEFAULT_PORT: u16 = 80;
const DEFAULT_PORT_DEV: u16 = 8001;
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

    let mut port = DEFAULT_PORT;
    let mut pg_config = PG_CONFIG;

    let args = argument::parse_arguments();
    let is_dev = args.is_present("dev");
    if is_dev {
        port = DEFAULT_PORT_DEV;
        pg_config = PG_CONFIG_DEV;
    }
    if let Some(p) = argument::args_port(&args) {
        port = p;
    }

    let db_pool = init_pool(pg_config, 16).await;

    if is_dev {
        warp::serve(route::dev_routes(State::init(db_pool))).run(([0, 0, 0, 0], port)).await;
    } else {
        warp::serve(route::routes(State::init(db_pool))).run(([0, 0, 0, 0], port)).await;
    }
}
