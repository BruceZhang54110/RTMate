// 统一的错误类型与响应信封已从 rtmate-common 提供。
// 本模块保留 re-export，以兼容现有 `crate::common::{AppError, BizError, ValidationErrorDetail}` 的用法。
pub use rtmate_common::common::{AppError, BizError, ValidationErrorDetail};
