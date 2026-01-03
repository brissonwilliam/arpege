mod config;
mod processor;
mod rest;
mod storage;

// Set env vars with RUST_LOG=debug
use env_logger;

/// Simple program to greet a person
#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    name: String,

    #[arg(short, long, default_value_t = 1)]
    count: u8,
}

#[tokio::main]
async fn main() {
    // initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let import_dir = std::path::Path::new("data/import");
    std::fs::create_dir_all(import_dir).unwrap();

    let fs_dir = std::path::Path::new("data/fs");
    std::fs::create_dir_all(fs_dir).unwrap();

    let _ = config::get(); // calling get() to run the singleton that reads and loads the config to
                           // validate before we start doing other stuff

    let pool = storage::get_pool().await.unwrap_or_else(|err| {
        log::error!("FATAL could not initialize db pool: {}", err.to_string());
        std::process::exit(-1);
    });
    let store = storage::Storage::new(pool);

    let mut processor = processor::Processor::new();
    processor.start(store);

    processor.push(processor::ProcessorJob {
        path: String::from("./data/import/ato.m4a"),
        override_existing: false,
    });

    rest::start().await;
}
