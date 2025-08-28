mod rest;
mod storage;
use env_logger;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!("calisss");
    storage::init();
    rest::start().await;
}
