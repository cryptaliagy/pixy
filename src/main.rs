#[macro_use]
extern crate rocket;

use fern::colors::{Color, ColoredLevelConfig};
use log::{debug, info};

use rocket::fairing::AdHoc;
use rocket::serde::json::Json;
use rocket::{Build, Rocket, State};

use websvc::config::AppConfig;
use websvc::models::{RequestTimer, StatusMessage};

/// A route that returns a 200 status code and a short json message. This is used to
/// confirm that the web server is receiving requests but not performing any specific
/// operation.
#[get("/heartbeat")]
async fn heartbeat() -> Json<StatusMessage> {
    Json(StatusMessage {
        status: "ok".to_string(),
        message: "The server is running".to_string(),
    })
}

/// A debug route (this only compiles when using the debug profile, so it doesn't exist in
/// production) to retrieve the current running configuration of the application. This shows
/// how to retrieve state managed by the application
#[cfg(debug_assertions)]
#[get("/config")]
async fn show_configs(configs: &State<AppConfig>) -> Json<&AppConfig> {
    Json(configs)
}

/// Constructs the rocket that will be used based on the configuration passed to this function.
/// This will then be used by the `rocket()` function to launch the application.
///
/// The configuration passed in will be made available to routes using the `&State<AppConfig>`
/// type as a parameter in the function.
fn build_rocket(configs: AppConfig) -> Rocket<Build> {
    info!("Building rocket...");
    let ship = rocket::build();

    // All fairings should be attached below here and before the routes
    // vector is constructed. This ensures that the logging fairings are the
    // last ones to be executed.
    let ship = if configs.profiling_enabled() {
        debug!("Profiling enabled! Attaching fairing...");
        ship.attach(RequestTimer::default())
    } else {
        ship
    };

    #[allow(unused_mut)]
    let mut routes = routes![heartbeat];

    // Since `show_configs` doesn't exist when compiling the release profile,
    // we need to use the same macro under this scope to prevent the scope from being
    // compiled in release mode. This is useful if there's any routes that would
    // be a security risk in production but are useful to have in development.
    //
    // If we remove the `#[cfg(debug_assertions)]` macro from the `show_configs`
    // route, we could still add the route only conditionally by using
    // `if cfg!(debug_assertions) {}`
    #[cfg(debug_assertions)]
    {
        debug!("Debug profile enabled! Adding debug routes to routes vector...");
        let mut debug_routes = routes![show_configs];

        routes.append(&mut debug_routes);
    };

    debug!("Mounting state and routes...");
    ship.attach(AdHoc::on_ignite("logging ignite", |rocket| async {
        info!("Ignition complete! Launching rocket...");
        rocket
    }))
    .attach(AdHoc::on_liftoff("logging liftoff", |_| {
        Box::pin(async { info!("Launch complete! Service 'websvc' is running") })
    }))
    .manage(configs)
    .mount("/", routes)
}

/// This compiles down to the main function of the application using the #[launch] macro.
/// It creates the configuration from the environment, then uses that to construct the web
/// server.
#[launch]
async fn rocket() -> _ {
    #[cfg(debug_assertions)]
    println!("Building configuration...");
    let config = AppConfig::build().expect("Could not build configuration from environment");

    #[cfg(debug_assertions)]
    println!("Building logger...");

    let date_fmt: String = config.time_format().to_string();

    let colors = ColoredLevelConfig::new()
        .debug(Color::Cyan)
        .info(Color::Green)
        .warn(Color::Yellow)
        .error(Color::Red);

    let mut log_config = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{}][{}][{}]\t{}",
                chrono::Utc::now().format(&date_fmt),
                record.target(),
                colors.color(record.level()),
                message,
            ))
        })
        .level(config.level())
        .chain(std::io::stdout());

    if !config.log_all() {
        log_config = log_config.filter(|metadata| metadata.target().starts_with("websvc"))
    }

    log_config.apply().unwrap();

    debug!("Logger configuration finished!");
    let ship = build_rocket(config);

    info!("Rocket build! Igniting rocket...");
    ship
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    use rocket::http::{ContentType, Status};
    use rocket::local::blocking::{Client, LocalResponse};

    /// Creates a test client using the specified configuration
    fn scaffold_client_with(configs: AppConfig) -> Client {
        Client::tracked(build_rocket(configs)).expect("valid rocket instance")
    }

    /// Creates a test client using the default configuration
    fn scaffold_client() -> Client {
        scaffold_client_with(AppConfig::default())
    }

    /// Given a local response generated from a test client, parses the X-Request-Duration header
    /// injected by the request timer faring and returns the integer portion as a duration
    fn duration_from_response(response: &LocalResponse<'_>) -> Duration {
        // Parse the header in form of "X-Request-Duration: 12.34 <unit>"
        // where <unit> is either "s", "ms" or "µs" into a Duration.
        // If the unit is ms or µs, the numeric value is less than 1000.
        let duration_header = response.headers().get_one("X-Request-Duration").unwrap();
        let duration = duration_header
            .split_whitespace()
            .next()
            .unwrap()
            .split_terminator('.') // Ignore the decimal part
            .next()
            .unwrap()
            .parse::<u64>()
            .unwrap();

        let unit = duration_header.split_whitespace().last().unwrap();

        match unit {
            "s" => std::time::Duration::from_secs(duration),
            "ms" => std::time::Duration::from_millis(duration),
            "μs" => std::time::Duration::from_micros(duration),
            _ => panic!("Unknown unit {}", unit),
        }
    }

    /// Test that the heartbeat endpoint returns a 200 status code and a JSON
    /// response.
    #[test]
    fn test_heartbeat() {
        let client = scaffold_client();
        let response = client.get("/heartbeat").dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.content_type(),
            Some(ContentType::new("application", "json"))
        );
    }

    /// Test that a request returns a header with the duration of the request if
    /// profiling is enabled, and that the duration is smaller than 1 ms (if debug)
    /// or smaller than 200 μs (if release).
    #[test]
    fn test_profiling() {
        let mut configs = AppConfig::default();
        configs.enable_profiling(true);

        let client = scaffold_client_with(configs);
        let response = client.get("/heartbeat").dispatch();

        let max_duration = if cfg!(debug_assertions) {
            std::time::Duration::from_micros(1000)
        } else {
            std::time::Duration::from_micros(200)
        };

        assert_eq!(response.status(), Status::Ok);
        assert!(response.headers().get_one("X-Request-Duration").is_some());

        let duration = duration_from_response(&response);

        assert!(duration < max_duration);
    }

    /// Test that the config route returns the accurate configuration as passed in
    /// to the scaffolding.
    ///
    /// This test only runs when `--release` is NOT passed into `cargo test`.
    #[cfg(debug_assertions)]
    #[test]
    fn test_configs() {
        let mut configs = AppConfig::default();
        configs.enable_profiling(true);

        let cloned = configs.clone();

        assert_eq!(configs, cloned);

        let client = scaffold_client_with(cloned);
        let response = client.get("/config").dispatch();

        assert_eq!(response.status(), Status::Ok);

        let returned_configs: AppConfig = response.into_json().unwrap();

        assert_eq!(configs, returned_configs);
    }
}
