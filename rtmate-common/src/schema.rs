// @generated automatically by Diesel CLI.

diesel::table! {
    rt_app (id) {
        id -> BigInt,
        #[max_length = 100]
        app_id -> Varchar,
        #[max_length = 200]
        app_key -> Varchar,
        expire_time -> Nullable<Timestamptz>,
        created_time -> Nullable<Timestamptz>,
        updated_time -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    rt_client_connection (id) {
        id -> Int8,
        app_id -> Int8,
        #[max_length = 100]
        rt_app -> Varchar,
        #[max_length = 100]
        client_id -> Varchar,
        #[max_length = 100]
        connect_token -> Varchar,
        used -> Bool,
        created_time -> Nullable<Timestamptz>,
        expire_time -> Nullable<Timestamptz>,
    }
}
