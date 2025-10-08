
use rtmate_common::dao::Dao;

#[derive(Clone)]
pub struct WebContext {

    // 数据源
    pub dao: Dao,
}

impl WebContext {
    pub async fn new() -> anyhow::Result<Self> {
        let dao = Dao::new().await?;
        Ok(WebContext { dao })
    }

}