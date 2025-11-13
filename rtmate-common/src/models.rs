use chrono::Local;
use diesel::Selectable;
use diesel::Queryable;
use diesel::Insertable;
use serde::Deserialize;
use serde::Serialize;
use crate::schema::rt_app;
use crate::schema::rt_client_connection;
use chrono::DateTime;

#[derive(Queryable, Selectable,Deserialize, Serialize, Debug)]
#[diesel(table_name = rt_app)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RtApp {

    pub id: i64,
    pub app_id: String,
    pub app_key: String,
    pub expire_time: Option<DateTime<Local>>,
    pub created_time: Option<DateTime<Local>>,
    pub updated_time: Option<DateTime<Local>>,

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
    pub created_time: Option<DateTime<Local>>,
    pub expire_time: Option<DateTime<Local>>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = rt_client_connection)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewRtClientConnection {
    pub app_id: i64,
    pub rt_app: String,
    pub client_id: String,
    pub connect_token: String,
    pub used: bool,
    pub expire_time: Option<DateTime<Local>>,
}