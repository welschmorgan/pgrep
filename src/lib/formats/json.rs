use crate::{Project, ProjectMatchesWriter};

/// The most basic project writer: a human readable list on stdout
pub struct JsonProjectMatchesWriter {}

impl ProjectMatchesWriter for JsonProjectMatchesWriter {
  fn write<'a>(
    &'a self,
    to: &'a mut dyn std::io::Write,
    matches: &'a Vec<&'a Project>,
  ) -> crate::Result<()> {
    write!(to, "{}", serde_json::to_string_pretty(matches)?)?;
    Ok(())
  }
}