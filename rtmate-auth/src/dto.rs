use serde::{Deserialize, Serialize};


#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RtAppParam {

    pub app_id: String,
    pub state: String,
    pub signature: String,
    pub timestamp: u64,

}

#[derive(Deserialize, Serialize, Debug)]
pub struct AppAuthResult {
    app_id: String,
    token: String,
}

impl AppAuthResult {
    pub fn new(app_id: String, token: String) -> Self {
        AppAuthResult {
            app_id,
            token,
        }
    }
}
