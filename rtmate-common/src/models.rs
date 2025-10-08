use diesel::Selectable;
use diesel::Queryable;
use serde::Deserialize;
use serde::Serialize;
use crate::schema::rt_app;
use crate::schema::rt_client_connection;
use chrono::Utc;
use chrono::DateTime;

#[derive(Queryable, Selectable,Deserialize, Serialize, Debug)]
#[diesel(table_name = rt_app)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RtApp {

    pub id: i64,
    pub app_id: String,
    pub app_key: String,
    pub expire_time: Option<DateTime<Utc>>,
    pub created_time: Option<DateTime<Utc>>,
    pub updated_time: Option<DateTime<Utc>>,

}

#[derive(Queryable, Selectable,Deserialize, Serialize, Debug)]
#[diesel(table_name = rt_client_connection)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RtClientConnection {
    pub id: i64,
    pub app_id: i64,
    pub rt_app: String,
    pub client_id: String,
    pub connect_token: String,
    pub used: bool,
    pub created_time: Option<DateTime<Utc>>,
    pub expire_time: Option<DateTime<Utc>>,
}