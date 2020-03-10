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
