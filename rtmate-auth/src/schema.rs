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
