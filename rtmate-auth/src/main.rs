use rtmate_auth::web_context;

#[tokio::main]
async fn main() {
    web_context::startup().await;
}
