#[macro_export]
macro_rules! form_filter {
    ( TYPE Option [ $($type:tt)+ ] ) => {
        Option<$($type)+>
    };

    ( TYPE $($type:tt)+ ) => {
        $($type)+
    };

    ( DECLARE $field:ident Option [ $($type:tt)+ ] ) => {
        form_filter!( DECLARE $field $($type)+ )
    };

    ( DECLARE $field:ident $($type:tt)+ ) => {
        let mut $field: Option<$($type)+> = None;
    };

    ( PARSE $pair:ident $field:ident Option [ $($type:tt)+ ] ) => {
        form_filter!( PARSE $pair $field $($type)+ )
    };

    ( PARSE $pair:ident $field:ident Uuid ) => {
        if let Ok(id_string) = String::from_utf8($pair.1) {
            if let Ok(id) = Uuid::parse_str(id_string.as_str()) {
                $field = Some(id);
            }
        }
    };

    ( PARSE $pair:ident $field:ident String ) => {
        if let Ok(string) = String::from_utf8($pair.1) {
            $field = Some(string);
        }
    };

    ( PARSE $pair:ident $field:ident Vec<u8> ) => {
        $field = Some($pair.1);
    };

    ( CHECK $field:ident Option [ $($type:tt)+ ] ) => {};

    ( CHECK $field:ident $($type:tt)+ ) => {
        if let None = $field {
            return Err(Error::no_valid_form(stringify!($field)))
        }
    };

    ( UNWRAP $field:ident Option [ $($type:tt)+ ] ) => {
        $field
    };

    ( UNWRAP $field:ident $($type:tt)+ ) => {
        $field.unwrap()
    };

    ( $( $field:ident [ $($type:tt)+ ] )+ ) => {
        form()
        .max_length(2000000)
        .and_then(async move |form: FormData| -> HandlerResult<($( form_filter!( TYPE $($type)+ ) ),+,)> {
            async {
                $(
                    form_filter!( DECLARE $field $($type)+ );
                )+

                let pairs: Vec<(String, Vec<u8>)> =
                    form
                    .and_then(|part| {
                        let name = part.name().to_string();
                        part.stream()
                        .try_fold(
                            Vec::new(),
                            |mut buf, data| {
                                buf.put(data);
                                async move { Ok(buf) }
                            },
                        )
                        .map_ok(move |buf| (name, buf))
                    })
                    .try_collect()
                    .await?;

                for pair in pairs {
                    match pair.0.as_str() {
                        $(
                            stringify!($field) => {
                                form_filter!( PARSE pair $field $($type)+ )
                            }
                        )+
                        _ => {}
                    }
                }

                $(
                    form_filter!( CHECK $field $($type)+ );
                )+

                Ok((
                    $(
                        form_filter!( UNWRAP $field $($type)+ )
                    ),+,
                ))
            }
            .await
            .map_err(|err: Error| reject::custom(err))
        })
        .untuple_one()
    };
}