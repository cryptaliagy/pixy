use clap::Parser;
use janus_cli::cli::Cli;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    janus_cli::run(cli).await;
}
