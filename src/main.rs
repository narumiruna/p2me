mod cli;

use std::process::ExitCode;

fn main() -> ExitCode {
    #[cfg(all(feature = "test-endpoint", debug_assertions))]
    if let Some(port) = std::env::var_os("P2ME_TEST_PORT") {
        let Some(port) = port
            .to_str()
            .and_then(|port| port.parse::<std::num::NonZeroU16>().ok())
        else {
            eprintln!("error: invalid test server port");
            return ExitCode::FAILURE;
        };
        return cli::run(&format!("http://127.0.0.1:{port}/api/v1"));
    }

    cli::run("https://prove2.me/api/v1")
}
