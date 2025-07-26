use std::collections::HashMap;

pub struct Store {
    // 存储应用认证信息, appId -> appKey
    pub data: HashMap<String, String>,
}

impl Store {
    pub fn new() -> Self {
        Store {
            data: HashMap::new(),
        }
    }

    pub fn insert(&mut self, app_id: String, app_key: String) {
        self.data.insert(app_id, app_key);
    }

    pub fn get(&mut self, app_id: &str) -> Option<&String> {
        self.data.get(app_id)
    }
    
}