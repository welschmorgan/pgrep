use std::io::{BufWriter, Write as _};

use clap::ValueEnum;
use strum::{EnumIter, VariantNames};

use crate::{BoxedProjectMatchesFormatter, Project};

pub trait UI {
  /// Write a string directly to the screen
  fn write_matches(
    &mut self,
    matches: &Vec<Project>,
    fmt: &BoxedProjectMatchesFormatter,
  ) -> crate::Result<()>;

  /// Write a log message
  fn write_log(&mut self, text: &str, lvl: log::Level) -> crate::Result<()>;

  /// Custom render loop
  fn render_loop(&mut self) -> crate::Result<()> {
    Ok(())
  }
}

pub type BoxedUI = Box<dyn UI>;
