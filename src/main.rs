mod cli;

use std::process::ExitCode;

fn main() -> ExitCode {
    cli::run("https://prove2.me/api/v1")
}
