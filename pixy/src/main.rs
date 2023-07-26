use clap::Parser;
use pixy::cli::Cli;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    pixy::logging::setup_logging(&cli);

    pixy::run(cli).await;
}
