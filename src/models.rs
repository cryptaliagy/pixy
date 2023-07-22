use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::serde::Serialize;
use rocket::{Data, Request, Response};
use std::time::SystemTime;

use log::info;

use crate::utils;

/// Fairing for timing requests.
#[derive(Default)]
pub struct RequestTimer {}

/// Value stored in request-local state.
#[derive(Copy, Clone)]
pub struct TimerStart(Option<SystemTime>);

/// Struct for serializing a status message.
#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct StatusMessage {
    pub status: String,
    pub message: String,
}

/// A trait defining what should constitute a configuration
/// for the application
pub trait Configuration {
    /// Gets a value indicating whether profiling should be
    /// enabled or not.
    fn profiling_enabled(&self) -> bool;

    /// Gets a value that is either a &str of the maintenance
    /// webhook url, or None if no webhook is set.
    fn maintenance_webhook(&self) -> Option<&str>;

    /// Gets a value that is either a &str of the feedback webhook
    /// url, or None if no webhook is set.
    fn feedback_webhook(&self) -> Option<&str>;
}

#[rocket::async_trait]
impl Fairing for RequestTimer {
    fn info(&self) -> Info {
        Info {
            name: "Request Timer",
            kind: Kind::Request | Kind::Response,
        }
    }

    /// Stores the start time of the request in request-local state. This is used
    /// during the response fairing to calculate the request runtime.
    async fn on_request(&self, request: &mut Request<'_>, _: &mut Data<'_>) {
        // Store a `TimerStart` instead of directly storing a `SystemTime`
        // to ensure that this usage doesn't conflict with anything else
        // that might store a `SystemTime` in request-local cache.
        request.local_cache(|| TimerStart(Some(SystemTime::now())));
    }

    /// Calculates the duration of time that the request has taken and prints it to the logs.
    /// This will also inject that duration as a header on the response.
    async fn on_response<'r>(&self, req: &'r Request<'_>, res: &mut Response<'r>) {
        let start_time = req.local_cache(|| TimerStart(None));
        if let Some(Ok(duration)) = start_time.0.map(|st| st.elapsed()) {
            let formatted = utils::format_duration(duration);
            info!(
                "{method:<7} | {duration:>12} | {status} | \"{uri}\"",
                method = req.method(),
                uri = req.uri(),
                duration = formatted,
                status = res.status().code,
            );

            res.set_header(Header::new("X-Request-Duration", formatted));
        }
    }
}
