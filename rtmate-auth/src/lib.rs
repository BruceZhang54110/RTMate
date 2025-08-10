pub mod web_context;
pub mod service;
pub mod db;
pub mod dao;


#[cfg(test)]
mod tests {

    use crate::dao::Dao;

    #[tokio::test]
    async fn test_dao() {
        let dao = Dao::new().await.unwrap();
        let res = dao.query().await.unwrap();
        println!("res: {}", res);
    }
}