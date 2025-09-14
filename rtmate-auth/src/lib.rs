pub mod web_context;
pub mod service;
pub mod db;
pub mod schema;
pub mod models;
pub mod common;
pub mod dto;
pub mod dao_query;


#[cfg(test)]
mod tests {

    use crate::common::AppError;
    use anyhow::anyhow;
    use crate::common::BizError;



    // 测试 BizError::AppNotFound 是否能正确转换为 AppError
    #[test]
    fn test_biz_error_to_app_error_conversion() {
        // 创建一个 BizError 实例
        let biz_error = BizError::AppNotFound;
        let app_error = AppError::from(biz_error);
        assert_eq!(app_error.code, 1004);
        assert_eq!(app_error.message, "您的app未找到，请检查appId");
    }

    // 测试 anyhow::Error 是否能正确转换为 AppError
    #[test]
    fn test_anyhow_error_to_app_error_conversion() {
        let anyhow_error = anyhow!("数据库连接失败");
        let app_error = AppError::from(anyhow_error);
        assert_eq!(app_error.code, 500);
        assert_eq!(app_error.message, "系统内部错误");
    }


}