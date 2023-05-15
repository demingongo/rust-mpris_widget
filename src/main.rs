use mpris_widget::Config;
use std::{env, process};

#[tokio::main] // to allow 'main' function to be async
async fn main() {
    let config = Config::build(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    // run application
    if let Err(e) = mpris_widget::run(config).await {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}
