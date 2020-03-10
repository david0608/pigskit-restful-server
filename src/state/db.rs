use tokio_postgres::NoTls;
use bb8_postgres::PostgresConnectionManager;

pub type Pool = bb8::Pool<PostgresConnectionManager<NoTls>>;

pub async fn init_pool(config: &str, size: u32) -> Pool {
    let config = config.parse().expect("parse pg config.");
    let manager = PostgresConnectionManager::new(
        config,
        NoTls,
    );
    Pool::builder()
        .max_size(size)
        .build(manager)
        .await
        .expect("init pool")
}
