use std::{
    fmt,
    str::FromStr,
    num::ParseIntError,
};
use postgres_types::{ToSql, FromSql};
use serde::de::{self, Deserialize, Deserializer};
use uuid::Uuid;

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
#[postgres(name = "permission_nn")]
pub struct PermissionNN(pub Permission);

#[derive(Serialize, Deserialize, Debug, ToSql, FromSql)]
#[postgres(name = "authority")]
pub enum Authority {
    #[postgres(name = "member_authority")]
    MemberAuthority,
    #[postgres(name = "order_authority")]
    OrderAuthority,
    #[postgres(name = "product_authority")]
    ProductAuthority,
}

#[derive(Serialize, Deserialize, Debug, ToSql, FromSql)]
#[postgres(name = "authority_nn")]
pub struct AuthorityNN(pub Authority);

#[derive(Serialize, Deserialize, Debug, ToSql, FromSql)]
#[postgres(name = "text_nn")]
pub struct TextNN(pub String);

#[derive(Serialize, Deserialize, Debug, ToSql, FromSql)]
#[postgres(name = "text_nz")]
pub struct TextNZ(pub String);

#[derive(Serialize, Deserialize, Debug, ToSql, FromSql)]
#[postgres(name = "int_nn")]
pub struct IntNN(pub i32);

impl FromStr for IntNN {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(IntNN(i32::from_str(s)?))
    }
}

#[derive(Serialize, Deserialize, Debug, ToSql, FromSql)]
#[postgres(name = "uuid_nn")]
pub struct UuidNN(pub Uuid);

impl fmt::Display for UuidNN {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for UuidNN {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(UuidNN(Uuid::parse_str(s)?))
    }
}

pub fn from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where T: FromStr,
          T::Err: fmt::Display,
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

#[cfg(test)]
mod test {
    use postgres_types::{ToSql, FromSql};
    use crate::state::init_pool;
    use crate::PG_CONFIG_DEV;
    use super::{
        TextNZ,
        IntNN,
    };

    #[derive(Serialize, Deserialize, Debug, ToSql, FromSql)]
    #[postgres(name = "option")]
    struct SqlOption {
        name: TextNZ,
        price: IntNN,
    }

    #[tokio::test]
    async fn test_option() {
        let pool = init_pool(PG_CONFIG_DEV, 1).await;
        let conn = pool.get().await.unwrap();
        let (opt,) = query_one!(
            conn,
            "SELECT option_create('new', 123)",
            &[],
            (option_create: SqlOption),
        ).unwrap();
        println!("opt:{:?}", opt);
    }
}