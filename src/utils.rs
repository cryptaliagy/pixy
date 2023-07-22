use std::time::Duration;

/// Formats a `Duration` as a string.
///
/// This takes into account the resolution of the duration to give the most
/// appropriate message for its resolution
///
/// ```
/// # use std::time::Duration;
/// # use websvc::utils::format_duration;
///
/// let duration = Duration::from_millis(1005);
/// let milliduration = Duration::from_micros(1005);
/// let nanoduration = Duration::from_nanos(1005);
///
/// assert_eq!("1.005 s", &format_duration(duration));
/// assert_eq!("1.005 ms", &format_duration(milliduration));
/// assert_eq!("1.005 μs", &format_duration(nanoduration));
/// ```
pub fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    let millis = duration.subsec_millis();
    let micros = duration.subsec_micros() - millis * 1000;
    let nanos = duration.subsec_nanos() - micros * 1000;
    if secs > 0 {
        format!("{}.{:0>3} s", secs, millis)
    } else if millis > 0 {
        format!("{}.{:0>3} ms", millis, micros)
    } else {
        format!("{}.{:0>3} μs", micros, nanos)
    }
}
