pub mod error;

pub use error::*;

use clap::Parser;

#[derive(Parser)]
#[command(about, long_about = None, version)]
pub struct AppOptions {
  query: String,
}

pub struct App {
  options: AppOptions,
}

impl App {
  pub fn new() -> crate::Result<Self> {
    let options = AppOptions::try_parse()?;
    Ok(Self { options })
  }

  pub fn run(self) -> crate::Result<()> {
    println!("Looking for '{}'", self.options.query);
    Ok(())
  }
}
