use crate::{Project, ProjectMatchesFormatter};

/// The most basic project writer: a human readable list on stdout
pub struct JsonProjectMatchesWriter {}

impl ProjectMatchesFormatter for JsonProjectMatchesWriter {
  fn write(
    &self,
    to: &mut dyn std::io::Write,
    matches: &Vec<Project>,
  ) -> crate::Result<()> {
    write!(to, "{}", serde_json::to_string_pretty(matches)?)?;
    Ok(())
  }
}
