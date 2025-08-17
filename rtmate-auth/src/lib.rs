pub mod web_context;
pub mod service;
pub mod db;
pub mod dao;
pub mod schema;
pub mod models;


#[cfg(test)]
mod tests {

    use crate::dao::Dao;

    #[tokio::test]
    async fn test_dao() {
        let dao = Dao::new().await.unwrap();
        let res = dao.query().await.unwrap();
        println!("res: {}", res);
    }

    #[tokio::test]
    async fn test_query_all_rt_app() {
        let dao = Dao::new().await.unwrap();
        let res = dao.query_all_rt_app().await;
        match res {
            Ok(_) => println!("Query all rt_app successful"),
            Err(e) => println!("Error querying rt_app: {}", e),
        }
    }
}