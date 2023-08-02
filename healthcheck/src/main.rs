use std::{env, process::ExitCode};

#[inline]
fn get(url: &str) -> bool {
    minreq::get(url)
        .send()
        .map(|res| {
            println!("Received status code {}", res.status_code);
            res
        })
        .map_err(|e| println!("{}", e))
        .is_ok_and(|res| (200..=299).contains(&res.status_code))
}

fn main() -> ExitCode {
    let port = env::var("PIXY_PORT").unwrap_or_else(|_| String::from("8000"));
    let endpoint = format!("http://localhost:{}/healthz", port);

    if get(&endpoint) {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::MockServer;

    fn test_get_failure(status_code: u16, success: bool) {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::GET);
            then.status(status_code);
        });

        let res = get(&server.url("/healthz"));

        assert_eq!(res, success);
        mock.assert();
    }

    macro_rules! test_get {
        ($($a:ident: $b:expr, $c:expr,)*) => {
        mod test_get {
                use super::*;
            $(
                #[test]
                fn $a() {
                    test_get_failure($b, $c);
                }
            )*
        }
        };
    }

    test_get!(
        success_when_200: 200, true,
        success_when_201: 201, true,
        success_when_219: 219, true,
        failure_when_300: 300, false,
        failure_when_400: 400, false,
        failure_when_401: 401, false,
        failure_when_404: 404, false,
        failure_when_500: 500, false,
    );
}
