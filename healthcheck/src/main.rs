use std::{env, process::ExitCode};

fn main() -> ExitCode {
    let port = env::var("PIXY_PORT").unwrap_or_else(|_| String::from("8000"));
    let endpoint = format!("http://localhost:{}/healthz", port);

    minreq::get(endpoint)
        .send()
        .map(|res| {
            if !(200..=299).contains(&res.status_code) {
                println!("Received status code {}", res.status_code);
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        })
        .unwrap_or_else(|e| {
            println!("{}", e);
            ExitCode::FAILURE
        })
}
