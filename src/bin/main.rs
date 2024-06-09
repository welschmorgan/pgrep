use std::process::ExitCode;

use log::error;
use pgrep::App;

fn run() -> pgrep::Result<()> {
  App::new()?.run()
}

fn main() -> ExitCode {
  if let Err(e) = run() {
    error!("\x1b[1mfatal\x1b[0m: {}", e);
    return ExitCode::FAILURE;
  }
  ExitCode::SUCCESS
}
