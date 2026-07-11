use chrono::{Duration, Local};
use jsonwebtoken::{encode, EncodingKey, Header};
use rtmate_common::dto::Claims;
use uuid::Uuid;

fn main() {
    let app_id = "abcdef";
    let app_key = "af57761c55de41a7aef0a5e940f751af";
    let client_id = "client_test_001";

    let now = Local::now();
    let exp = now + Duration::hours(2);
    let jti = Uuid::new_v4().as_simple().to_string();
    let claims = Claims::new(
        app_id.to_string(),
        client_id.to_string(),
        jti,
        now.to_utc(),
        exp.to_utc(),
    );

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(app_key.as_ref()),
    )
    .expect("generate token failed");

    println!("{}", token);
}
