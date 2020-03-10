#[macro_use] mod db;
#[macro_use] mod sql;

pub use db::{Pool, init_pool};

#[derive(Clone)]
pub struct State {
    db_pool: Pool
}

impl State {
    pub fn init(db_pool: Pool) -> Self {
        State {
            db_pool: db_pool,
        }
    }

    pub fn db_pool(&self) -> &Pool {
        &self.db_pool
    }
}
