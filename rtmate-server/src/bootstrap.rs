use tracing_subscriber::{EnvFilter, fmt, registry};
use tracing_subscriber::prelude::*; // bring SubscriberExt into scope for .with()
use std::sync::Arc;
use crate::web_context::WebContext;

pub fn init_tracing() {
    registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(fmt::layer())
        .init();
}

pub async fn build_web_context() -> anyhow::Result<Arc<WebContext>> {
    let ctx = WebContext::new().await?;
    Ok(ctx.into())
}
