pub mod web_context;
pub mod service;
pub mod db;
pub mod dao;
pub mod schema;
pub mod models;
pub mod common;


#[cfg(test)]
mod tests {

    use crate::dao::Dao;
    use crate::common::BizError;
    use crate::common::AppError;
    use anyhow::anyhow;

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


    // 测试 BizError::AppNotFound 是否能正确转换为 AppError
    #[test]
    fn test_biz_error_to_app_error_conversion() {
        // 创建一个 BizError 实例
        let biz_error = BizError::AppNotFound;
        let app_error: AppError = biz_error.into();
        assert_eq!(app_error.code, 1004);
        assert_eq!(app_error.message, "您的app未找到，请检查appId");
    }

    // 测试 anyhow::Error 是否能正确转换为 AppError
    #[test]
    fn test_anyhow_error_to_app_error_conversion() {
        let anyhow_error = anyhow!("数据库连接失败");
        let app_error: AppError = anyhow_error.into();
        assert_eq!(app_error.code, 500);
        assert_eq!(app_error.message, "系统内部错误");

    }


}