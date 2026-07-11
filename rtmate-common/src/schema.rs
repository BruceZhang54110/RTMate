// @generated automatically by Diesel CLI.

diesel::table! {
    rt_app (id) {
        id -> Int8,
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
    rt_channel (id) {
        id -> Int8,
        app_id -> Int8,
        #[max_length = 100]
        app_id_str -> Varchar,
        #[max_length = 64]
        name -> Varchar,
        #[max_length = 256]
        description -> Nullable<Varchar>,
        #[max_length = 100]
        created_by -> Nullable<Varchar>,
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

diesel::allow_tables_to_appear_in_same_query!(rt_app, rt_channel, rt_client_connection,);
