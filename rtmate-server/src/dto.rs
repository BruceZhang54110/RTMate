use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct AuthResponse {
    pub state: bool,
    pub client_id: String,
}

impl AuthResponse {
    pub fn new(state: bool, client_id: String) -> Self {
        AuthResponse { state, client_id }
    }
    
}


#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum WsData {

    Auth(AuthResponse),
}