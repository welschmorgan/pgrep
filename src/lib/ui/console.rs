use crate::{BoxedProjectMatchesFormatter, Project, UI};

/// Represent the raw console, where messages are just appended to each other (stdout by default)
pub struct Console;

impl Console {
  pub fn new() -> Self {
    Self
  }
}

impl UI for Console {
  fn write_matches(
    &mut self,
    matches: &Vec<Project>,
    fmt: &BoxedProjectMatchesFormatter,
  ) -> crate::Result<()> {
    fmt.write(&mut std::io::stdout(), matches)?;
    Ok(())
  }

  fn write_log(&mut self, text: &str, lvl: log::Level) -> crate::Result<()> {
    log::log!(lvl, "{}", text);
    Ok(())
  }
}
