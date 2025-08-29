use diesel::Selectable;
use diesel::Queryable;
use serde::Deserialize;
use serde::Serialize;
use crate::schema::rt_app;
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