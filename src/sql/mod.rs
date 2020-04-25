use std::{
    fmt::Display,
    str::FromStr,
    num::ParseIntError,
};
use postgres_types::{ToSql, FromSql};
use serde::de::{self, Deserialize, Deserializer};

#[derive(Serialize, Deserialize, Debug, ToSql, FromSql)]
#[postgres(name = "permission")]
pub enum Permission {
    #[postgres(name = "none")]
    None,
    #[postgres(name = "read-only")]
    ReadOnly,
    #[postgres(name = "all")]
    All,
}

#[derive(Serialize, Deserialize, Debug, ToSql, FromSql)]
#[postgres(name = "text_nn")]
pub struct TextNN(String);

#[derive(Serialize, Deserialize, Debug, ToSql, FromSql)]
#[postgres(name = "int_nn")]
pub struct IntNN(i32);

impl FromStr for IntNN {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(IntNN(i32::from_str(s)?))
    }
}

pub fn from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where T: FromStr,
          T::Err: Display,
          D: Deserializer<'de>
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(de::Error::custom)
}

#[macro_export]
macro_rules! query_one {
    (
        $conn:ident,
        $statement:expr,
        $params:expr,
        ($($column:ident: $type:ty),+),
    ) => {
        match $conn.query_one($statement, $params).await {
            Ok(row) => {
                let res: ($($type,)+) = ($(row.get(stringify!($column)),)+);
                Ok(res)
            },
            Err(error) => Err(error),
        }
    }
}