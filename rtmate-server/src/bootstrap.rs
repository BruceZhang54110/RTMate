use tracing_subscriber::{EnvFilter, fmt, registry, prelude::*, layer::SubscriberExt};
use tracing_log::LogTracer;
use std::sync::Arc;
use crate::web_context::WebContext;
use tracing_subscriber::fmt::time::LocalTime;
use time::macros::format_description;

pub fn init_tracing() {
    // 先桥接 log 到 tracing
    let timer = LocalTime::new(format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]"));
    registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=info", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(fmt::layer()
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_target(false)
            .with_level(true)
            .with_line_number(true)
            .with_file(true)
            .with_timer(timer)
        )
        .init();
}

pub async fn build_web_context() -> anyhow::Result<Arc<WebContext>> {
    let ctx = WebContext::new().await?;
    Ok(ctx.into())
}
